//! HTTPS tunnel runtime integration.
//!
//! This module intentionally wraps the HTTP runtime instead of copying it. The
//! HTTPS listener owns TCP accept and TLS handshake; once a TLS stream exists it
//! is handed to `HttpTunnelRuntime::handle_stream`, so request parsing, routing,
//! forwarding, keep-alive, chunked bodies, and HTTP logs stay in one place.

use crate::core::TunnelId;
use crate::runtime::config::{BufferConfig, ListenerConfig, RetryConfig, TimeoutConfig};
use crate::runtime::error::{HttpsError, ListenerError, RuntimeError};
use crate::runtime::http::{
    HttpConnectionMetadata, HttpHostResolver, HttpRouteConfig, HttpRuntimeConfig,
    HttpRuntimeMetrics, HttpTlsLog, HttpTunnelRuntime,
};
use crate::runtime::lifecycle::RuntimeLifecycle;
use crate::runtime::monitor::{RuntimeMetrics, RuntimeMonitor};
use crate::runtime::scheduler::RuntimeScheduler;
use crate::runtime::state::{RuntimeState, SessionState};
use crate::runtime::worker::TaskId;
use futures::future::BoxFuture;
use gate_server_tls::cert_store::CertificateStore;
use gate_server_tls::certificate::{CertificateRecord, CertificateStatus};
use gate_server_tls::{CertificateMaterial, TlsProvider};
use parking_lot::{Mutex, RwLock};
use rustls::crypto::ring;
use rustls::server::{ClientHello, ResolvesServerCert};
use rustls::sign::CertifiedKey;
use rustls::{ProtocolVersion, ServerConfig, SupportedProtocolVersion};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt;
use std::io::Cursor;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::net::{TcpListener as TokioTcpListener, TcpStream};
use tokio::sync::{mpsc, watch};
use tokio::time::sleep;
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;
use tracing::{debug, error, info, warn};

const RECENT_TLS_LIMIT: usize = 128;

/// TLS protocol versions exposed in HTTPS settings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TlsProtocolVersion {
    Tls12,
    Tls13,
}

impl Default for TlsProtocolVersion {
    fn default() -> Self {
        Self::Tls12
    }
}

/// HTTPS listener and TLS settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpsTlsConfig {
    pub enabled: bool,
    pub http_redirect: bool,
    pub preferred_tls_version: Option<TlsProtocolVersion>,
    pub minimum_tls_version: TlsProtocolVersion,
    pub cipher_suites: Vec<String>,
    pub ocsp_enabled: bool,
    pub hsts_enabled: bool,
    pub default_domain: Option<String>,
    pub certificate_domains: Vec<String>,
}

impl Default for HttpsTlsConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            http_redirect: false,
            preferred_tls_version: Some(TlsProtocolVersion::Tls13),
            minimum_tls_version: TlsProtocolVersion::Tls12,
            cipher_suites: Vec::new(),
            ocsp_enabled: false,
            hsts_enabled: false,
            default_domain: None,
            certificate_domains: Vec::new(),
        }
    }
}

/// Certificate source used by the HTTPS runtime.
pub trait HttpsCertificateProvider: Send + Sync {
    fn load_all<'a>(&'a self) -> BoxFuture<'a, Result<Vec<CertificateMaterial>, HttpsError>>;

    fn load<'a>(
        &'a self,
        domain: &'a str,
    ) -> BoxFuture<'a, Result<CertificateMaterial, HttpsError>>;

    fn reload<'a>(
        &'a self,
        domain: &'a str,
    ) -> BoxFuture<'a, Result<CertificateMaterial, HttpsError>>;

    fn request_missing<'a>(&'a self, _domain: &'a str) -> BoxFuture<'a, Result<(), HttpsError>> {
        Box::pin(async { Ok(()) })
    }
}

/// Adapter for the existing certificate store infrastructure.
#[derive(Debug, Clone)]
pub struct StoreCertificateProvider<S>
where
    S: CertificateStore,
{
    store: S,
}

impl<S> StoreCertificateProvider<S>
where
    S: CertificateStore,
{
    pub fn new(store: S) -> Self {
        Self { store }
    }
}

impl<S> HttpsCertificateProvider for StoreCertificateProvider<S>
where
    S: CertificateStore + 'static,
{
    fn load_all<'a>(&'a self) -> BoxFuture<'a, Result<Vec<CertificateMaterial>, HttpsError>> {
        Box::pin(async move {
            let mut materials = Vec::new();
            for record in self.store.list().map_err(HttpsError::certificate_source)? {
                let stored = self
                    .store
                    .load(&record.domain)
                    .map_err(HttpsError::certificate_source)?;
                materials.push(stored.into());
            }
            Ok(materials)
        })
    }

    fn load<'a>(
        &'a self,
        domain: &'a str,
    ) -> BoxFuture<'a, Result<CertificateMaterial, HttpsError>> {
        Box::pin(async move {
            self.store
                .load(domain)
                .map(CertificateMaterial::from)
                .map_err(HttpsError::certificate_source)
        })
    }

    fn reload<'a>(
        &'a self,
        domain: &'a str,
    ) -> BoxFuture<'a, Result<CertificateMaterial, HttpsError>> {
        self.load(domain)
    }
}

/// Adapter for the existing `TlsProvider` abstraction.
#[derive(Debug)]
pub struct TlsProviderCertificateProvider<P>
where
    P: TlsProvider,
{
    provider: Mutex<P>,
    domains: RwLock<Vec<String>>,
}

impl<P> TlsProviderCertificateProvider<P>
where
    P: TlsProvider,
{
    pub fn new(provider: P, domains: Vec<String>) -> Self {
        Self {
            provider: Mutex::new(provider),
            domains: RwLock::new(domains),
        }
    }

    pub fn add_domain(&self, domain: impl Into<String>) {
        let domain = normalize_host(&domain.into());
        let mut domains = self.domains.write();
        if !domains.iter().any(|item| item == &domain) {
            domains.push(domain);
        }
    }
}

impl<P> HttpsCertificateProvider for TlsProviderCertificateProvider<P>
where
    P: TlsProvider + Send + 'static,
{
    fn load_all<'a>(&'a self) -> BoxFuture<'a, Result<Vec<CertificateMaterial>, HttpsError>> {
        Box::pin(async move {
            let domains = self.domains.read().clone();
            let mut materials = Vec::new();
            for domain in domains {
                materials.push(self.reload(&domain).await?);
            }
            Ok(materials)
        })
    }

    fn load<'a>(
        &'a self,
        domain: &'a str,
    ) -> BoxFuture<'a, Result<CertificateMaterial, HttpsError>> {
        self.reload(domain)
    }

    fn reload<'a>(
        &'a self,
        domain: &'a str,
    ) -> BoxFuture<'a, Result<CertificateMaterial, HttpsError>> {
        Box::pin(async move {
            let mut provider = self.provider.lock();
            provider.reload(domain).map_err(HttpsError::tls_provider)
        })
    }
}

/// Runtime configuration with injected TLS dependencies.
#[derive(Clone)]
pub struct HttpsRuntimeConfig {
    pub http: HttpRuntimeConfig,
    pub tls: HttpsTlsConfig,
    pub certificates: Arc<dyn HttpsCertificateProvider>,
    pub host_resolver: Option<Arc<dyn HttpHostResolver>>,
}

impl fmt::Debug for HttpsRuntimeConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HttpsRuntimeConfig")
            .field("http", &self.http)
            .field("tls", &self.tls)
            .field("has_host_resolver", &self.host_resolver.is_some())
            .finish_non_exhaustive()
    }
}

/// Builder for [`HttpsRuntimeConfig`] and [`HttpsTunnelRuntime`].
#[derive(Clone, Default)]
pub struct HttpsRuntimeBuilder {
    http: HttpRuntimeConfig,
    tls: HttpsTlsConfig,
    certificates: Option<Arc<dyn HttpsCertificateProvider>>,
    host_resolver: Option<Arc<dyn HttpHostResolver>>,
}

impl HttpsRuntimeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn http_config(mut self, http: HttpRuntimeConfig) -> Self {
        self.http = http;
        self
    }

    pub fn tls_config(mut self, tls: HttpsTlsConfig) -> Self {
        self.tls = tls;
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.http.base.name = name.into();
        self
    }

    pub fn tunnel_id(mut self, tunnel_id: TunnelId) -> Self {
        self.http.base.listener.tunnel_id = tunnel_id;
        self
    }

    pub fn listen_addr(mut self, listen_addr: SocketAddr) -> Self {
        self.http.base.listener.listen_addr = listen_addr;
        self
    }

    pub fn listener(mut self, listener: ListenerConfig) -> Self {
        self.http.base.listener = listener;
        self
    }

    pub fn buffer(mut self, buffer: BufferConfig) -> Self {
        self.http.base.buffer = buffer;
        self
    }

    pub fn retry(mut self, retry: RetryConfig) -> Self {
        self.http.base.retry = retry;
        self
    }

    pub fn timeout(mut self, timeout: TimeoutConfig) -> Self {
        self.http.base.timeout = timeout;
        self
    }

    pub fn route(mut self, mut route: HttpRouteConfig) -> Self {
        route.headers.forwarded_proto = "https".to_string();
        self.http.routes.push(route);
        self
    }

    pub fn routes(mut self, routes: Vec<HttpRouteConfig>) -> Self {
        self.http.routes = routes
            .into_iter()
            .map(|mut route| {
                route.headers.forwarded_proto = "https".to_string();
                route
            })
            .collect();
        self
    }

    pub fn certificate_provider(mut self, certificates: Arc<dyn HttpsCertificateProvider>) -> Self {
        self.certificates = Some(certificates);
        self
    }

    pub fn host_resolver(mut self, resolver: Arc<dyn HttpHostResolver>) -> Self {
        self.host_resolver = Some(resolver);
        self
    }

    pub fn build_config(self) -> Result<HttpsRuntimeConfig, HttpsError> {
        let certificates = self.certificates.ok_or_else(|| HttpsError::Certificate {
            reason: "HTTPS certificate provider is required".to_string(),
        })?;
        Ok(HttpsRuntimeConfig {
            http: self.http,
            tls: self.tls,
            certificates,
            host_resolver: self.host_resolver,
        })
    }

    pub fn build(self) -> Result<HttpsTunnelRuntime, HttpsError> {
        Ok(HttpsTunnelRuntime::new(self.build_config()?))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CertificateSnapshot {
    pub domain: String,
    pub status: String,
    pub expire_days: i64,
    pub issuer: String,
    pub fingerprint_sha256: String,
    pub san: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HttpsRuntimeMetrics {
    pub http: HttpRuntimeMetrics,
    pub certificates: Vec<CertificateSnapshot>,
    pub handshake_count: u64,
    pub handshake_error_count: u64,
    pub average_handshake_ms: f64,
    pub recent_tls_versions: Vec<String>,
    pub https_upload_bytes: u64,
    pub https_download_bytes: u64,
    pub error_count: u64,
}

#[derive(Debug)]
struct HttpsMetricsState {
    handshake_count: u64,
    handshake_error_count: u64,
    total_handshake_ms: u128,
    recent_tls_versions: VecDeque<String>,
}

impl HttpsMetricsState {
    fn new() -> Self {
        Self {
            handshake_count: 0,
            handshake_error_count: 0,
            total_handshake_ms: 0,
            recent_tls_versions: VecDeque::new(),
        }
    }
}

#[derive(Debug)]
struct HttpsMetricsStore {
    state: Mutex<HttpsMetricsState>,
}

impl HttpsMetricsStore {
    fn new() -> Self {
        Self {
            state: Mutex::new(HttpsMetricsState::new()),
        }
    }

    fn record_handshake(&self, handshake_ms: u64, tls_version: Option<&str>) {
        let mut state = self.state.lock();
        state.handshake_count = state.handshake_count.saturating_add(1);
        state.total_handshake_ms = state
            .total_handshake_ms
            .saturating_add(u128::from(handshake_ms));
        if let Some(tls_version) = tls_version {
            state.recent_tls_versions.push_back(tls_version.to_string());
            while state.recent_tls_versions.len() > RECENT_TLS_LIMIT {
                let _ = state.recent_tls_versions.pop_front();
            }
        }
    }

    fn record_handshake_error(&self) {
        let mut state = self.state.lock();
        state.handshake_error_count = state.handshake_error_count.saturating_add(1);
    }
}

struct HttpsTunnelRuntimeInner {
    http: HttpTunnelRuntime,
    tls: HttpsTlsConfig,
    certificates: Arc<dyn HttpsCertificateProvider>,
    host_resolver: Option<Arc<dyn HttpHostResolver>>,
    cache: Arc<TlsCertificateCache>,
    metrics: Arc<HttpsMetricsStore>,
    status: RwLock<HttpsListenerStatus>,
    task_id: RwLock<Option<TaskId>>,
    bound_addr: RwLock<Option<SocketAddr>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HttpsListenerStatus {
    Created,
    Listening,
    Stopped,
}

/// HTTPS tunnel runtime entry point.
#[derive(Clone)]
pub struct HttpsTunnelRuntime {
    inner: Arc<HttpsTunnelRuntimeInner>,
}

impl fmt::Debug for HttpsTunnelRuntime {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("HttpsTunnelRuntime")
            .field("tunnel_id", &self.tunnel_id())
            .field("bound_addr", &self.bound_addr())
            .field("state", &self.inner.http.context().state.current())
            .finish()
    }
}

/// Public HTTPS tunnel type alias for management-layer naming.
pub type HttpsTunnel = HttpsTunnelRuntime;

impl HttpsTunnelRuntime {
    pub fn builder() -> HttpsRuntimeBuilder {
        HttpsRuntimeBuilder::default()
    }

    pub fn new(config: HttpsRuntimeConfig) -> Self {
        let cache = Arc::new(TlsCertificateCache::new(config.tls.default_domain.clone()));
        Self {
            inner: Arc::new(HttpsTunnelRuntimeInner {
                http: HttpTunnelRuntime::new(config.http),
                tls: config.tls,
                certificates: config.certificates,
                host_resolver: config.host_resolver,
                cache,
                metrics: Arc::new(HttpsMetricsStore::new()),
                status: RwLock::new(HttpsListenerStatus::Created),
                task_id: RwLock::new(None),
                bound_addr: RwLock::new(None),
            }),
        }
    }

    pub fn context(&self) -> Arc<crate::runtime::context::RuntimeContext> {
        self.inner.http.context()
    }

    pub fn scheduler(&self) -> Arc<RuntimeScheduler> {
        self.inner.http.scheduler()
    }

    pub fn monitor(&self) -> Arc<RuntimeMonitor> {
        self.inner.http.monitor()
    }

    pub fn metrics(&self) -> RuntimeMetrics {
        self.inner.http.metrics()
    }

    pub fn http_metrics(&self) -> HttpRuntimeMetrics {
        self.inner.http.http_metrics()
    }

    pub fn https_metrics(&self) -> HttpsRuntimeMetrics {
        let runtime_metrics = self.metrics();
        let state = self.inner.metrics.state.lock();
        HttpsRuntimeMetrics {
            http: self.inner.http.http_metrics(),
            certificates: self.inner.cache.snapshots(),
            handshake_count: state.handshake_count,
            handshake_error_count: state.handshake_error_count,
            average_handshake_ms: average(state.total_handshake_ms, state.handshake_count),
            recent_tls_versions: state.recent_tls_versions.iter().cloned().collect(),
            https_upload_bytes: runtime_metrics.upload,
            https_download_bytes: runtime_metrics.download,
            error_count: runtime_metrics.error_count,
        }
    }

    pub fn routes(&self) -> Vec<HttpRouteConfig> {
        self.inner.http.routes()
    }

    pub fn replace_routes(&self, routes: Vec<HttpRouteConfig>) {
        self.inner.http.replace_routes(routes);
    }

    pub fn upsert_route(&self, route: HttpRouteConfig) {
        self.inner.http.upsert_route(route);
    }

    pub fn remove_route(&self, tunnel_id: TunnelId) -> Option<HttpRouteConfig> {
        self.inner.http.remove_route(tunnel_id)
    }

    pub fn bound_addr(&self) -> Option<SocketAddr> {
        *self.inner.bound_addr.read()
    }

    pub fn tunnel_id(&self) -> TunnelId {
        self.inner.http.tunnel_id()
    }

    pub async fn reload_certificate(&self, domain: &str) -> Result<(), HttpsError> {
        let material = self.inner.certificates.reload(domain).await?;
        self.inner.cache.upsert(material)?;
        info!(
            target: "gate_runtime",
            domain = %domain,
            "HTTPS certificate reloaded"
        );
        Ok(())
    }

    pub async fn reload_certificates(&self) -> Result<(), HttpsError> {
        let materials = self.inner.certificates.load_all().await?;
        self.inner.cache.replace_all(materials)?;
        Ok(())
    }

    pub async fn start(&self) -> Result<(), RuntimeError> {
        match self.inner.http.context().state.current() {
            RuntimeState::Initializing | RuntimeState::Starting | RuntimeState::Running => {
                return Ok(())
            }
            RuntimeState::Paused => return self.resume().await,
            RuntimeState::Closed => {
                return Err(RuntimeError::InvalidStateTransition {
                    from: self.inner.http.context().state.current(),
                    to: RuntimeState::Starting,
                });
            }
            RuntimeState::Failed => {
                self.inner
                    .http
                    .context()
                    .state
                    .transition_to(RuntimeState::Restarting)?;
            }
            _ => {}
        }

        if !self.inner.tls.enabled {
            let _ = self
                .inner
                .http
                .context()
                .state
                .transition_to(RuntimeState::Failed);
            return Err(HttpsError::Tls {
                reason: "HTTPS is disabled by configuration".to_string(),
            }
            .into());
        }

        self.inner
            .http
            .context()
            .state
            .transition_to(RuntimeState::Initializing)?;
        self.inner.http.context().reset_shutdown();
        let result = async {
            self.inner
                .http
                .context()
                .state
                .transition_to(RuntimeState::Starting)?;
            self.load_startup_certificates().await?;
            self.start_listener().await?;
            self.start_monitor_task()?;
            self.start_cleanup_task()?;
            self.inner
                .http
                .context()
                .state
                .transition_to(RuntimeState::Running)?;
            Ok::<(), RuntimeError>(())
        }
        .await;

        if let Err(error) = result {
            let _ = self
                .inner
                .http
                .context()
                .state
                .transition_to(RuntimeState::Failed);
            return Err(error);
        }

        info!(
            target: "gate_runtime",
            tunnel_id = %self.tunnel_id(),
            name = %self.inner.http.context().config.name,
            "HTTPS Runtime Start"
        );
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), RuntimeError> {
        match self.inner.http.context().state.current() {
            RuntimeState::Stopped | RuntimeState::Closed => return Ok(()),
            _ => {}
        }

        self.inner
            .http
            .context()
            .state
            .transition_to(RuntimeState::Stopping)?;
        self.inner.http.context().request_shutdown();
        self.stop_listener().await?;
        self.inner
            .http
            .scheduler()
            .graceful_shutdown(self.inner.http.context().config.timeout.shutdown_timeout)
            .await?;
        self.inner
            .http
            .context()
            .sessions
            .close_all(SessionState::Closed);
        self.inner
            .http
            .context()
            .state
            .transition_to(RuntimeState::Stopped)?;
        Ok(())
    }

    pub async fn restart(&self) -> Result<(), RuntimeError> {
        match self.inner.http.context().state.current() {
            RuntimeState::Running | RuntimeState::Paused => {
                self.inner
                    .http
                    .context()
                    .state
                    .transition_to(RuntimeState::Restarting)?;
                self.stop().await?;
            }
            RuntimeState::Created | RuntimeState::Stopped => {}
            RuntimeState::Failed => {
                self.inner
                    .http
                    .context()
                    .state
                    .transition_to(RuntimeState::Restarting)?;
            }
            RuntimeState::Closed => {
                return Err(RuntimeError::InvalidStateTransition {
                    from: RuntimeState::Closed,
                    to: RuntimeState::Starting,
                });
            }
            _ => {}
        }

        self.start().await
    }

    pub async fn pause(&self) -> Result<(), RuntimeError> {
        if self.inner.http.context().state.current() == RuntimeState::Paused {
            return Ok(());
        }

        self.inner
            .http
            .context()
            .state
            .transition_to(RuntimeState::Paused)?;
        Ok(())
    }

    pub async fn resume(&self) -> Result<(), RuntimeError> {
        if self.inner.http.context().state.current() != RuntimeState::Paused {
            return Ok(());
        }

        self.inner
            .http
            .context()
            .state
            .transition_to(RuntimeState::Running)?;
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), RuntimeError> {
        if self.inner.http.context().state.current() == RuntimeState::Closed {
            return Ok(());
        }

        self.inner
            .http
            .context()
            .state
            .transition_to(RuntimeState::Stopping)?;
        self.inner.http.context().request_shutdown();
        self.stop_listener().await?;
        self.inner
            .http
            .scheduler()
            .graceful_shutdown(self.inner.http.context().config.timeout.shutdown_timeout)
            .await?;
        self.inner
            .http
            .context()
            .sessions
            .close_all(SessionState::Closed);
        self.inner
            .http
            .context()
            .state
            .transition_to(RuntimeState::Closed)?;
        Ok(())
    }

    async fn load_startup_certificates(&self) -> Result<(), HttpsError> {
        let mut materials = self.inner.certificates.load_all().await?;
        let configured_domains = self.inner.tls.certificate_domains.clone();
        for domain in configured_domains {
            if materials
                .iter()
                .any(|material| certificate_matches(&material.record, &domain))
            {
                continue;
            }
            match self.inner.certificates.load(&domain).await {
                Ok(material) => materials.push(material),
                Err(error) => {
                    warn!(
                        target: "gate_runtime",
                        domain = %domain,
                        error = %error,
                        "configured HTTPS certificate could not be loaded"
                    );
                }
            }
        }

        self.inner.cache.replace_all(materials)?;
        if self.inner.cache.is_empty() {
            return Err(HttpsError::Certificate {
                reason: "no usable HTTPS certificates were loaded".to_string(),
            });
        }
        Ok(())
    }

    async fn start_listener(&self) -> Result<(), RuntimeError> {
        if self.inner.task_id.read().is_some() {
            return Ok(());
        }

        let listen_addr = self.inner.http.context().config.listener.listen_addr;
        let listener = TokioTcpListener::bind(listen_addr)
            .await
            .map_err(|source| ListenerError::Bind {
                addr: listen_addr,
                source,
            })?;
        let bound_addr = listener
            .local_addr()
            .map_err(|source| ListenerError::Bind {
                addr: listen_addr,
                source,
            })?;

        let acceptor = self.build_acceptor()?;
        *self.inner.bound_addr.write() = Some(bound_addr);
        *self.inner.status.write() = HttpsListenerStatus::Listening;

        let runtime = self.clone();
        let task_id = self.inner.http.scheduler().spawn_listener(
            format!("https-listener:{bound_addr}"),
            async move {
                runtime.accept_loop(listener, acceptor, bound_addr).await;
            },
        )?;
        *self.inner.task_id.write() = Some(task_id);

        info!(
            target: "gate_runtime",
            addr = %bound_addr,
            "HTTPS Listener Start"
        );

        Ok(())
    }

    async fn stop_listener(&self) -> Result<(), ListenerError> {
        if let Some(task_id) = self.inner.task_id.write().take() {
            let _ = self.inner.http.scheduler().cancel(task_id);
        }
        *self.inner.status.write() = HttpsListenerStatus::Stopped;
        Ok(())
    }

    fn build_acceptor(&self) -> Result<TlsAcceptor, HttpsError> {
        let (missing_tx, mut missing_rx) = mpsc::unbounded_channel();
        let cache = Arc::clone(&self.inner.cache);
        let resolver = Arc::new(SniCertificateResolver {
            cache: Arc::clone(&cache),
            missing_tx,
        });
        let config = build_server_config(&self.inner.tls, resolver)?;
        let certificates = Arc::clone(&self.inner.certificates);
        let runtime = self.clone();

        self.inner
            .http
            .scheduler()
            .spawn_cleanup(
                format!("https-cert-missing:{}", self.tunnel_id()),
                async move {
                    while let Some(domain) = missing_rx.recv().await {
                        if runtime.inner.http.context().shutdown_requested() {
                            break;
                        }
                        if let Err(error) = certificates.request_missing(&domain).await {
                            warn!(
                                target: "gate_runtime",
                                domain = %domain,
                                error = %error,
                                "HTTPS missing certificate request failed"
                            );
                            continue;
                        }
                        match certificates.reload(&domain).await {
                            Ok(material) => {
                                if let Err(error) = cache.upsert(material) {
                                    warn!(
                                        target: "gate_runtime",
                                        domain = %domain,
                                        error = %error,
                                        "HTTPS requested certificate could not be activated"
                                    );
                                }
                            }
                            Err(error) => {
                                debug!(
                                    target: "gate_runtime",
                                    domain = %domain,
                                    error = %error,
                                    "HTTPS requested certificate is not available yet"
                                );
                            }
                        }
                    }
                },
            )
            .map_err(|source| HttpsError::Tls {
                reason: source.to_string(),
            })?;

        Ok(TlsAcceptor::from(Arc::new(config)))
    }

    async fn accept_loop(
        self,
        listener: TokioTcpListener,
        acceptor: TlsAcceptor,
        bound_addr: SocketAddr,
    ) {
        let mut shutdown = self.inner.http.context().subscribe_shutdown();

        loop {
            if *shutdown.borrow() {
                break;
            }

            if self.inner.http.context().state.current() == RuntimeState::Paused {
                sleep(self.inner.http.context().config.listener.accept_backoff).await;
                continue;
            }

            tokio::select! {
                _ = shutdown_notified(&mut shutdown) => break,
                accepted = listener.accept() => match accepted {
                    Ok((stream, remote_addr)) => {
                        let runtime = self.clone();
                        let acceptor = acceptor.clone();
                        let task_name = format!("https-handshake:{remote_addr}");
                        if let Err(error) = self.inner.http.scheduler().spawn_forward(task_name, async move {
                            if let Err(error) = runtime.handle_tls_client(stream, acceptor, remote_addr).await {
                                runtime.inner.http.context().traffic.increment_error();
                                error!(
                                    target: "gate_runtime",
                                    remote_addr = %remote_addr,
                                    error = %error,
                                    "HTTPS client handling failed"
                                );
                            }
                        }) {
                            self.inner.http.context().traffic.increment_error();
                            error!(
                                target: "gate_runtime",
                                addr = %bound_addr,
                                error = %error,
                                "HTTPS listener client scheduling failed"
                            );
                        }
                    }
                    Err(source) => {
                        self.inner.http.context().traffic.increment_error();
                        warn!(
                            target: "gate_runtime",
                            addr = %bound_addr,
                            error = %source,
                            "HTTPS listener accept failed"
                        );
                        sleep(self.inner.http.context().config.listener.accept_backoff).await;
                    }
                }
            }
        }

        *self.inner.status.write() = HttpsListenerStatus::Stopped;
        debug!(
            target: "gate_runtime",
            addr = %bound_addr,
            "HTTPS listener stopped"
        );
    }

    async fn handle_tls_client(
        &self,
        stream: TcpStream,
        acceptor: TlsAcceptor,
        remote_addr: SocketAddr,
    ) -> Result<(), RuntimeError> {
        if self.inner.http.context().config.listener.tcp_nodelay {
            stream.set_nodelay(true).map_err(|source| {
                RuntimeError::Listener(ListenerError::Lifecycle {
                    reason: format!("failed to set TCP_NODELAY: {source}"),
                })
            })?;
        }

        let local_addr = stream
            .local_addr()
            .unwrap_or(self.inner.http.context().config.listener.listen_addr);
        let handshake_started = Instant::now();
        let tls_stream = acceptor.accept(stream).await.map_err(|source| {
            self.inner.metrics.record_handshake_error();
            HttpsError::Handshake {
                reason: source.to_string(),
            }
        })?;
        let handshake_ms = elapsed_millis(handshake_started);
        let tls_log = self.tls_log(&tls_stream, handshake_ms);
        self.inner
            .metrics
            .record_handshake(handshake_ms, tls_log.tls_version.as_deref());

        info!(
            target: "gate_runtime",
            remote_addr = %remote_addr,
            tls_version = ?tls_log.tls_version,
            cipher_suite = ?tls_log.cipher_suite,
            sni = ?tls_log.sni,
            certificate = ?tls_log.certificate_domain,
            handshake_time_ms = handshake_ms,
            "HTTPS Handshake"
        );

        self.inner
            .http
            .handle_stream(
                tls_stream,
                remote_addr,
                local_addr,
                HttpConnectionMetadata::https(tls_log, self.inner.host_resolver.clone()),
            )
            .await
    }

    fn tls_log(&self, stream: &TlsStream<TcpStream>, handshake_time_ms: u64) -> HttpTlsLog {
        let (_, connection) = stream.get_ref();
        let tls_version = connection.protocol_version().map(protocol_version_name);
        let cipher_suite = connection
            .negotiated_cipher_suite()
            .map(|suite| format!("{:?}", suite.suite()));
        let sni = connection.server_name().map(ToString::to_string);
        let certificate = self.inner.cache.material_for_name(sni.as_deref());

        HttpTlsLog {
            tls_version,
            cipher_suite,
            sni,
            certificate_domain: certificate
                .as_ref()
                .map(|material| material.record.domain.clone()),
            certificate_fingerprint: certificate
                .as_ref()
                .map(|material| material.record.fingerprint.sha256.clone()),
            handshake_time_ms,
        }
    }

    fn start_monitor_task(&self) -> Result<(), RuntimeError> {
        let context = self.inner.http.context();
        let monitor = self.inner.http.monitor();
        let interval = context
            .config
            .monitor_interval
            .max(Duration::from_millis(100));
        let tunnel_id = self.tunnel_id();

        self.inner.http.scheduler().spawn_monitor(
            format!("https-runtime-monitor:{tunnel_id}"),
            async move {
                let mut shutdown = context.subscribe_shutdown();
                loop {
                    tokio::select! {
                        _ = shutdown_notified(&mut shutdown) => break,
                        _ = sleep(interval) => {
                            let _ = monitor.snapshot();
                        }
                    }
                }
            },
        )?;

        Ok(())
    }

    fn start_cleanup_task(&self) -> Result<(), RuntimeError> {
        let context = self.inner.http.context();
        let interval = context
            .config
            .cleanup_interval
            .max(Duration::from_millis(100));
        let tunnel_id = self.tunnel_id();

        self.inner.http.scheduler().spawn_cleanup(
            format!("https-runtime-cleanup:{tunnel_id}"),
            async move {
                let mut shutdown = context.subscribe_shutdown();
                loop {
                    tokio::select! {
                        _ = shutdown_notified(&mut shutdown) => break,
                        _ = sleep(interval) => {
                            let _ = context.traffic.drain_current();
                        }
                    }
                }
            },
        )?;

        Ok(())
    }
}

impl RuntimeLifecycle for HttpsTunnelRuntime {
    fn state(&self) -> RuntimeState {
        self.inner.http.context().state.current()
    }

    fn start(&self) -> BoxFuture<'static, Result<(), RuntimeError>> {
        let runtime = self.clone();
        Box::pin(async move { HttpsTunnelRuntime::start(&runtime).await })
    }

    fn stop(&self) -> BoxFuture<'static, Result<(), RuntimeError>> {
        let runtime = self.clone();
        Box::pin(async move { HttpsTunnelRuntime::stop(&runtime).await })
    }

    fn restart(&self) -> BoxFuture<'static, Result<(), RuntimeError>> {
        let runtime = self.clone();
        Box::pin(async move { HttpsTunnelRuntime::restart(&runtime).await })
    }

    fn pause(&self) -> BoxFuture<'static, Result<(), RuntimeError>> {
        let runtime = self.clone();
        Box::pin(async move { HttpsTunnelRuntime::pause(&runtime).await })
    }

    fn resume(&self) -> BoxFuture<'static, Result<(), RuntimeError>> {
        let runtime = self.clone();
        Box::pin(async move { HttpsTunnelRuntime::resume(&runtime).await })
    }

    fn shutdown(&self) -> BoxFuture<'static, Result<(), RuntimeError>> {
        let runtime = self.clone();
        Box::pin(async move { HttpsTunnelRuntime::shutdown(&runtime).await })
    }
}

#[derive(Debug)]
struct CertificateEntry {
    material: CertificateMaterial,
    key: Arc<CertifiedKey>,
}

#[derive(Debug)]
struct CertificateCacheState {
    entries: HashMap<String, Arc<CertificateEntry>>,
    default_domain: Option<String>,
}

#[derive(Debug)]
struct TlsCertificateCache {
    state: RwLock<CertificateCacheState>,
}

impl TlsCertificateCache {
    fn new(default_domain: Option<String>) -> Self {
        Self {
            state: RwLock::new(CertificateCacheState {
                entries: HashMap::new(),
                default_domain: default_domain.map(|domain| normalize_host(&domain)),
            }),
        }
    }

    fn is_empty(&self) -> bool {
        self.state.read().entries.is_empty()
    }

    fn replace_all(&self, materials: Vec<CertificateMaterial>) -> Result<(), HttpsError> {
        let default_domain = self.state.read().default_domain.clone();
        let mut next = CertificateCacheState {
            entries: HashMap::new(),
            default_domain,
        };
        for material in materials {
            insert_material(&mut next.entries, material)?;
        }
        *self.state.write() = next;
        Ok(())
    }

    fn upsert(&self, material: CertificateMaterial) -> Result<(), HttpsError> {
        let mut state = self.state.write();
        insert_material(&mut state.entries, material)
    }

    fn resolve_entry(&self, server_name: Option<&str>) -> Option<Arc<CertificateEntry>> {
        let state = self.state.read();
        if let Some(name) = server_name {
            let name = normalize_host(name);
            if let Some(entry) = state.entries.get(&name) {
                return Some(Arc::clone(entry));
            }
            if let Some(wildcard) = wildcard_name(&name) {
                if let Some(entry) = state.entries.get(&wildcard) {
                    return Some(Arc::clone(entry));
                }
            }
        }

        state
            .default_domain
            .as_deref()
            .and_then(|domain| state.entries.get(domain))
            .cloned()
            .or_else(|| state.entries.values().next().cloned())
    }

    fn material_for_name(&self, server_name: Option<&str>) -> Option<CertificateMaterial> {
        self.resolve_entry(server_name)
            .map(|entry| entry.material.clone())
    }

    fn snapshots(&self) -> Vec<CertificateSnapshot> {
        let mut seen = HashSet::new();
        let mut snapshots = self
            .state
            .read()
            .entries
            .values()
            .filter_map(|entry| {
                let fingerprint = entry.material.record.fingerprint.sha256.clone();
                if !seen.insert(fingerprint.clone()) {
                    return None;
                }
                let record = &entry.material.record;
                Some(CertificateSnapshot {
                    domain: record.domain.clone(),
                    status: certificate_status_name(&record.status).to_string(),
                    expire_days: record
                        .expire_time
                        .signed_duration_since(chrono::Utc::now())
                        .num_days(),
                    issuer: record.issuer.clone(),
                    fingerprint_sha256: fingerprint,
                    san: record.san.clone(),
                })
            })
            .collect::<Vec<_>>();
        snapshots.sort_by(|left, right| left.domain.cmp(&right.domain));
        snapshots
    }
}

#[derive(Debug)]
struct SniCertificateResolver {
    cache: Arc<TlsCertificateCache>,
    missing_tx: mpsc::UnboundedSender<String>,
}

impl ResolvesServerCert for SniCertificateResolver {
    fn resolve(&self, client_hello: ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
        let server_name = client_hello.server_name().map(normalize_host);
        let entry = self.cache.resolve_entry(server_name.as_deref());
        if entry.is_none() {
            if let Some(server_name) = server_name {
                let _ = self.missing_tx.send(server_name);
            }
            return None;
        }
        entry.map(|entry| Arc::clone(&entry.key))
    }
}

fn build_server_config(
    tls: &HttpsTlsConfig,
    resolver: Arc<dyn ResolvesServerCert>,
) -> Result<ServerConfig, HttpsError> {
    let mut provider = ring::default_provider();
    if !tls.cipher_suites.is_empty() {
        let requested = tls
            .cipher_suites
            .iter()
            .map(|value| value.to_ascii_uppercase())
            .collect::<HashSet<_>>();
        provider.cipher_suites.retain(|suite| {
            requested.contains(&format!("{:?}", suite.suite()).to_ascii_uppercase())
        });
        if provider.cipher_suites.is_empty() {
            return Err(HttpsError::Tls {
                reason: "configured cipher suite list has no rustls-supported entries".to_string(),
            });
        }
    }

    let versions = supported_versions(tls)?;
    let builder = ServerConfig::builder_with_provider(Arc::new(provider))
        .with_protocol_versions(&versions)
        .map_err(|source| HttpsError::Tls {
            reason: source.to_string(),
        })?;
    let mut config = builder.with_no_client_auth().with_cert_resolver(resolver);
    config.alpn_protocols = vec![b"http/1.1".to_vec()];
    Ok(config)
}

fn supported_versions(
    tls: &HttpsTlsConfig,
) -> Result<Vec<&'static SupportedProtocolVersion>, HttpsError> {
    let mut versions = match tls.minimum_tls_version {
        TlsProtocolVersion::Tls12 => vec![&rustls::version::TLS13, &rustls::version::TLS12],
        TlsProtocolVersion::Tls13 => vec![&rustls::version::TLS13],
    };

    if let Some(preferred) = tls.preferred_tls_version {
        versions.sort_by_key(|version| {
            if version.version == tls_version_value(preferred) {
                0
            } else {
                1
            }
        });
    }

    if versions.is_empty() {
        return Err(HttpsError::Tls {
            reason: "at least one TLS protocol version must be enabled".to_string(),
        });
    }
    Ok(versions)
}

fn insert_material(
    entries: &mut HashMap<String, Arc<CertificateEntry>>,
    material: CertificateMaterial,
) -> Result<(), HttpsError> {
    let certified_key = parse_certified_key(&material)?;
    let entry = Arc::new(CertificateEntry {
        material,
        key: certified_key,
    });

    for name in certificate_names(&entry.material.record) {
        entries.insert(name, Arc::clone(&entry));
    }
    Ok(())
}

fn parse_certified_key(material: &CertificateMaterial) -> Result<Arc<CertifiedKey>, HttpsError> {
    let mut cert_reader = Cursor::new(material.certificate_pem.as_bytes());
    let cert_chain = rustls_pemfile::certs(&mut cert_reader)
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| HttpsError::Certificate {
            reason: source.to_string(),
        })?;
    if cert_chain.is_empty() {
        return Err(HttpsError::Certificate {
            reason: format!(
                "certificate chain for `{}` is empty",
                material.record.domain
            ),
        });
    }

    let mut key_reader = Cursor::new(material.private_key_pem.as_bytes());
    let private_key =
        rustls_pemfile::private_key(&mut key_reader).map_err(|source| HttpsError::Certificate {
            reason: source.to_string(),
        })?;
    let Some(private_key) = private_key else {
        return Err(HttpsError::Certificate {
            reason: format!("private key for `{}` is missing", material.record.domain),
        });
    };

    let signing_key =
        ring::sign::any_supported_type(&private_key).map_err(|source| HttpsError::Certificate {
            reason: source.to_string(),
        })?;
    Ok(Arc::new(CertifiedKey::new(cert_chain, signing_key)))
}

fn certificate_names(record: &CertificateRecord) -> Vec<String> {
    let mut names = Vec::new();
    names.push(normalize_host(&record.domain));
    for name in &record.san {
        let normalized = normalize_host(name);
        if !names.iter().any(|item| item == &normalized) {
            names.push(normalized);
        }
    }
    names
}

fn certificate_matches(record: &CertificateRecord, domain: &str) -> bool {
    let domain = normalize_host(domain);
    certificate_names(record).iter().any(|name| {
        name == &domain
            || name
                .strip_prefix("*.")
                .map(|suffix| domain.ends_with(&format!(".{suffix}")))
                .unwrap_or(false)
    })
}

fn normalize_host(host: &str) -> String {
    host.trim().trim_end_matches('.').to_ascii_lowercase()
}

fn wildcard_name(host: &str) -> Option<String> {
    let (_, suffix) = host.split_once('.')?;
    if suffix.is_empty() {
        None
    } else {
        Some(format!("*.{suffix}"))
    }
}

fn protocol_version_name(version: ProtocolVersion) -> String {
    match version {
        ProtocolVersion::TLSv1_2 => "TLSv1.2".to_string(),
        ProtocolVersion::TLSv1_3 => "TLSv1.3".to_string(),
        other => format!("{other:?}"),
    }
}

fn tls_version_value(version: TlsProtocolVersion) -> ProtocolVersion {
    match version {
        TlsProtocolVersion::Tls12 => ProtocolVersion::TLSv1_2,
        TlsProtocolVersion::Tls13 => ProtocolVersion::TLSv1_3,
    }
}

fn certificate_status_name(status: &CertificateStatus) -> &'static str {
    match status {
        CertificateStatus::Pending => "pending",
        CertificateStatus::Active => "active",
        CertificateStatus::ExpiringSoon => "expiring_soon",
        CertificateStatus::Expired => "expired",
        CertificateStatus::Revoked => "revoked",
        CertificateStatus::Deleted => "deleted",
        CertificateStatus::Failed => "failed",
    }
}

fn elapsed_millis(started: Instant) -> u64 {
    started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
}

fn average(total: u128, count: u64) -> f64 {
    if count == 0 {
        0.0
    } else {
        total as f64 / count as f64
    }
}

async fn shutdown_notified(shutdown: &mut watch::Receiver<bool>) {
    if *shutdown.borrow() {
        return;
    }

    while shutdown.changed().await.is_ok() {
        if *shutdown.borrow() {
            return;
        }
    }
}

impl HttpsError {
    fn certificate_source(source: impl std::error::Error) -> Self {
        Self::Certificate {
            reason: source.to_string(),
        }
    }

    fn tls_provider(source: impl std::error::Error) -> Self {
        Self::Certificate {
            reason: source.to_string(),
        }
    }
}
