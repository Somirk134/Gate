//! HTTP/1.1 tunnel runtime built on the TCP runtime primitives.
//!
//! The HTTP runtime keeps the same lifecycle, scheduler, connector, session,
//! and traffic-statistics model as the TCP runtime. It adds request-level
//! routing, configurable header mutation, and HTTP access logs.

use crate::connection::ConnectionId;
use crate::core::TunnelId;
use crate::runtime::config::{
    BufferConfig, ConnectorConfig, ListenerConfig, RetryConfig, RuntimeConfig, TimeoutConfig,
};
use crate::runtime::connector::TcpConnector;
use crate::runtime::context::RuntimeContext;
use crate::runtime::error::{ForwardError, ListenerError, RuntimeError};
use crate::runtime::lifecycle::RuntimeLifecycle;
use crate::runtime::monitor::{RuntimeMetrics, RuntimeMonitor};
use crate::runtime::scheduler::RuntimeScheduler;
use crate::runtime::state::{RuntimeState, SessionState};
use crate::runtime::worker::TaskId;
use futures::future::BoxFuture;
use parking_lot::{Mutex, RwLock};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::io::{Error, ErrorKind};
use std::net::SocketAddr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener as TokioTcpListener, TcpStream};
use tokio::sync::watch;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

const DEFAULT_MAX_HEADER_BYTES: usize = 64 * 1024;
const DEFAULT_RECENT_LOG_LIMIT: usize = 512;
const MAX_CHUNK_LINE_BYTES: usize = 8 * 1024;

/// Header set operation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpHeaderRule {
    pub name: String,
    pub value: String,
}

impl HttpHeaderRule {
    pub fn new(name: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            value: value.into(),
        }
    }
}

/// Per-route request/response header mutation settings.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpHeaderConfig {
    pub request_set: Vec<HttpHeaderRule>,
    pub request_remove: Vec<String>,
    pub response_set: Vec<HttpHeaderRule>,
    pub response_remove: Vec<String>,
    pub x_forwarded_for: bool,
    pub x_real_ip: bool,
    pub x_forwarded_proto: bool,
    pub forwarded_proto: String,
}

impl Default for HttpHeaderConfig {
    fn default() -> Self {
        Self {
            request_set: Vec::new(),
            request_remove: Vec::new(),
            response_set: Vec::new(),
            response_remove: Vec::new(),
            x_forwarded_for: true,
            x_real_ip: true,
            x_forwarded_proto: true,
            forwarded_proto: "http".to_string(),
        }
    }
}

/// One host/path route to a local HTTP service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRouteConfig {
    pub tunnel_id: TunnelId,
    pub name: String,
    pub host: Option<String>,
    pub path_prefix: Option<String>,
    pub target_addr: SocketAddr,
    pub headers: HttpHeaderConfig,
    pub rewrite: Option<String>,
    pub enabled: bool,
}

impl HttpRouteConfig {
    pub fn new(name: impl Into<String>, target_addr: SocketAddr) -> Self {
        Self {
            tunnel_id: TunnelId::new(),
            name: name.into(),
            host: None,
            path_prefix: Some("/".to_string()),
            target_addr,
            headers: HttpHeaderConfig::default(),
            rewrite: None,
            enabled: true,
        }
    }

    pub fn tunnel_id(mut self, tunnel_id: TunnelId) -> Self {
        self.tunnel_id = tunnel_id;
        self
    }

    pub fn host(mut self, host: impl Into<String>) -> Self {
        self.host = Some(normalize_host_config(&host.into()));
        self
    }

    pub fn path_prefix(mut self, path_prefix: impl Into<String>) -> Self {
        self.path_prefix = Some(normalize_path_prefix(&path_prefix.into()));
        self
    }

    pub fn headers(mut self, headers: HttpHeaderConfig) -> Self {
        self.headers = headers;
        self
    }
}

/// HTTP runtime configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRuntimeConfig {
    pub base: RuntimeConfig,
    pub routes: Vec<HttpRouteConfig>,
    pub max_header_bytes: usize,
    pub recent_log_limit: usize,
    pub upgrade_reserved: bool,
}

impl HttpRuntimeConfig {
    pub fn builder() -> HttpRuntimeConfigBuilder {
        HttpRuntimeConfigBuilder::default()
    }
}

impl Default for HttpRuntimeConfig {
    fn default() -> Self {
        Self {
            base: RuntimeConfig::default(),
            routes: Vec::new(),
            max_header_bytes: DEFAULT_MAX_HEADER_BYTES,
            recent_log_limit: DEFAULT_RECENT_LOG_LIMIT,
            upgrade_reserved: true,
        }
    }
}

impl From<RuntimeConfig> for HttpRuntimeConfig {
    fn from(base: RuntimeConfig) -> Self {
        Self {
            base,
            ..Self::default()
        }
    }
}

/// Builder for [`HttpRuntimeConfig`].
#[derive(Debug, Clone)]
pub struct HttpRuntimeConfigBuilder {
    config: HttpRuntimeConfig,
}

impl Default for HttpRuntimeConfigBuilder {
    fn default() -> Self {
        Self {
            config: HttpRuntimeConfig::default(),
        }
    }
}

impl HttpRuntimeConfigBuilder {
    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.base.name = name.into();
        self
    }

    pub fn tunnel_id(mut self, tunnel_id: TunnelId) -> Self {
        self.config.base.listener.tunnel_id = tunnel_id;
        self
    }

    pub fn listen_addr(mut self, listen_addr: SocketAddr) -> Self {
        self.config.base.listener.listen_addr = listen_addr;
        self
    }

    pub fn listener(mut self, listener: ListenerConfig) -> Self {
        self.config.base.listener = listener;
        self
    }

    pub fn buffer(mut self, buffer: BufferConfig) -> Self {
        self.config.base.buffer = buffer;
        self
    }

    pub fn retry(mut self, retry: RetryConfig) -> Self {
        self.config.base.retry = retry;
        self
    }

    pub fn timeout(mut self, timeout: TimeoutConfig) -> Self {
        self.config.base.timeout = timeout;
        self
    }

    pub fn max_tasks(mut self, max_tasks: usize) -> Self {
        self.config.base.max_tasks = max_tasks;
        self
    }

    pub fn max_sessions(mut self, max_sessions: usize) -> Self {
        self.config.base.max_sessions = max_sessions;
        self
    }

    pub fn route(mut self, route: HttpRouteConfig) -> Self {
        self.config.routes.push(route);
        self
    }

    pub fn routes(mut self, routes: Vec<HttpRouteConfig>) -> Self {
        self.config.routes = routes;
        self
    }

    pub fn max_header_bytes(mut self, max_header_bytes: usize) -> Self {
        self.config.max_header_bytes = max_header_bytes;
        self
    }

    pub fn recent_log_limit(mut self, recent_log_limit: usize) -> Self {
        self.config.recent_log_limit = recent_log_limit;
        self
    }

    pub fn upgrade_reserved(mut self, upgrade_reserved: bool) -> Self {
        self.config.upgrade_reserved = upgrade_reserved;
        self
    }

    pub fn build(self) -> HttpRuntimeConfig {
        self.config
    }
}

/// One HTTP access-log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRequestLog {
    pub id: String,
    pub tunnel_id: Option<String>,
    pub route_name: Option<String>,
    pub method: String,
    pub url: String,
    pub path: String,
    pub host: String,
    pub status: u16,
    pub latency_ms: u64,
    pub client_ip: String,
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub timestamp_millis: i64,
    pub http_version: Option<String>,
    pub tls: Option<HttpTlsLog>,
}

/// TLS metadata attached to HTTPS request logs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HttpTlsLog {
    pub tls_version: Option<String>,
    pub cipher_suite: Option<String>,
    pub sni: Option<String>,
    pub certificate_domain: Option<String>,
    pub certificate_fingerprint: Option<String>,
    pub handshake_time_ms: u64,
}

/// Host resolver adapter used by HTTPS runtime integrations.
///
/// The concrete domain-management module can implement or wrap this trait
/// without making the HTTP data plane parse domain records itself.
pub trait HttpHostResolver: Send + Sync {
    fn resolve_host(&self, host: &str) -> Result<String, String>;
}

/// Connection-level metadata supplied by plain HTTP or HTTPS listeners.
#[derive(Clone)]
pub(crate) struct HttpConnectionMetadata {
    forwarded_proto: String,
    tls: Option<HttpTlsLog>,
    host_resolver: Option<Arc<dyn HttpHostResolver>>,
}

impl HttpConnectionMetadata {
    pub(crate) fn http() -> Self {
        Self {
            forwarded_proto: "http".to_string(),
            tls: None,
            host_resolver: None,
        }
    }

    pub(crate) fn https(tls: HttpTlsLog, host_resolver: Option<Arc<dyn HttpHostResolver>>) -> Self {
        Self {
            forwarded_proto: "https".to_string(),
            tls: Some(tls),
            host_resolver,
        }
    }
}

impl Default for HttpConnectionMetadata {
    fn default() -> Self {
        Self::http()
    }
}

/// Aggregated metrics for one HTTP route.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HttpRouteMetrics {
    pub tunnel_id: String,
    pub name: String,
    pub host: Option<String>,
    pub path_prefix: Option<String>,
    pub requests_total: u64,
    pub request_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub status_codes: HashMap<u16, u64>,
    pub latency: HttpLatencyMetrics,
    pub bandwidth: HttpBandwidthMetrics,
    pub success_rate: f64,
    pub average_latency_ms: f64,
    pub recent_requests: Vec<HttpRequestLog>,
}

/// Runtime-level HTTP metrics for dashboard views.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HttpRuntimeMetrics {
    pub requests_total: u64,
    pub active_requests: u64,
    pub request_count: u64,
    pub success_count: u64,
    pub failure_count: u64,
    pub status_codes: HashMap<u16, u64>,
    pub latency: HttpLatencyMetrics,
    pub bandwidth: HttpBandwidthMetrics,
    pub success_rate: f64,
    pub average_latency_ms: f64,
    pub recent_requests: Vec<HttpRequestLog>,
    pub routes: Vec<HttpRouteMetrics>,
}

/// HTTP 延迟指标，保留总量便于外部计算窗口平均值。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HttpLatencyMetrics {
    pub total_ms: u64,
    pub average_ms: f64,
}

/// HTTP 带宽指标，按请求日志真实累计上传和下载字节。
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HttpBandwidthMetrics {
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub total_bytes: u64,
}

#[derive(Debug, Default, Clone)]
struct RouteAggregate {
    tunnel_id: String,
    name: String,
    host: Option<String>,
    path_prefix: Option<String>,
    request_count: u64,
    success_count: u64,
    failure_count: u64,
    total_latency_ms: u128,
    upload_bytes: u64,
    download_bytes: u64,
    status_codes: HashMap<u16, u64>,
}

#[derive(Debug)]
struct HttpLogState {
    recent_limit: usize,
    logs: VecDeque<HttpRequestLog>,
    routes: HashMap<String, RouteAggregate>,
    request_count: u64,
    success_count: u64,
    failure_count: u64,
    total_latency_ms: u128,
    upload_bytes: u64,
    download_bytes: u64,
    status_codes: HashMap<u16, u64>,
}

impl HttpLogState {
    fn new(recent_limit: usize) -> Self {
        Self {
            recent_limit: recent_limit.max(1),
            logs: VecDeque::new(),
            routes: HashMap::new(),
            request_count: 0,
            success_count: 0,
            failure_count: 0,
            total_latency_ms: 0,
            upload_bytes: 0,
            download_bytes: 0,
            status_codes: HashMap::new(),
        }
    }
}

/// Thread-safe HTTP log and aggregate store.
#[derive(Debug)]
pub struct HttpLogStore {
    state: Mutex<HttpLogState>,
    active_requests: AtomicU64,
}

impl HttpLogStore {
    pub fn new(recent_limit: usize) -> Self {
        Self {
            state: Mutex::new(HttpLogState::new(recent_limit)),
            active_requests: AtomicU64::new(0),
        }
    }

    fn begin_request(&self) {
        self.active_requests.fetch_add(1, Ordering::Relaxed);
    }

    fn end_request(&self) {
        let _ = self
            .active_requests
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |value| {
                value.checked_sub(1)
            });
    }

    pub fn record(&self, route: Option<&HttpRouteConfig>, log: HttpRequestLog) {
        let mut state = self.state.lock();
        let success = (200..400).contains(&log.status);
        state.request_count = state.request_count.saturating_add(1);
        if success {
            state.success_count = state.success_count.saturating_add(1);
        } else {
            state.failure_count = state.failure_count.saturating_add(1);
        }
        state.total_latency_ms = state
            .total_latency_ms
            .saturating_add(u128::from(log.latency_ms));
        state.upload_bytes = state.upload_bytes.saturating_add(log.upload_bytes);
        state.download_bytes = state.download_bytes.saturating_add(log.download_bytes);
        *state.status_codes.entry(log.status).or_insert(0) += 1;

        if let Some(route) = route {
            let key = route.tunnel_id.to_string();
            let aggregate = state
                .routes
                .entry(key.clone())
                .or_insert_with(|| RouteAggregate {
                    tunnel_id: key,
                    name: route.name.clone(),
                    host: route.host.clone(),
                    path_prefix: route.path_prefix.clone(),
                    ..RouteAggregate::default()
                });
            aggregate.request_count = aggregate.request_count.saturating_add(1);
            if success {
                aggregate.success_count = aggregate.success_count.saturating_add(1);
            } else {
                aggregate.failure_count = aggregate.failure_count.saturating_add(1);
            }
            aggregate.total_latency_ms = aggregate
                .total_latency_ms
                .saturating_add(u128::from(log.latency_ms));
            aggregate.upload_bytes = aggregate.upload_bytes.saturating_add(log.upload_bytes);
            aggregate.download_bytes = aggregate.download_bytes.saturating_add(log.download_bytes);
            *aggregate.status_codes.entry(log.status).or_insert(0) += 1;
        }

        state.logs.push_back(log);
        while state.logs.len() > state.recent_limit {
            let _ = state.logs.pop_front();
        }
    }

    pub fn snapshot(&self) -> HttpRuntimeMetrics {
        let state = self.state.lock();
        let recent_requests = state.logs.iter().cloned().collect::<Vec<_>>();
        let active_requests = self.active_requests.load(Ordering::Relaxed);
        let routes = state
            .routes
            .values()
            .map(|aggregate| {
                let route_recent = recent_requests
                    .iter()
                    .filter(|log| log.tunnel_id.as_deref() == Some(aggregate.tunnel_id.as_str()))
                    .cloned()
                    .collect::<Vec<_>>();
                HttpRouteMetrics {
                    tunnel_id: aggregate.tunnel_id.clone(),
                    name: aggregate.name.clone(),
                    host: aggregate.host.clone(),
                    path_prefix: aggregate.path_prefix.clone(),
                    requests_total: aggregate.request_count,
                    request_count: aggregate.request_count,
                    success_count: aggregate.success_count,
                    failure_count: aggregate.failure_count,
                    status_codes: aggregate.status_codes.clone(),
                    latency: HttpLatencyMetrics {
                        total_ms: saturating_u128_to_u64(aggregate.total_latency_ms),
                        average_ms: average(aggregate.total_latency_ms, aggregate.request_count),
                    },
                    bandwidth: HttpBandwidthMetrics {
                        upload_bytes: aggregate.upload_bytes,
                        download_bytes: aggregate.download_bytes,
                        total_bytes: aggregate
                            .upload_bytes
                            .saturating_add(aggregate.download_bytes),
                    },
                    success_rate: ratio(aggregate.success_count, aggregate.request_count),
                    average_latency_ms: average(
                        aggregate.total_latency_ms,
                        aggregate.request_count,
                    ),
                    recent_requests: route_recent,
                }
            })
            .collect::<Vec<_>>();

        HttpRuntimeMetrics {
            requests_total: state.request_count,
            active_requests,
            request_count: state.request_count,
            success_count: state.success_count,
            failure_count: state.failure_count,
            status_codes: state.status_codes.clone(),
            latency: HttpLatencyMetrics {
                total_ms: saturating_u128_to_u64(state.total_latency_ms),
                average_ms: average(state.total_latency_ms, state.request_count),
            },
            bandwidth: HttpBandwidthMetrics {
                upload_bytes: state.upload_bytes,
                download_bytes: state.download_bytes,
                total_bytes: state.upload_bytes.saturating_add(state.download_bytes),
            },
            success_rate: ratio(state.success_count, state.request_count),
            average_latency_ms: average(state.total_latency_ms, state.request_count),
            recent_requests,
            routes,
        }
    }
}

struct ActiveHttpRequest {
    logs: Arc<HttpLogStore>,
}

impl ActiveHttpRequest {
    fn new(logs: Arc<HttpLogStore>) -> Self {
        logs.begin_request();
        Self { logs }
    }
}

impl Drop for ActiveHttpRequest {
    fn drop(&mut self) {
        self.logs.end_request();
    }
}

#[derive(Debug)]
struct HttpTunnelRuntimeInner {
    context: Arc<RuntimeContext>,
    scheduler: Arc<RuntimeScheduler>,
    monitor: Arc<RuntimeMonitor>,
    routes: RwLock<Vec<HttpRouteConfig>>,
    logs: Arc<HttpLogStore>,
    max_header_bytes: usize,
    upgrade_reserved: bool,
    status: RwLock<HttpListenerStatus>,
    task_id: RwLock<Option<TaskId>>,
    bound_addr: RwLock<Option<SocketAddr>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HttpListenerStatus {
    Created,
    Listening,
    Stopped,
}

/// HTTP tunnel runtime entry point.
#[derive(Debug, Clone)]
pub struct HttpTunnelRuntime {
    inner: Arc<HttpTunnelRuntimeInner>,
}

/// Public HTTP tunnel type alias for management-layer naming.
pub type HttpTunnel = HttpTunnelRuntime;

impl HttpTunnelRuntime {
    pub fn builder() -> HttpRuntimeBuilder {
        HttpRuntimeBuilder::default()
    }

    pub fn new(config: HttpRuntimeConfig) -> Self {
        let context = Arc::new(RuntimeContext::new(config.base.clone()));
        let scheduler = Arc::new(RuntimeScheduler::new(config.base.max_tasks));
        let monitor = Arc::new(RuntimeMonitor::new(
            Arc::clone(&context.traffic),
            Arc::clone(&context.sessions),
        ));
        let logs = Arc::new(HttpLogStore::new(config.recent_log_limit));

        Self {
            inner: Arc::new(HttpTunnelRuntimeInner {
                context,
                scheduler,
                monitor,
                routes: RwLock::new(config.routes),
                logs,
                max_header_bytes: config.max_header_bytes.max(1024),
                upgrade_reserved: config.upgrade_reserved,
                status: RwLock::new(HttpListenerStatus::Created),
                task_id: RwLock::new(None),
                bound_addr: RwLock::new(None),
            }),
        }
    }

    pub fn context(&self) -> Arc<RuntimeContext> {
        Arc::clone(&self.inner.context)
    }

    pub fn scheduler(&self) -> Arc<RuntimeScheduler> {
        Arc::clone(&self.inner.scheduler)
    }

    pub fn monitor(&self) -> Arc<RuntimeMonitor> {
        Arc::clone(&self.inner.monitor)
    }

    pub fn metrics(&self) -> RuntimeMetrics {
        self.inner.monitor.snapshot()
    }

    pub fn http_metrics(&self) -> HttpRuntimeMetrics {
        self.inner.logs.snapshot()
    }

    pub fn routes(&self) -> Vec<HttpRouteConfig> {
        self.inner.routes.read().clone()
    }

    pub fn replace_routes(&self, routes: Vec<HttpRouteConfig>) {
        *self.inner.routes.write() = routes;
    }

    pub fn upsert_route(&self, route: HttpRouteConfig) {
        let mut routes = self.inner.routes.write();
        if let Some(existing) = routes
            .iter_mut()
            .find(|item| item.tunnel_id == route.tunnel_id)
        {
            *existing = route;
        } else {
            routes.push(route);
        }
    }

    pub fn remove_route(&self, tunnel_id: TunnelId) -> Option<HttpRouteConfig> {
        let mut routes = self.inner.routes.write();
        let index = routes
            .iter()
            .position(|route| route.tunnel_id == tunnel_id)?;
        Some(routes.remove(index))
    }

    pub fn bound_addr(&self) -> Option<SocketAddr> {
        *self.inner.bound_addr.read()
    }

    pub fn tunnel_id(&self) -> TunnelId {
        self.inner.context.config.listener.tunnel_id
    }

    pub async fn start(&self) -> Result<(), RuntimeError> {
        match self.inner.context.state.current() {
            RuntimeState::Initializing | RuntimeState::Starting | RuntimeState::Running => {
                return Ok(())
            }
            RuntimeState::Paused => return self.resume().await,
            RuntimeState::Closed => {
                return Err(RuntimeError::InvalidStateTransition {
                    from: self.inner.context.state.current(),
                    to: RuntimeState::Starting,
                });
            }
            RuntimeState::Failed => {
                self.inner
                    .context
                    .state
                    .transition_to(RuntimeState::Restarting)?;
            }
            _ => {}
        }

        self.inner
            .context
            .state
            .transition_to(RuntimeState::Initializing)?;
        self.inner.context.reset_shutdown();
        let result = async {
            self.inner
                .context
                .state
                .transition_to(RuntimeState::Starting)?;
            self.start_listener().await?;
            self.start_monitor_task()?;
            self.start_cleanup_task()?;
            self.inner
                .context
                .state
                .transition_to(RuntimeState::Running)?;
            Ok::<(), RuntimeError>(())
        }
        .await;

        if let Err(error) = result {
            let _ = self.inner.context.state.transition_to(RuntimeState::Failed);
            return Err(error);
        }

        info!(
            target: "gate_runtime",
            tunnel_id = %self.tunnel_id(),
            name = %self.inner.context.config.name,
            "HTTP Runtime Start"
        );
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), RuntimeError> {
        match self.inner.context.state.current() {
            RuntimeState::Stopped | RuntimeState::Closed => return Ok(()),
            _ => {}
        }

        self.inner
            .context
            .state
            .transition_to(RuntimeState::Stopping)?;
        self.inner.context.request_shutdown();
        self.stop_listener().await?;
        self.inner
            .scheduler
            .graceful_shutdown(self.inner.context.config.timeout.shutdown_timeout)
            .await?;
        self.inner.context.sessions.close_all(SessionState::Closed);
        self.inner
            .context
            .state
            .transition_to(RuntimeState::Stopped)?;
        Ok(())
    }

    pub async fn restart(&self) -> Result<(), RuntimeError> {
        match self.inner.context.state.current() {
            RuntimeState::Running | RuntimeState::Paused => {
                self.inner
                    .context
                    .state
                    .transition_to(RuntimeState::Restarting)?;
                self.stop().await?;
            }
            RuntimeState::Created | RuntimeState::Stopped => {}
            RuntimeState::Failed => {
                self.inner
                    .context
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
        if self.inner.context.state.current() == RuntimeState::Paused {
            return Ok(());
        }

        self.inner
            .context
            .state
            .transition_to(RuntimeState::Paused)?;
        info!(
            target: "gate_runtime",
            tunnel_id = %self.tunnel_id(),
            "HTTP runtime paused"
        );
        Ok(())
    }

    pub async fn resume(&self) -> Result<(), RuntimeError> {
        if self.inner.context.state.current() == RuntimeState::Running {
            return Ok(());
        }

        self.inner
            .context
            .state
            .transition_to(RuntimeState::Running)?;
        info!(
            target: "gate_runtime",
            tunnel_id = %self.tunnel_id(),
            "HTTP runtime resumed"
        );
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), RuntimeError> {
        if self.inner.context.state.current() == RuntimeState::Closed {
            return Ok(());
        }

        self.inner
            .context
            .state
            .transition_to(RuntimeState::Stopping)?;
        self.inner.context.request_shutdown();
        self.stop_listener().await?;
        self.inner
            .scheduler
            .graceful_shutdown(self.inner.context.config.timeout.shutdown_timeout)
            .await?;
        self.inner.context.sessions.close_all(SessionState::Closed);
        self.inner
            .context
            .state
            .transition_to(RuntimeState::Closed)?;
        Ok(())
    }

    async fn start_listener(&self) -> Result<(), ListenerError> {
        if self.inner.task_id.read().is_some() {
            return Ok(());
        }

        let listen_addr = self.inner.context.config.listener.listen_addr;
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

        *self.inner.bound_addr.write() = Some(bound_addr);
        *self.inner.status.write() = HttpListenerStatus::Listening;

        let runtime = self.clone();
        let task_id = self.inner.scheduler.spawn_listener(
            format!("http-listener:{bound_addr}"),
            async move {
                runtime.accept_loop(listener, bound_addr).await;
            },
        )?;
        *self.inner.task_id.write() = Some(task_id);

        info!(
            target: "gate_runtime",
            addr = %bound_addr,
            "HTTP Listener Start"
        );

        Ok(())
    }

    async fn stop_listener(&self) -> Result<(), ListenerError> {
        if let Some(task_id) = self.inner.task_id.write().take() {
            let _ = self.inner.scheduler.cancel(task_id);
        }
        *self.inner.status.write() = HttpListenerStatus::Stopped;
        Ok(())
    }

    async fn accept_loop(self, listener: TokioTcpListener, bound_addr: SocketAddr) {
        let mut shutdown = self.inner.context.subscribe_shutdown();

        loop {
            if *shutdown.borrow() {
                break;
            }

            if self.inner.context.state.current() == RuntimeState::Paused {
                sleep(self.inner.context.config.listener.accept_backoff).await;
                continue;
            }

            tokio::select! {
                _ = shutdown_notified(&mut shutdown) => break,
                accepted = listener.accept() => match accepted {
                    Ok((stream, remote_addr)) => {
                        if let Err(error) = self.handle_client(stream, remote_addr).await {
                            self.inner.context.traffic.increment_error();
                            error!(
                                target: "gate_runtime",
                                addr = %bound_addr,
                                error = %error,
                                "HTTP listener client handling failed"
                            );
                        }
                    }
                    Err(source) => {
                        self.inner.context.traffic.increment_error();
                        warn!(
                            target: "gate_runtime",
                            addr = %bound_addr,
                            error = %source,
                            "HTTP listener accept failed"
                        );
                        sleep(self.inner.context.config.listener.accept_backoff).await;
                    }
                }
            }
        }

        *self.inner.status.write() = HttpListenerStatus::Stopped;
        debug!(
            target: "gate_runtime",
            addr = %bound_addr,
            "HTTP listener stopped"
        );
    }

    async fn handle_client(
        &self,
        client: TcpStream,
        remote_addr: SocketAddr,
    ) -> Result<(), RuntimeError> {
        if self.inner.context.config.listener.tcp_nodelay {
            client.set_nodelay(true).map_err(|source| {
                RuntimeError::Listener(ListenerError::Lifecycle {
                    reason: format!("failed to set TCP_NODELAY: {source}"),
                })
            })?;
        }

        let local_addr = client
            .local_addr()
            .unwrap_or(self.inner.context.config.listener.listen_addr);
        self.handle_stream(
            client,
            remote_addr,
            local_addr,
            HttpConnectionMetadata::http(),
        )
        .await
    }

    pub(crate) async fn handle_stream<S>(
        &self,
        client: S,
        remote_addr: SocketAddr,
        local_addr: SocketAddr,
        metadata: HttpConnectionMetadata,
    ) -> Result<(), RuntimeError>
    where
        S: AsyncRead + AsyncWrite + Unpin + Send + 'static,
    {
        if self.inner.context.state.current() == RuntimeState::Paused {
            return Err(RuntimeError::Paused);
        }

        let max_sessions = self
            .inner
            .context
            .config
            .max_sessions
            .min(self.inner.context.config.listener.max_connections);
        if self.inner.context.sessions.active_count() >= max_sessions {
            return Err(RuntimeError::InvalidConfig {
                reason: format!("max session limit reached: {max_sessions}"),
            });
        }

        let session = self.inner.context.sessions.create(
            self.inner.context.config.listener.tunnel_id,
            ConnectionId::new(),
            remote_addr,
            local_addr,
        );

        info!(
            target: "gate_runtime",
            session_id = %session.id,
            tunnel_id = %session.tunnel_id,
            remote_addr = %remote_addr,
            local_addr = %local_addr,
            "HTTP Session Create"
        );

        let runtime = self.clone();
        let sessions = Arc::clone(&self.inner.context.sessions);
        let traffic = Arc::clone(&self.inner.context.traffic);
        let session_id = session.id;

        self.inner
            .scheduler
            .spawn_forward(format!("http-forward:{session_id}"), async move {
                session.set_status(SessionState::Forwarding);
                let result = runtime
                    .serve_connection(client, remote_addr, metadata)
                    .await;

                match result {
                    Ok(()) => {
                        session.set_status(SessionState::Closing);
                        let _ = sessions.close(&session_id, SessionState::Closed);
                        info!(
                            target: "gate_runtime",
                            session_id = %session_id,
                            "HTTP Session Close"
                        );
                    }
                    Err(RuntimeError::Forward(ForwardError::Shutdown)) => {
                        session.set_status(SessionState::Closing);
                        let _ = sessions.close(&session_id, SessionState::Closed);
                    }
                    Err(error) => {
                        traffic.increment_error();
                        let _ = sessions.close(&session_id, SessionState::Failed);
                        error!(
                            target: "gate_runtime",
                            session_id = %session_id,
                            error = %error,
                            "HTTP Session Close"
                        );
                    }
                }
            })?;

        Ok(())
    }

    async fn serve_connection<S>(
        &self,
        mut client: S,
        remote_addr: SocketAddr,
        metadata: HttpConnectionMetadata,
    ) -> Result<(), RuntimeError>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let mut shutdown = self.inner.context.subscribe_shutdown();

        loop {
            if *shutdown.borrow() {
                return Err(ForwardError::Shutdown.into());
            }

            let head_bytes = tokio::select! {
                head = read_head(&mut client, self.inner.max_header_bytes) => head.map_err(forward_io)?,
                _ = shutdown_notified(&mut shutdown) => return Err(ForwardError::Shutdown.into()),
            };
            let Some(head_bytes) = head_bytes else {
                return Ok(());
            };

            let started = Instant::now();
            let _active_request = ActiveHttpRequest::new(Arc::clone(&self.inner.logs));
            let request = match HttpHead::parse_request(&head_bytes) {
                Ok(request) => request,
                Err(_) => {
                    let log = self
                        .send_error_response(
                            &mut client,
                            None,
                            "BAD",
                            "",
                            "",
                            remote_addr,
                            400,
                            "Bad Request",
                            started,
                            None,
                            &metadata,
                        )
                        .await?;
                    self.inner.logs.record(None, log);
                    return Ok(());
                }
            };

            if self.inner.upgrade_reserved && request.is_upgrade() {
                let log = self
                    .send_error_response(
                        &mut client,
                        None,
                        &request.method,
                        &request.target,
                        &request.host,
                        remote_addr,
                        501,
                        "Connection Upgrade is reserved",
                        started,
                        Some(&request.version),
                        &metadata,
                    )
                    .await?;
                self.inner.logs.record(None, log);
                return Ok(());
            }

            let route =
                match self.select_route_with_metadata(&request.host, &request.path, &metadata) {
                    Ok(route) => route,
                    Err(error) => {
                        warn!(
                            target: "gate_runtime",
                            host = %request.host,
                            error = %error,
                            "HTTP host resolver failed"
                        );
                        None
                    }
                };

            let Some(route) = route else {
                let log = self
                    .send_error_response(
                        &mut client,
                        None,
                        &request.method,
                        &request.target,
                        &request.host,
                        remote_addr,
                        404,
                        "No HTTP tunnel route matched",
                        started,
                        Some(&request.version),
                        &metadata,
                    )
                    .await?;
                self.inner.logs.record(None, log);
                if !request.keep_alive() {
                    return Ok(());
                }
                continue;
            };

            let connector = TcpConnector::new(
                ConnectorConfig {
                    target_addr: route.target_addr,
                    tcp_nodelay: self.inner.context.config.connector.tcp_nodelay,
                    keepalive: self.inner.context.config.connector.keepalive,
                },
                self.inner.context.config.retry.clone(),
                self.inner.context.config.timeout.clone(),
                Arc::clone(&self.inner.context.traffic),
            );

            let mut target = connector.connect().await?;
            let request_head = build_request_head(
                &request,
                &route,
                remote_addr,
                Some(&metadata.forwarded_proto),
            );
            target.write_all(&request_head).await.map_err(forward_io)?;
            self.inner
                .context
                .traffic
                .record_upload(request_head.len() as u64);
            let mut upload_bytes = request_head.len() as u64;
            upload_bytes = upload_bytes.saturating_add(
                copy_body(
                    &mut client,
                    &mut target,
                    request_body_kind(&request),
                    Direction::Upload,
                    Arc::clone(&self.inner.context.traffic),
                )
                .await?,
            );
            target.flush().await.map_err(forward_io)?;

            let response_head_bytes = read_head(&mut target, self.inner.max_header_bytes)
                .await
                .map_err(forward_io)?
                .ok_or_else(|| {
                    RuntimeError::Forward(ForwardError::Io {
                        source: Error::new(
                            ErrorKind::UnexpectedEof,
                            "target closed before response",
                        ),
                    })
                })?;
            let response = HttpHead::parse_response(&response_head_bytes).map_err(|source| {
                RuntimeError::Forward(ForwardError::Io {
                    source: Error::new(ErrorKind::InvalidData, source),
                })
            })?;
            let response_kind = response_body_kind(&request.method, response.status, &response);
            let close_after_response = matches!(response_kind, BodyKind::UntilEof);
            let keep_client_alive = request.keep_alive() && !close_after_response;
            let response_head = build_response_head(&response, &route, keep_client_alive);
            client.write_all(&response_head).await.map_err(forward_io)?;
            self.inner
                .context
                .traffic
                .record_download(response_head.len() as u64);
            let mut download_bytes = response_head.len() as u64;
            download_bytes = download_bytes.saturating_add(
                copy_body(
                    &mut target,
                    &mut client,
                    response_kind,
                    Direction::Download,
                    Arc::clone(&self.inner.context.traffic),
                )
                .await?,
            );
            client.flush().await.map_err(forward_io)?;

            let latency_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
            let log = HttpRequestLog {
                id: Uuid::new_v4().to_string(),
                tunnel_id: Some(route.tunnel_id.to_string()),
                route_name: Some(route.name.clone()),
                method: request.method.clone(),
                url: request.target.clone(),
                path: request.path.clone(),
                host: request.host.clone(),
                status: response.status,
                latency_ms,
                client_ip: remote_addr.ip().to_string(),
                upload_bytes,
                download_bytes,
                timestamp_millis: chrono::Utc::now().timestamp_millis(),
                http_version: Some(request.version.clone()),
                tls: metadata.tls.clone(),
            };
            self.inner.logs.record(Some(&route), log);

            info!(
                target: "gate_runtime",
                tunnel_id = %route.tunnel_id,
                method = %request.method,
                path = %request.path,
                host = %request.host,
                status = response.status,
                latency_ms,
                upload_bytes,
                download_bytes,
                "HTTP Request"
            );

            if !keep_client_alive {
                return Ok(());
            }
        }
    }

    async fn send_error_response<S>(
        &self,
        client: &mut S,
        route: Option<&HttpRouteConfig>,
        method: &str,
        url: &str,
        host: &str,
        remote_addr: SocketAddr,
        status: u16,
        message: &str,
        started: Instant,
        http_version: Option<&str>,
        metadata: &HttpConnectionMetadata,
    ) -> Result<HttpRequestLog, RuntimeError>
    where
        S: AsyncWrite + Unpin,
    {
        let body = format!("{message}\n");
        let response = format!(
            "HTTP/1.1 {status} {message}\r\nContent-Length: {}\r\nContent-Type: text/plain; charset=utf-8\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        client
            .write_all(response.as_bytes())
            .await
            .map_err(forward_io)?;
        client.flush().await.map_err(forward_io)?;
        self.inner
            .context
            .traffic
            .record_download(response.len() as u64);

        Ok(HttpRequestLog {
            id: Uuid::new_v4().to_string(),
            tunnel_id: route.map(|route| route.tunnel_id.to_string()),
            route_name: route.map(|route| route.name.clone()),
            method: method.to_string(),
            url: url.to_string(),
            path: target_to_path(url),
            host: host.to_string(),
            status,
            latency_ms: started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
            client_ip: remote_addr.ip().to_string(),
            upload_bytes: 0,
            download_bytes: response.len() as u64,
            timestamp_millis: chrono::Utc::now().timestamp_millis(),
            http_version: http_version.map(ToString::to_string),
            tls: metadata.tls.clone(),
        })
    }

    fn select_route_with_metadata(
        &self,
        host: &str,
        path: &str,
        metadata: &HttpConnectionMetadata,
    ) -> Result<Option<HttpRouteConfig>, String> {
        let Some(resolver) = metadata.host_resolver.as_ref() else {
            return Ok(self.select_route(host, path));
        };
        let tunnel_id = resolver.resolve_host(host)?;
        Ok(self
            .select_route_by_tunnel_id(&tunnel_id, path)
            .or_else(|| self.select_route(host, path)))
    }

    fn select_route_by_tunnel_id(&self, tunnel_id: &str, path: &str) -> Option<HttpRouteConfig> {
        let routes = self.inner.routes.read();
        routes
            .iter()
            .filter(|route| route.enabled)
            .filter(|route| route.tunnel_id.to_string() == tunnel_id)
            .filter(|route| path_matches(route.path_prefix.as_deref(), path))
            .max_by_key(|route| route.path_prefix.as_ref().map_or(1, |path| path.len()))
            .cloned()
    }

    fn select_route(&self, host: &str, path: &str) -> Option<HttpRouteConfig> {
        let host = normalize_host_value(host);
        let routes = self.inner.routes.read();
        routes
            .iter()
            .filter(|route| route.enabled)
            .filter(|route| host_matches(route.host.as_deref(), &host))
            .filter(|route| path_matches(route.path_prefix.as_deref(), path))
            .max_by_key(|route| {
                let host_score = usize::from(route.host.is_some());
                let path_score = route.path_prefix.as_ref().map_or(1, |path| path.len());
                (host_score, path_score)
            })
            .cloned()
    }

    fn start_monitor_task(&self) -> Result<(), RuntimeError> {
        let context = Arc::clone(&self.inner.context);
        let monitor = Arc::clone(&self.inner.monitor);
        let interval = context
            .config
            .monitor_interval
            .max(Duration::from_millis(100));
        let tunnel_id = self.tunnel_id();

        self.inner.scheduler.spawn_monitor(
            format!("http-runtime-monitor:{tunnel_id}"),
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
        let context = Arc::clone(&self.inner.context);
        let interval = context
            .config
            .cleanup_interval
            .max(Duration::from_millis(100));
        let tunnel_id = self.tunnel_id();

        self.inner.scheduler.spawn_cleanup(
            format!("http-runtime-cleanup:{tunnel_id}"),
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

impl RuntimeLifecycle for HttpTunnelRuntime {
    fn state(&self) -> RuntimeState {
        self.inner.context.state.current()
    }

    fn start(&self) -> BoxFuture<'static, Result<(), RuntimeError>> {
        let runtime = self.clone();
        Box::pin(async move { HttpTunnelRuntime::start(&runtime).await })
    }

    fn stop(&self) -> BoxFuture<'static, Result<(), RuntimeError>> {
        let runtime = self.clone();
        Box::pin(async move { HttpTunnelRuntime::stop(&runtime).await })
    }

    fn restart(&self) -> BoxFuture<'static, Result<(), RuntimeError>> {
        let runtime = self.clone();
        Box::pin(async move { HttpTunnelRuntime::restart(&runtime).await })
    }

    fn pause(&self) -> BoxFuture<'static, Result<(), RuntimeError>> {
        let runtime = self.clone();
        Box::pin(async move { HttpTunnelRuntime::pause(&runtime).await })
    }

    fn resume(&self) -> BoxFuture<'static, Result<(), RuntimeError>> {
        let runtime = self.clone();
        Box::pin(async move { HttpTunnelRuntime::resume(&runtime).await })
    }

    fn shutdown(&self) -> BoxFuture<'static, Result<(), RuntimeError>> {
        let runtime = self.clone();
        Box::pin(async move { HttpTunnelRuntime::shutdown(&runtime).await })
    }
}

/// Builder for [`HttpTunnelRuntime`].
#[derive(Debug, Clone)]
pub struct HttpRuntimeBuilder {
    config: HttpRuntimeConfig,
}

impl Default for HttpRuntimeBuilder {
    fn default() -> Self {
        Self {
            config: HttpRuntimeConfig::default(),
        }
    }
}

impl HttpRuntimeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn config(mut self, config: HttpRuntimeConfig) -> Self {
        self.config = config;
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.base.name = name.into();
        self
    }

    pub fn listen_addr(mut self, listen_addr: SocketAddr) -> Self {
        self.config.base.listener.listen_addr = listen_addr;
        self
    }

    pub fn route(mut self, route: HttpRouteConfig) -> Self {
        self.config.routes.push(route);
        self
    }

    pub fn routes(mut self, routes: Vec<HttpRouteConfig>) -> Self {
        self.config.routes = routes;
        self
    }

    pub fn timeout(mut self, timeout: TimeoutConfig) -> Self {
        self.config.base.timeout = timeout;
        self
    }

    pub fn build(self) -> HttpTunnelRuntime {
        HttpTunnelRuntime::new(self.config)
    }
}

#[derive(Debug, Clone)]
struct Header {
    name: String,
    value: String,
}

#[derive(Debug, Clone)]
struct HttpHead {
    method: String,
    target: String,
    path: String,
    version: String,
    status: u16,
    reason: String,
    host: String,
    headers: Vec<Header>,
}

impl HttpHead {
    fn parse_request(bytes: &[u8]) -> Result<Self, String> {
        let text = std::str::from_utf8(bytes).map_err(|error| error.to_string())?;
        let mut lines = text.split("\r\n");
        let start_line = lines
            .next()
            .ok_or_else(|| "missing request line".to_string())?
            .to_string();
        let parts = start_line.split_whitespace().collect::<Vec<_>>();
        if parts.len() != 3 {
            return Err("invalid request line".to_string());
        }
        let method = parts[0].to_string();
        let target = parts[1].to_string();
        let version = parts[2].to_string();
        let headers = parse_headers(lines)?;
        let host = header_value(&headers, "Host").unwrap_or_default();
        Ok(Self {
            method,
            target: target.clone(),
            path: target_to_path(&target),
            version,
            status: 0,
            reason: String::new(),
            host,
            headers,
        })
    }

    fn parse_response(bytes: &[u8]) -> Result<Self, String> {
        let text = std::str::from_utf8(bytes).map_err(|error| error.to_string())?;
        let mut lines = text.split("\r\n");
        let start_line = lines
            .next()
            .ok_or_else(|| "missing status line".to_string())?
            .to_string();
        let mut parts = start_line.splitn(3, ' ');
        let version = parts
            .next()
            .ok_or_else(|| "missing HTTP version".to_string())?
            .to_string();
        let status = parts
            .next()
            .ok_or_else(|| "missing status code".to_string())?
            .parse::<u16>()
            .map_err(|error| error.to_string())?;
        let reason = parts.next().unwrap_or_default().to_string();
        let headers = parse_headers(lines)?;
        Ok(Self {
            method: String::new(),
            target: String::new(),
            path: String::new(),
            version,
            status,
            reason,
            host: String::new(),
            headers,
        })
    }

    fn header(&self, name: &str) -> Option<String> {
        header_value(&self.headers, name)
    }

    fn keep_alive(&self) -> bool {
        let connection = self.header("Connection").unwrap_or_default();
        if connection
            .split(',')
            .any(|value| value.trim().eq_ignore_ascii_case("close"))
        {
            return false;
        }
        if self.version.eq_ignore_ascii_case("HTTP/1.0") {
            return connection
                .split(',')
                .any(|value| value.trim().eq_ignore_ascii_case("keep-alive"));
        }
        true
    }

    fn is_upgrade(&self) -> bool {
        self.header("Upgrade").is_some()
            || self
                .header("Connection")
                .unwrap_or_default()
                .split(',')
                .any(|value| value.trim().eq_ignore_ascii_case("upgrade"))
    }
}

#[derive(Debug, Clone, Copy)]
enum BodyKind {
    None,
    ContentLength(u64),
    Chunked,
    UntilEof,
}

#[derive(Debug, Clone, Copy)]
enum Direction {
    Upload,
    Download,
}

async fn read_head<S>(stream: &mut S, max_header_bytes: usize) -> std::io::Result<Option<Vec<u8>>>
where
    S: AsyncRead + Unpin,
{
    let mut buffer = Vec::with_capacity(1024);
    let mut byte = [0_u8; 1];

    loop {
        let read = stream.read(&mut byte).await?;
        if read == 0 {
            if buffer.is_empty() {
                return Ok(None);
            }
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "connection closed while reading HTTP headers",
            ));
        }
        buffer.push(byte[0]);
        if buffer.len() > max_header_bytes {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "HTTP header limit exceeded",
            ));
        }
        if buffer.ends_with(b"\r\n\r\n") {
            return Ok(Some(buffer));
        }
    }
}

fn parse_headers<'a, I>(lines: I) -> Result<Vec<Header>, String>
where
    I: Iterator<Item = &'a str>,
{
    let mut headers = Vec::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        let Some((name, value)) = line.split_once(':') else {
            return Err(format!("invalid header line: {line}"));
        };
        headers.push(Header {
            name: name.trim().to_string(),
            value: value.trim().to_string(),
        });
    }
    Ok(headers)
}

fn request_body_kind(request: &HttpHead) -> BodyKind {
    if transfer_encoding_is_chunked(&request.headers) {
        return BodyKind::Chunked;
    }
    if let Some(length) = content_length(&request.headers) {
        return BodyKind::ContentLength(length);
    }
    BodyKind::None
}

fn response_body_kind(method: &str, status: u16, response: &HttpHead) -> BodyKind {
    if method.eq_ignore_ascii_case("HEAD")
        || (100..200).contains(&status)
        || status == 204
        || status == 304
    {
        return BodyKind::None;
    }
    if transfer_encoding_is_chunked(&response.headers) {
        return BodyKind::Chunked;
    }
    if let Some(length) = content_length(&response.headers) {
        return BodyKind::ContentLength(length);
    }
    BodyKind::UntilEof
}

async fn copy_body<R, W>(
    reader: &mut R,
    writer: &mut W,
    kind: BodyKind,
    direction: Direction,
    traffic: Arc<crate::runtime::monitor::TrafficStatistics>,
) -> Result<u64, RuntimeError>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    match kind {
        BodyKind::None => Ok(0),
        BodyKind::ContentLength(length) => {
            copy_exact_count(reader, writer, length, direction, traffic).await
        }
        BodyKind::Chunked => copy_chunked(reader, writer, direction, traffic).await,
        BodyKind::UntilEof => copy_until_eof(reader, writer, direction, traffic).await,
    }
}

async fn copy_exact_count<R, W>(
    reader: &mut R,
    writer: &mut W,
    mut remaining: u64,
    direction: Direction,
    traffic: Arc<crate::runtime::monitor::TrafficStatistics>,
) -> Result<u64, RuntimeError>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut copied = 0_u64;
    let mut buffer = vec![0_u8; 16 * 1024];
    while remaining > 0 {
        let read_len = remaining.min(buffer.len() as u64) as usize;
        let read = reader
            .read(&mut buffer[..read_len])
            .await
            .map_err(forward_io)?;
        if read == 0 {
            return Err(RuntimeError::Forward(ForwardError::Io {
                source: Error::new(ErrorKind::UnexpectedEof, "body ended early"),
            }));
        }
        writer
            .write_all(&buffer[..read])
            .await
            .map_err(forward_io)?;
        record_direction(&traffic, direction, read as u64);
        copied = copied.saturating_add(read as u64);
        remaining = remaining.saturating_sub(read as u64);
    }
    Ok(copied)
}

async fn copy_until_eof<R, W>(
    reader: &mut R,
    writer: &mut W,
    direction: Direction,
    traffic: Arc<crate::runtime::monitor::TrafficStatistics>,
) -> Result<u64, RuntimeError>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut copied = 0_u64;
    let mut buffer = vec![0_u8; 16 * 1024];
    loop {
        let read = reader.read(&mut buffer).await.map_err(forward_io)?;
        if read == 0 {
            break;
        }
        writer
            .write_all(&buffer[..read])
            .await
            .map_err(forward_io)?;
        record_direction(&traffic, direction, read as u64);
        copied = copied.saturating_add(read as u64);
    }
    Ok(copied)
}

async fn copy_chunked<R, W>(
    reader: &mut R,
    writer: &mut W,
    direction: Direction,
    traffic: Arc<crate::runtime::monitor::TrafficStatistics>,
) -> Result<u64, RuntimeError>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let mut copied = 0_u64;
    loop {
        let line = read_crlf_line(reader, MAX_CHUNK_LINE_BYTES)
            .await
            .map_err(forward_io)?;
        writer.write_all(&line).await.map_err(forward_io)?;
        record_direction(&traffic, direction, line.len() as u64);
        copied = copied.saturating_add(line.len() as u64);

        let size = parse_chunk_size(&line)?;
        if size > 0 {
            copied = copied.saturating_add(
                copy_exact_count(reader, writer, size, direction, Arc::clone(&traffic)).await?,
            );
            copied = copied.saturating_add(
                copy_exact_count(reader, writer, 2, direction, Arc::clone(&traffic)).await?,
            );
            continue;
        }

        loop {
            let trailer = read_crlf_line(reader, MAX_CHUNK_LINE_BYTES)
                .await
                .map_err(forward_io)?;
            writer.write_all(&trailer).await.map_err(forward_io)?;
            record_direction(&traffic, direction, trailer.len() as u64);
            copied = copied.saturating_add(trailer.len() as u64);
            if trailer == b"\r\n" {
                return Ok(copied);
            }
        }
    }
}

async fn read_crlf_line<S>(stream: &mut S, max_bytes: usize) -> std::io::Result<Vec<u8>>
where
    S: AsyncRead + Unpin,
{
    let mut line = Vec::new();
    let mut byte = [0_u8; 1];
    loop {
        let read = stream.read(&mut byte).await?;
        if read == 0 {
            return Err(Error::new(
                ErrorKind::UnexpectedEof,
                "connection closed while reading HTTP line",
            ));
        }
        line.push(byte[0]);
        if line.len() > max_bytes {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "HTTP line limit exceeded",
            ));
        }
        if line.ends_with(b"\r\n") {
            return Ok(line);
        }
    }
}

fn parse_chunk_size(line: &[u8]) -> Result<u64, RuntimeError> {
    let line = std::str::from_utf8(line).map_err(|source| {
        RuntimeError::Forward(ForwardError::Io {
            source: Error::new(ErrorKind::InvalidData, source),
        })
    })?;
    let size_hex = line
        .trim()
        .split_once(';')
        .map_or(line.trim(), |(size, _)| size.trim());
    u64::from_str_radix(size_hex, 16).map_err(|source| {
        RuntimeError::Forward(ForwardError::Io {
            source: Error::new(ErrorKind::InvalidData, source),
        })
    })
}

fn build_request_head(
    request: &HttpHead,
    route: &HttpRouteConfig,
    remote_addr: SocketAddr,
    forwarded_proto: Option<&str>,
) -> Vec<u8> {
    let mut headers = request.headers.clone();
    remove_hop_by_hop_headers(&mut headers);
    remove_headers(&mut headers, &route.headers.request_remove);
    set_headers(&mut headers, &route.headers.request_set);

    let client_ip = remote_addr.ip().to_string();
    if route.headers.x_forwarded_for {
        append_header_value(&mut headers, "X-Forwarded-For", &client_ip);
    }
    if route.headers.x_real_ip {
        set_header(&mut headers, "X-Real-IP", &client_ip);
    }
    if route.headers.x_forwarded_proto {
        set_header(
            &mut headers,
            "X-Forwarded-Proto",
            forwarded_proto.unwrap_or(&route.headers.forwarded_proto),
        );
    }
    set_header(&mut headers, "Connection", "close");

    let mut out = Vec::new();
    let start_line = format!(
        "{} {} {}\r\n",
        request.method, request.path, request.version
    );
    out.extend_from_slice(start_line.as_bytes());
    write_headers(&mut out, &headers);
    out.extend_from_slice(b"\r\n");
    out
}

fn build_response_head(response: &HttpHead, route: &HttpRouteConfig, keep_alive: bool) -> Vec<u8> {
    let mut headers = response.headers.clone();
    remove_hop_by_hop_headers(&mut headers);
    remove_headers(&mut headers, &route.headers.response_remove);
    set_headers(&mut headers, &route.headers.response_set);
    set_header(
        &mut headers,
        "Connection",
        if keep_alive { "keep-alive" } else { "close" },
    );

    let mut out = Vec::new();
    let reason = if response.reason.is_empty() {
        default_reason(response.status)
    } else {
        response.reason.as_str()
    };
    let start_line = format!("{} {} {}\r\n", response.version, response.status, reason);
    out.extend_from_slice(start_line.as_bytes());
    write_headers(&mut out, &headers);
    out.extend_from_slice(b"\r\n");
    out
}

fn write_headers(out: &mut Vec<u8>, headers: &[Header]) {
    for header in headers {
        out.extend_from_slice(header.name.as_bytes());
        out.extend_from_slice(b": ");
        out.extend_from_slice(header.value.as_bytes());
        out.extend_from_slice(b"\r\n");
    }
}

fn remove_hop_by_hop_headers(headers: &mut Vec<Header>) {
    let mut names = vec![
        "connection".to_string(),
        "proxy-connection".to_string(),
        "keep-alive".to_string(),
        "te".to_string(),
        "trailer".to_string(),
        "upgrade".to_string(),
    ];
    for value in headers
        .iter()
        .filter(|header| header.name.eq_ignore_ascii_case("Connection"))
        .flat_map(|header| header.value.split(','))
    {
        names.push(value.trim().to_ascii_lowercase());
    }
    headers.retain(|header| {
        let name = header.name.to_ascii_lowercase();
        !names.iter().any(|blocked| blocked == &name)
    });
}

fn remove_headers(headers: &mut Vec<Header>, names: &[String]) {
    headers.retain(|header| {
        !names
            .iter()
            .any(|name| header.name.eq_ignore_ascii_case(name))
    });
}

fn set_headers(headers: &mut Vec<Header>, rules: &[HttpHeaderRule]) {
    for rule in rules {
        set_header(headers, &rule.name, &rule.value);
    }
}

fn set_header(headers: &mut Vec<Header>, name: &str, value: &str) {
    if let Some(header) = headers
        .iter_mut()
        .find(|header| header.name.eq_ignore_ascii_case(name))
    {
        header.value = value.to_string();
    } else {
        headers.push(Header {
            name: canonical_header_name(name),
            value: value.to_string(),
        });
    }
}

fn append_header_value(headers: &mut Vec<Header>, name: &str, value: &str) {
    if let Some(header) = headers
        .iter_mut()
        .find(|header| header.name.eq_ignore_ascii_case(name))
    {
        if header.value.trim().is_empty() {
            header.value = value.to_string();
        } else {
            header.value = format!("{}, {}", header.value, value);
        }
    } else {
        headers.push(Header {
            name: canonical_header_name(name),
            value: value.to_string(),
        });
    }
}

fn header_value(headers: &[Header], name: &str) -> Option<String> {
    headers
        .iter()
        .find(|header| header.name.eq_ignore_ascii_case(name))
        .map(|header| header.value.clone())
}

fn content_length(headers: &[Header]) -> Option<u64> {
    header_value(headers, "Content-Length")?.parse().ok()
}

fn transfer_encoding_is_chunked(headers: &[Header]) -> bool {
    header_value(headers, "Transfer-Encoding")
        .map(|value| {
            value
                .split(',')
                .any(|part| part.trim().eq_ignore_ascii_case("chunked"))
        })
        .unwrap_or(false)
}

fn normalize_host_config(host: &str) -> String {
    normalize_host_value(host)
}

fn normalize_host_value(host: &str) -> String {
    let host = host.trim().trim_end_matches('.');
    let host = if let Some(stripped) = host.strip_prefix('[') {
        stripped
            .split_once(']')
            .map(|(inside, _)| inside)
            .unwrap_or(stripped)
    } else {
        host.split_once(':').map_or(host, |(name, _)| name)
    };
    host.to_ascii_lowercase()
}

fn normalize_path_prefix(path: &str) -> String {
    let path = path.trim();
    if path.is_empty() || path == "/" {
        return "/".to_string();
    }
    let mut normalized = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };
    while normalized.len() > 1 && normalized.ends_with('/') {
        normalized.pop();
    }
    normalized
}

fn host_matches(configured: Option<&str>, host: &str) -> bool {
    let Some(configured) = configured else {
        return true;
    };
    let configured = configured.to_ascii_lowercase();
    if configured == "*" {
        return true;
    }
    if let Some(suffix) = configured.strip_prefix("*.") {
        return host == suffix || host.ends_with(&format!(".{suffix}"));
    }
    configured == host
}

fn path_matches(prefix: Option<&str>, path: &str) -> bool {
    let prefix = prefix.unwrap_or("/");
    if prefix == "/" {
        return true;
    }
    path == prefix || path.starts_with(&format!("{prefix}/"))
}

fn target_to_path(target: &str) -> String {
    if target.is_empty() {
        return "/".to_string();
    }
    if target.starts_with('/') {
        return target.to_string();
    }
    if let Some(rest) = target
        .strip_prefix("http://")
        .or_else(|| target.strip_prefix("https://"))
    {
        if let Some(index) = rest.find('/') {
            return rest[index..].to_string();
        }
        return "/".to_string();
    }
    target.to_string()
}

fn canonical_header_name(name: &str) -> String {
    name.split('-')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => format!(
                    "{}{}",
                    first.to_ascii_uppercase(),
                    chars.as_str().to_ascii_lowercase()
                ),
                None => String::new(),
            }
        })
        .collect::<Vec<_>>()
        .join("-")
}

fn default_reason(status: u16) -> &'static str {
    match status {
        100 => "Continue",
        101 => "Switching Protocols",
        200 => "OK",
        201 => "Created",
        202 => "Accepted",
        204 => "No Content",
        301 => "Moved Permanently",
        302 => "Found",
        304 => "Not Modified",
        400 => "Bad Request",
        401 => "Unauthorized",
        403 => "Forbidden",
        404 => "Not Found",
        500 => "Internal Server Error",
        501 => "Not Implemented",
        502 => "Bad Gateway",
        503 => "Service Unavailable",
        _ => "OK",
    }
}

fn record_direction(
    traffic: &crate::runtime::monitor::TrafficStatistics,
    direction: Direction,
    bytes: u64,
) {
    match direction {
        Direction::Upload => traffic.record_upload(bytes),
        Direction::Download => traffic.record_download(bytes),
    }
}

fn ratio(success: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        success as f64 / total as f64
    }
}

fn average(total_latency_ms: u128, count: u64) -> f64 {
    if count == 0 {
        0.0
    } else {
        total_latency_ms as f64 / count as f64
    }
}

fn saturating_u128_to_u64(value: u128) -> u64 {
    value.min(u128::from(u64::MAX)) as u64
}

fn forward_io(source: Error) -> RuntimeError {
    RuntimeError::Forward(ForwardError::Io { source })
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
