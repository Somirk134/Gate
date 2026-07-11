use chrono::Utc;
use gate_protocol::{
    Body, Command, Frame, FrameEncoder, Message, MessageType, Metadata, ProtocolBuilder,
    ProtocolManager,
};
use gate_server_domain::{
    model::{
        Domain as ManagedDomain, DomainId as ManagedDomainId, DomainStatus as ManagedDomainStatus,
        Host as ManagedHost, RecordType as ManagedRecordType, TunnelId as ManagedTunnelId,
    },
    repository::{DomainRepository, SqliteRepository as SqliteDomainRepository},
};
use gate_server_tls::{
    acme::{
        AcmeAccountContact, AcmeCertificateRequest, AcmeProvider, Http01ChallengeStore,
        LetsEncryptProvider,
    },
    cert_store::{CertificateStore, FileCertificateStore},
    certificate::{
        CertificateAlgorithm, CertificateFingerprint, CertificateParser, CertificateRecord,
        CertificateStatus, CertificateValidator, StoredCertificate,
    },
    renew::{CertificateRenewer, RenewConfig, RenewScheduler},
    ChallengeType,
};
use rustls::crypto::ring;
use rustls::server::{ClientHello, ResolvesServerCert};
use rustls::sign::CertifiedKey;
use rustls::{ProtocolVersion, ServerConfig};
use serde_json::{json, Value};
use std::io::Cursor;
use std::{
    collections::{HashMap, HashSet, VecDeque},
    io::{self, ErrorKind},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    path::PathBuf,
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc, RwLock as StdRwLock,
    },
    time::{Duration, Instant},
};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    task::JoinHandle,
    time::{sleep, timeout},
};
use tokio_rustls::server::TlsStream;
use tokio_rustls::TlsAcceptor;
use tracing::{info, warn};
use uuid::Uuid;

use crate::discovery::server_capability_snapshot;

const DEFAULT_HEARTBEAT_TIMEOUT_MS: i64 = 90_000;
const HEARTBEAT_SWEEP_INTERVAL: Duration = Duration::from_secs(5);
const DEFAULT_MAX_CONNECTIONS: u64 = 4096;
const DEFAULT_IDLE_TIMEOUT_MS: u64 = 300_000;
const DEFAULT_BANDWIDTH_LIMIT_BPS: u64 = 0;
const RELAY_BUFFER_SIZE: usize = 16 * 1024;
const HTTP_HEADER_LIMIT: usize = 64 * 1024;
const HTTP_ACCESS_LOG_LIMIT: usize = 1024;
const ACME_HTTP01_PREFIX: &str = "/.well-known/acme-challenge/";
const DEFAULT_ACME_HTTP_PORT: u16 = 80;
const DEFAULT_HTTP_PUBLIC_PORT: u16 = 8880;
const DEFAULT_HTTPS_PUBLIC_PORT: u16 = 8443;

#[derive(Debug, Clone)]
pub struct TunnelGateway {
    inner: Arc<GatewayInner>,
}

#[derive(Debug)]
struct GatewayInner {
    bind_ip: IpAddr,
    sessions: Mutex<HashMap<String, ClientSession>>,
    tunnels: Mutex<HashMap<String, Arc<TunnelSession>>>,
    listeners: Mutex<HashMap<u16, ListenerHandle>>,
    domain_repository: Option<SqliteDomainRepository>,
    http_access_logs: Mutex<VecDeque<HttpAccessLog>>,
    http_status_codes: Mutex<HashMap<u16, u64>>,
    certificate_store: FileCertificateStore,
    tls_cache: Arc<GatewayTlsCertificateCache>,
    acme_challenges: Arc<GatewayHttp01Challenges>,
    counters: GatewayCounters,
    http_counters: HttpGatewayCounters,
    https_counters: HttpsGatewayCounters,
    renew_config: RenewConfig,
    heartbeat_timeout_ms: i64,
}

#[derive(Debug, Default)]
struct GatewayCounters {
    registered_tunnels: AtomicU64,
    active_connections: AtomicU64,
    total_connections: AtomicU64,
    failed_connections: AtomicU64,
    failed_sessions: AtomicU64,
    recovered_sessions: AtomicU64,
    bytes_in: AtomicU64,
    bytes_out: AtomicU64,
    errors: AtomicU64,
}

#[derive(Debug, Default)]
struct HttpGatewayCounters {
    requests_total: AtomicU64,
    active_requests: AtomicU64,
    bytes_total: AtomicU64,
    latency_total_ms: AtomicU64,
}

#[derive(Debug, Default)]
struct HttpsGatewayCounters {
    tls_handshake_total: AtomicU64,
    tls_errors: AtomicU64,
    https_requests: AtomicU64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TunnelSessionStatus {
    Initializing,
    Online,
    Offline,
    Recovering,
    Closed,
    Failed,
}

impl TunnelSessionStatus {
    fn as_str(self) -> &'static str {
        match self {
            Self::Initializing => "Initializing",
            Self::Online => "Online",
            Self::Offline => "Offline",
            Self::Recovering => "Recovering",
            Self::Closed => "Closed",
            Self::Failed => "Failed",
        }
    }

    fn is_terminal(self) -> bool {
        matches!(self, Self::Closed | Self::Failed)
    }
}

#[derive(Debug, Clone)]
struct ClientSession {
    client_id: String,
    session_id: String,
    connected: bool,
    status: TunnelSessionStatus,
    created_at: i64,
    last_seen_at: i64,
    registered_tunnels: Vec<String>,
    runtime_status: Value,
}

#[derive(Debug)]
struct TunnelSession {
    config: Mutex<TunnelConfig>,
    // Tunnel 生命周期独立于控制连接，便于断线恢复时保留运行态指标。
    lifecycle: Mutex<TunnelLifecycle>,
    relay_workers: Mutex<VecDeque<TcpStream>>,
    active_connections: AtomicU64,
    total_connections: AtomicU64,
    failed_connections: AtomicU64,
    bytes_in: AtomicU64,
    bytes_out: AtomicU64,
    last_activity: AtomicU64,
    errors: AtomicU64,
    http_requests: AtomicU64,
    http_errors: AtomicU64,
    http_bytes: AtomicU64,
    http_latency_ms: AtomicU64,
}

#[derive(Debug, Clone)]
struct TunnelLifecycle {
    tunnel_id: String,
    client_id: String,
    session_id: String,
    status: TunnelSessionStatus,
    created_at: i64,
    last_heartbeat: i64,
}

#[derive(Debug, Clone)]
struct TunnelConfig {
    tunnel_id: String,
    session_id: String,
    protocol: String,
    remote_port: u16,
    local_host: String,
    local_port: u16,
    host: Option<String>,
    path: Option<String>,
    metadata: Value,
    enabled: bool,
    updated_at: i64,
    runtime: TunnelRuntimeConfig,
}

#[derive(Debug, Clone, Copy)]
struct TunnelRuntimeConfig {
    max_connections: u64,
    idle_timeout_ms: u64,
    bandwidth_limit_bps: u64,
}

impl Default for TunnelRuntimeConfig {
    fn default() -> Self {
        Self {
            max_connections: DEFAULT_MAX_CONNECTIONS,
            idle_timeout_ms: DEFAULT_IDLE_TIMEOUT_MS,
            bandwidth_limit_bps: DEFAULT_BANDWIDTH_LIMIT_BPS,
        }
    }
}

#[derive(Debug)]
struct ListenerHandle {
    tunnel_id: String,
    protocol: ListenerProtocol,
    handle: JoinHandle<()>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ListenerProtocol {
    Tcp,
    Http,
    Https,
}

#[derive(Debug, Clone)]
struct HttpAccessLog {
    method: String,
    path: String,
    host: String,
    tunnel_id: Option<String>,
    status: u16,
    latency_ms: u64,
    bytes: u64,
    timestamp_millis: i64,
    scheme: &'static str,
    tls_version: Option<String>,
    sni: Option<String>,
}

#[derive(Debug, Clone)]
struct HttpGatewayConnection {
    scheme: &'static str,
    tls_version: Option<String>,
    sni: Option<String>,
}

impl HttpGatewayConnection {
    fn plain() -> Self {
        Self {
            scheme: "http",
            tls_version: None,
            sni: None,
        }
    }

    fn tls(tls_version: Option<String>, sni: Option<String>) -> Self {
        Self {
            scheme: "https",
            tls_version,
            sni,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct RelayCopyOutcome {
    bytes_in: u64,
    bytes_out: u64,
    duration_ms: u64,
}

#[derive(Debug, Clone, Copy)]
enum CopyDirection {
    BytesIn,
    BytesOut,
}

impl TunnelGateway {
    pub fn new() -> Self {
        Self::with_heartbeat_timeout_and_domain_repository(
            std::env::var("GATE_HEARTBEAT_TIMEOUT_MS")
                .ok()
                .and_then(|value| value.parse::<i64>().ok())
                .filter(|value| *value > 0)
                .unwrap_or(DEFAULT_HEARTBEAT_TIMEOUT_MS),
            default_domain_repository(),
        )
    }

    pub fn with_domain_repository(domain_repository: SqliteDomainRepository) -> Self {
        Self::with_heartbeat_timeout_and_domain_repository(
            DEFAULT_HEARTBEAT_TIMEOUT_MS,
            Some(domain_repository),
        )
    }

    #[cfg(test)]
    fn with_heartbeat_timeout(heartbeat_timeout_ms: i64) -> Self {
        Self::with_heartbeat_timeout_and_domain_repository(heartbeat_timeout_ms, None)
    }

    #[cfg(test)]
    fn with_certificate_store_root(certificate_store_root: PathBuf) -> Self {
        Self::with_gateway_options(DEFAULT_HEARTBEAT_TIMEOUT_MS, None, certificate_store_root)
    }

    fn with_heartbeat_timeout_and_domain_repository(
        heartbeat_timeout_ms: i64,
        domain_repository: Option<SqliteDomainRepository>,
    ) -> Self {
        Self::with_gateway_options(
            heartbeat_timeout_ms,
            domain_repository,
            certificate_store_root(),
        )
    }

    fn with_gateway_options(
        heartbeat_timeout_ms: i64,
        domain_repository: Option<SqliteDomainRepository>,
        certificate_store_root: PathBuf,
    ) -> Self {
        let bind_ip = std::env::var("GATE_TUNNEL_BIND_ADDR")
            .ok()
            .and_then(|value| value.parse::<IpAddr>().ok())
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));

        let gateway = Self {
            inner: Arc::new(GatewayInner {
                bind_ip,
                sessions: Mutex::new(HashMap::new()),
                tunnels: Mutex::new(HashMap::new()),
                listeners: Mutex::new(HashMap::new()),
                domain_repository,
                http_access_logs: Mutex::new(VecDeque::new()),
                http_status_codes: Mutex::new(HashMap::new()),
                certificate_store: FileCertificateStore::new(certificate_store_root),
                tls_cache: Arc::new(GatewayTlsCertificateCache::default()),
                acme_challenges: Arc::new(GatewayHttp01Challenges::default()),
                counters: GatewayCounters::default(),
                http_counters: HttpGatewayCounters::default(),
                https_counters: HttpsGatewayCounters::default(),
                renew_config: RenewConfig::default(),
                heartbeat_timeout_ms,
            }),
        };
        gateway.start_heartbeat_watcher();
        gateway.start_certificate_renew_watcher();
        gateway
    }

    pub async fn create_session(&self) -> String {
        self.create_or_restore_session(None, None).await
    }

    pub async fn create_or_restore_session(
        &self,
        client_id: Option<String>,
        requested_session_id: Option<String>,
    ) -> String {
        let session_id = requested_session_id
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| Uuid::new_v4().to_string());
        let client_id = client_id
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or_else(|| format!("client:{session_id}"));
        let now = Utc::now().timestamp_millis();

        {
            let mut sessions = self.inner.sessions.lock().await;
            if let Some(session) = sessions.get_mut(&session_id) {
                session.client_id = client_id.clone();
                session.connected = true;
                session.status = TunnelSessionStatus::Recovering;
                session.last_seen_at = now;
                self.inner
                    .counters
                    .recovered_sessions
                    .fetch_add(1, Ordering::Relaxed);
            } else {
                sessions.insert(
                    session_id.clone(),
                    ClientSession {
                        client_id: client_id.clone(),
                        session_id: session_id.clone(),
                        connected: true,
                        status: TunnelSessionStatus::Online,
                        created_at: now,
                        last_seen_at: now,
                        registered_tunnels: Vec::new(),
                        runtime_status: json!({}),
                    },
                );
            }
        }

        self.recover_tunnels_for_session(&session_id, &client_id, now)
            .await;
        session_id
    }

    pub async fn record_session_failure(&self) {
        self.inner
            .counters
            .failed_sessions
            .fetch_add(1, Ordering::Relaxed);
        self.inner.counters.errors.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn touch_session(&self, session_id: &str) {
        let now = Utc::now().timestamp_millis();
        if let Some(session) = self.inner.sessions.lock().await.get_mut(session_id) {
            session.connected = true;
            session.status = TunnelSessionStatus::Online;
            session.last_seen_at = now;
        }
    }

    pub async fn update_heartbeat(&self, session_id: &str, body: &Value) -> Value {
        let now = Utc::now().timestamp_millis();
        let client_id = heartbeat_client_id(body);
        let registered_tunnels = heartbeat_registered_tunnels(body);
        let runtime_status = heartbeat_runtime_status(body);
        let client_status = heartbeat_client_status(body)
            .unwrap_or_else(|| "online".to_string())
            .to_ascii_lowercase();

        let stored_client_id = {
            let mut sessions = self.inner.sessions.lock().await;
            let session = sessions
                .entry(session_id.to_string())
                .or_insert_with(|| ClientSession {
                    client_id: client_id
                        .clone()
                        .unwrap_or_else(|| format!("client:{session_id}")),
                    session_id: session_id.to_string(),
                    connected: true,
                    status: TunnelSessionStatus::Online,
                    created_at: now,
                    last_seen_at: now,
                    registered_tunnels: Vec::new(),
                    runtime_status: json!({}),
                });

            if let Some(client_id) = client_id {
                session.client_id = client_id;
            }
            session.connected = client_status != "offline";
            session.status = if session.connected {
                TunnelSessionStatus::Online
            } else {
                TunnelSessionStatus::Offline
            };
            session.last_seen_at = now;
            session.registered_tunnels = registered_tunnels.clone();
            session.runtime_status = runtime_status.clone();
            session.client_id.clone()
        };

        self.apply_heartbeat_to_tunnels(session_id, &stored_client_id, &registered_tunnels, now)
            .await;

        json!({
            "sessionId": session_id,
            "clientId": stored_client_id,
            "status": client_status,
            "registeredTunnels": registered_tunnels,
            "timestamp": now
        })
    }

    pub async fn expire_heartbeat_timeouts(&self) {
        let now = Utc::now().timestamp_millis();
        let timeout_ms = self.inner.heartbeat_timeout_ms;

        {
            let mut sessions = self.inner.sessions.lock().await;
            for session in sessions.values_mut() {
                if session.connected && now.saturating_sub(session.last_seen_at) > timeout_ms {
                    session.connected = false;
                    session.status = TunnelSessionStatus::Offline;
                }
            }
        }

        let tunnels = {
            let tunnels = self.inner.tunnels.lock().await;
            tunnels.values().cloned().collect::<Vec<_>>()
        };

        for tunnel in tunnels {
            let mut lifecycle = tunnel.lifecycle.lock().await;
            if lifecycle.status.is_terminal() {
                continue;
            }
            if now.saturating_sub(lifecycle.last_heartbeat) > timeout_ms {
                lifecycle.status = TunnelSessionStatus::Offline;
                tunnel.relay_workers.lock().await.clear();
            }
        }
    }

    pub async fn close_session(&self, session_id: &str) {
        let now = Utc::now().timestamp_millis();
        if let Some(session) = self.inner.sessions.lock().await.get_mut(session_id) {
            session.connected = false;
            session.status = TunnelSessionStatus::Recovering;
            session.last_seen_at = now;
        }

        let tunnels = {
            let tunnels = self.inner.tunnels.lock().await;
            tunnels.values().cloned().collect::<Vec<_>>()
        };

        for tunnel in tunnels {
            let matches_session = tunnel.lifecycle.lock().await.session_id == session_id;
            if matches_session {
                {
                    let mut lifecycle = tunnel.lifecycle.lock().await;
                    if !lifecycle.status.is_terminal() {
                        lifecycle.status = TunnelSessionStatus::Recovering;
                    }
                }
                tunnel.relay_workers.lock().await.clear();
            }
        }
    }

    pub async fn register_tunnel(&self, session_id: &str, body: &Value) -> anyhow::Result<Value> {
        self.touch_session(session_id).await;
        let client_id = self
            .client_id_for_session(session_id)
            .await
            .unwrap_or_else(|| format!("client:{session_id}"));
        let config = parse_tunnel_config(session_id, body)?;
        let tunnel_id = config.tunnel_id.clone();
        let mut stale_remote_port = None;
        let now = Utc::now().timestamp_millis();

        {
            let mut tunnels = self.inner.tunnels.lock().await;
            if let Some(existing) = tunnels.get(&tunnel_id) {
                let mut existing_config = existing.config.lock().await;
                if existing_config.remote_port != config.remote_port {
                    stale_remote_port = Some(existing_config.remote_port);
                    existing.relay_workers.lock().await.clear();
                }
                *existing_config = config.clone();

                let mut lifecycle = existing.lifecycle.lock().await;
                lifecycle.client_id = client_id.clone();
                lifecycle.session_id = session_id.to_string();
                lifecycle.status = TunnelSessionStatus::Recovering;
                lifecycle.last_heartbeat = now;
            } else {
                let session = Arc::new(TunnelSession {
                    config: Mutex::new(config.clone()),
                    lifecycle: Mutex::new(TunnelLifecycle {
                        tunnel_id: tunnel_id.clone(),
                        client_id: client_id.clone(),
                        session_id: session_id.to_string(),
                        status: TunnelSessionStatus::Initializing,
                        created_at: now,
                        last_heartbeat: now,
                    }),
                    relay_workers: Mutex::new(VecDeque::new()),
                    active_connections: AtomicU64::new(0),
                    total_connections: AtomicU64::new(0),
                    failed_connections: AtomicU64::new(0),
                    bytes_in: AtomicU64::new(0),
                    bytes_out: AtomicU64::new(0),
                    last_activity: AtomicU64::new(now.max(0) as u64),
                    errors: AtomicU64::new(0),
                    http_requests: AtomicU64::new(0),
                    http_errors: AtomicU64::new(0),
                    http_bytes: AtomicU64::new(0),
                    http_latency_ms: AtomicU64::new(0),
                });
                tunnels.insert(tunnel_id.clone(), session);
                self.inner
                    .counters
                    .registered_tunnels
                    .fetch_add(1, Ordering::Relaxed);
            }
        }

        self.register_tunnel_on_client_session(session_id, &tunnel_id)
            .await;

        if let Some(remote_port) = stale_remote_port {
            self.stop_listener_if_unused(remote_port).await;
        }

        if is_http_tunnel_protocol(&config.protocol) {
            self.bind_configured_domain(&config).await?;
        }

        if config.protocol == "https" {
            self.ensure_https_certificate(&config).await?;
        }

        if is_http_tunnel_protocol(&config.protocol) {
            self.purge_stale_http_tunnels_on_port(&tunnel_id, config.remote_port)
                .await;
        }

        if let Err(source) = self.ensure_listener(tunnel_id.clone(), &config).await {
            self.mark_tunnel_status(&tunnel_id, TunnelSessionStatus::Failed)
                .await;
            return Err(source);
        }

        self.mark_tunnel_status(&tunnel_id, TunnelSessionStatus::Online)
            .await;

        Ok(json!({
            "tunnelId": tunnel_id,
            "remotePort": config.remote_port,
            "protocol": config.protocol.clone(),
            "registeredAt": config.updated_at,
            "session": {
                "clientId": client_id,
                "sessionId": session_id,
                "status": TunnelSessionStatus::Online.as_str()
            },
            "runtime": {
                "maxConnections": config.runtime.max_connections,
                "idleTimeoutMs": config.runtime.idle_timeout_ms,
                "bandwidthLimitBps": config.runtime.bandwidth_limit_bps
            }
        }))
    }

    pub async fn stop_tunnel(&self, tunnel_id: &str) -> anyhow::Result<bool> {
        self.release_domains_for_tunnel(tunnel_id)?;
        let tunnel = self.inner.tunnels.lock().await.remove(tunnel_id);
        let mut removed_listener = false;

        if let Some(tunnel) = tunnel {
            tunnel.relay_workers.lock().await.clear();
            {
                let mut lifecycle = tunnel.lifecycle.lock().await;
                lifecycle.status = TunnelSessionStatus::Closed;
                lifecycle.last_heartbeat = Utc::now().timestamp_millis();
            }
            let remote_port = tunnel.config.lock().await.remote_port;
            removed_listener = self.stop_listener_if_unused(remote_port).await;
        }

        Ok(removed_listener)
    }

    pub async fn attach_relay_worker(
        &self,
        tunnel_id: &str,
        session_id: &str,
        stream: TcpStream,
    ) -> anyhow::Result<()> {
        self.validate_relay_worker(tunnel_id, session_id).await?;
        let tunnel = {
            let tunnels = self.inner.tunnels.lock().await;
            tunnels
                .get(tunnel_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("tunnel not registered"))?
        };

        tunnel.relay_workers.lock().await.push_back(stream);
        self.mark_tunnel_status(tunnel_id, TunnelSessionStatus::Online)
            .await;
        Ok(())
    }

    pub async fn validate_relay_worker(
        &self,
        tunnel_id: &str,
        session_id: &str,
    ) -> anyhow::Result<()> {
        self.touch_session(session_id).await;
        let tunnel = {
            let tunnels = self.inner.tunnels.lock().await;
            tunnels
                .get(tunnel_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("tunnel not registered"))?
        };

        let config = tunnel.config.lock().await.clone();
        if config.session_id != session_id {
            return Err(anyhow::anyhow!("relay session does not own tunnel"));
        }
        if !config.enabled {
            return Err(anyhow::anyhow!("tunnel is stopped"));
        }

        let lifecycle = tunnel.lifecycle.lock().await.clone();
        if matches!(
            lifecycle.status,
            TunnelSessionStatus::Offline
                | TunnelSessionStatus::Closed
                | TunnelSessionStatus::Failed
        ) {
            return Err(anyhow::anyhow!(
                "tunnel session is not online: {}",
                lifecycle.status.as_str()
            ));
        }

        Ok(())
    }

    pub async fn statistics(&self) -> Value {
        self.expire_heartbeat_timeouts().await;
        let sessions = self.inner.sessions.lock().await;
        let tunnels = self.inner.tunnels.lock().await;
        let session_items = sessions
            .values()
            .map(|session| {
                json!({
                    "clientId": session.client_id.clone(),
                    "sessionId": session.session_id.clone(),
                    "status": session.status.as_str(),
                    "connected": session.connected,
                    "createdAt": session.created_at,
                    "lastHeartbeat": session.last_seen_at,
                    "registeredTunnels": session.registered_tunnels.clone(),
                    "runtimeStatus": session.runtime_status.clone()
                })
            })
            .collect::<Vec<_>>();
        let mut tunnel_items = Vec::with_capacity(tunnels.len());

        for tunnel in tunnels.values() {
            let config = tunnel.config.lock().await.clone();
            let lifecycle = tunnel.lifecycle.lock().await.clone();
            let queued_workers = tunnel.relay_workers.lock().await.len();
            let active_connections = tunnel.active_connections.load(Ordering::Relaxed);
            let total_connections = tunnel.total_connections.load(Ordering::Relaxed);
            let bytes_in = tunnel.bytes_in.load(Ordering::Relaxed);
            let bytes_out = tunnel.bytes_out.load(Ordering::Relaxed);
            let last_activity = tunnel.last_activity.load(Ordering::Relaxed);
            let http_requests = tunnel.http_requests.load(Ordering::Relaxed);
            let http_errors = tunnel.http_errors.load(Ordering::Relaxed);
            let http_latency_ms = tunnel.http_latency_ms.load(Ordering::Relaxed);
            let http_bytes = tunnel.http_bytes.load(Ordering::Relaxed);
            let uptime_seconds =
                (Utc::now().timestamp_millis() - lifecycle.created_at).max(0) / 1000;

            tunnel_items.push(json!({
                "tunnelId": lifecycle.tunnel_id.clone(),
                "clientId": lifecycle.client_id.clone(),
                "sessionId": lifecycle.session_id.clone(),
                "status": lifecycle.status.as_str(),
                "protocol": config.protocol.clone(),
                "remotePort": config.remote_port,
                "localTarget": format!("{}:{}", config.local_host, config.local_port),
                "host": config.host.clone(),
                "path": config.path.clone(),
                "enabled": config.enabled,
                "relayWorkers": queued_workers,
                "activeConnections": active_connections,
                "totalConnections": total_connections,
                "connectionsTotal": total_connections,
                "active_connections": active_connections,
                "connections_total": total_connections,
                "failedConnections": tunnel.failed_connections.load(Ordering::Relaxed),
                "bytesIn": bytes_in,
                "bytesOut": bytes_out,
                "bytes_in": bytes_in,
                "bytes_out": bytes_out,
                "lastActivity": last_activity,
                "last_activity": last_activity,
                "uptimeSeconds": uptime_seconds,
                "errors": tunnel.errors.load(Ordering::Relaxed),
                "http": {
                    "requestsTotal": http_requests,
                    "activeRequests": if is_http_tunnel_protocol(&config.protocol) { active_connections } else { 0 },
                    "errorCount": http_errors,
                    "errorRate": ratio_u64(http_errors, http_requests),
                    "latencyMs": {
                        "total": http_latency_ms,
                        "average": average_u64(http_latency_ms, http_requests)
                    },
                    "bandwidth": {
                        "bytes": http_bytes
                    }
                },
                "createdAt": lifecycle.created_at,
                "lastHeartbeat": lifecycle.last_heartbeat,
                "updatedAt": config.updated_at,
                "runtime": {
                    "maxConnections": config.runtime.max_connections,
                    "idleTimeoutMs": config.runtime.idle_timeout_ms,
                    "bandwidthLimitBps": config.runtime.bandwidth_limit_bps
                }
            }));
        }
        let access_logs = self
            .inner
            .http_access_logs
            .lock()
            .await
            .iter()
            .cloned()
            .collect::<Vec<_>>();
        let status_codes = self.inner.http_status_codes.lock().await.clone();
        let http_requests_total = self
            .inner
            .http_counters
            .requests_total
            .load(Ordering::Relaxed);
        let http_latency_total = self
            .inner
            .http_counters
            .latency_total_ms
            .load(Ordering::Relaxed);

        json!({
            "sessions": sessions.len(),
            "onlineSessions": sessions.values().filter(|session| session.connected).count(),
            "activeSessions": sessions.values().filter(|session| session.connected).count(),
            "failedSessions": self.inner.counters.failed_sessions.load(Ordering::Relaxed),
            "recoveredSessions": self.inner.counters.recovered_sessions.load(Ordering::Relaxed),
            "sessionItems": session_items,
            "tunnels": tunnel_items,
            "registeredTunnels": self.inner.counters.registered_tunnels.load(Ordering::Relaxed),
            "activeConnections": self.inner.counters.active_connections.load(Ordering::Relaxed),
            "totalConnections": self.inner.counters.total_connections.load(Ordering::Relaxed),
            "failedConnections": self.inner.counters.failed_connections.load(Ordering::Relaxed),
            "bytesIn": self.inner.counters.bytes_in.load(Ordering::Relaxed),
            "bytesOut": self.inner.counters.bytes_out.load(Ordering::Relaxed),
            "errors": self.inner.counters.errors.load(Ordering::Relaxed),
            "http": {
                "requestsTotal": http_requests_total,
                "activeRequests": self.inner.http_counters.active_requests.load(Ordering::Relaxed),
                "statusCodes": status_codes,
                "latency": {
                    "totalMs": http_latency_total,
                    "averageMs": average_u64(http_latency_total, http_requests_total)
                },
                "bandwidth": {
                    "bytes": self.inner.http_counters.bytes_total.load(Ordering::Relaxed)
                },
                "accessLogs": access_logs.into_iter().map(|log| json!({
                    "method": log.method,
                    "path": log.path,
                    "host": log.host,
                    "tunnelId": log.tunnel_id,
                    "status": log.status,
                    "latency": log.latency_ms,
                    "latencyMs": log.latency_ms,
                    "bytes": log.bytes,
                    "timestamp": log.timestamp_millis,
                    "scheme": log.scheme,
                    "tlsVersion": log.tls_version,
                    "sni": log.sni
                })).collect::<Vec<_>>()
            },
            "https": {
                "tls_handshake_total": self.inner.https_counters.tls_handshake_total.load(Ordering::Relaxed),
                "tls_errors": self.inner.https_counters.tls_errors.load(Ordering::Relaxed),
                "tls_version": self.latest_tls_version().await,
                "certificate_expire_days": self.certificate_expire_days(),
                "https_requests": self.inner.https_counters.https_requests.load(Ordering::Relaxed)
            }
        })
    }

    pub async fn capability_snapshot(&self) -> Value {
        let tunnels = self.inner.tunnels.lock().await;
        let mut ports = Vec::with_capacity(tunnels.len());
        for tunnel in tunnels.values() {
            ports.push(tunnel.config.lock().await.remote_port);
        }

        server_capability_snapshot(&ports)
    }

    async fn ensure_listener(
        &self,
        tunnel_id: String,
        config: &TunnelConfig,
    ) -> anyhow::Result<()> {
        if config.protocol == "https" {
            return self.ensure_https_listener(config.remote_port).await;
        }
        if config.protocol == "http" {
            return self.ensure_http_listener(config.remote_port).await;
        }

        self.ensure_tcp_listener(tunnel_id, config.remote_port)
            .await
    }

    async fn ensure_tcp_listener(&self, tunnel_id: String, remote_port: u16) -> anyhow::Result<()> {
        let mut listeners = self.inner.listeners.lock().await;
        if let Some(existing) = listeners.get(&remote_port) {
            if existing.protocol == ListenerProtocol::Tcp && existing.tunnel_id == tunnel_id {
                return Ok(());
            }
            return Err(anyhow::anyhow!(
                "remote port {remote_port} is already used by another tunnel"
            ));
        }

        let bind_addr = SocketAddr::new(self.inner.bind_ip, remote_port);
        let listener = TcpListener::bind(bind_addr).await?;
        let bound_addr = listener.local_addr()?;
        let gateway = self.clone();
        let listener_tunnel_id = tunnel_id.clone();
        let handle = tokio::spawn(async move {
            gateway
                .accept_public_connections(listener_tunnel_id, listener, bound_addr)
                .await;
        });

        listeners.insert(
            remote_port,
            ListenerHandle {
                tunnel_id,
                protocol: ListenerProtocol::Tcp,
                handle,
            },
        );
        info!(
            target: "gate_gateway",
            addr = %bound_addr,
            "Tunnel gateway public listener started"
        );
        Ok(())
    }

    async fn ensure_http_listener(&self, remote_port: u16) -> anyhow::Result<()> {
        let mut listeners = self.inner.listeners.lock().await;
        if let Some(existing) = listeners.get(&remote_port) {
            if existing.protocol == ListenerProtocol::Http {
                return Ok(());
            }
            return Err(anyhow::anyhow!(
                "remote port {remote_port} is already used by another listener"
            ));
        }

        let bind_addr = SocketAddr::new(self.inner.bind_ip, remote_port);
        let listener = TcpListener::bind(bind_addr).await?;
        let bound_addr = listener.local_addr()?;
        let gateway = self.clone();
        let handle = tokio::spawn(async move {
            gateway
                .accept_http_public_connections(remote_port, listener, bound_addr)
                .await;
        });

        listeners.insert(
            remote_port,
            ListenerHandle {
                tunnel_id: format!("http:{remote_port}"),
                protocol: ListenerProtocol::Http,
                handle,
            },
        );
        info!(
            target: "gate_gateway",
            addr = %bound_addr,
            "HTTP host router public listener started"
        );
        Ok(())
    }

    async fn ensure_https_listener(&self, remote_port: u16) -> anyhow::Result<()> {
        self.reload_tls_cache_from_store()?;
        if self.inner.tls_cache.is_empty() {
            return Err(anyhow::anyhow!(
                "HTTPS listener requires at least one loaded certificate"
            ));
        }

        let mut listeners = self.inner.listeners.lock().await;
        if let Some(existing) = listeners.get(&remote_port) {
            if existing.protocol == ListenerProtocol::Https {
                return Ok(());
            }
            return Err(anyhow::anyhow!(
                "remote port {remote_port} is already used by another listener"
            ));
        }

        let bind_addr = SocketAddr::new(self.inner.bind_ip, remote_port);
        let listener = TcpListener::bind(bind_addr).await?;
        let bound_addr = listener.local_addr()?;
        let acceptor = self.build_tls_acceptor()?;
        let gateway = self.clone();
        let handle = tokio::spawn(async move {
            gateway
                .accept_https_public_connections(remote_port, listener, acceptor, bound_addr)
                .await;
        });

        listeners.insert(
            remote_port,
            ListenerHandle {
                tunnel_id: format!("https:{remote_port}"),
                protocol: ListenerProtocol::Https,
                handle,
            },
        );
        info!(
            target: "gate_gateway",
            addr = %bound_addr,
            "HTTPS TLS listener public listener started"
        );
        Ok(())
    }

    async fn accept_public_connections(
        self,
        tunnel_id: String,
        listener: TcpListener,
        bound_addr: SocketAddr,
    ) {
        loop {
            match listener.accept().await {
                Ok((public_stream, remote_addr)) => {
                    let gateway = self.clone();
                    let tunnel_id = tunnel_id.clone();
                    tokio::spawn(async move {
                        if let Err(error) = gateway
                            .handle_public_connection(tunnel_id, public_stream, remote_addr)
                            .await
                        {
                            warn!(
                                target: "gate_gateway",
                                remote_addr = %remote_addr,
                                error = %error,
                                "Tunnel gateway forwarding failed"
                            );
                        }
                    });
                }
                Err(source) => {
                    warn!(
                        target: "gate_gateway",
                        addr = %bound_addr,
                        error = %source,
                        "Tunnel gateway accept failed"
                    );
                    break;
                }
            }
        }
    }

    async fn accept_http_public_connections(
        self,
        remote_port: u16,
        listener: TcpListener,
        bound_addr: SocketAddr,
    ) {
        loop {
            match listener.accept().await {
                Ok((public_stream, remote_addr)) => {
                    let gateway = self.clone();
                    tokio::spawn(async move {
                        if let Err(error) = gateway
                            .handle_http_public_connection(remote_port, public_stream, remote_addr)
                            .await
                        {
                            warn!(
                                target: "gate_gateway",
                                remote_addr = %remote_addr,
                                error = %error,
                                "HTTP host router forwarding failed"
                            );
                        }
                    });
                }
                Err(source) => {
                    warn!(
                        target: "gate_gateway",
                        addr = %bound_addr,
                        error = %source,
                        "HTTP host router accept failed"
                    );
                    break;
                }
            }
        }
    }

    async fn accept_https_public_connections(
        self,
        remote_port: u16,
        listener: TcpListener,
        acceptor: TlsAcceptor,
        bound_addr: SocketAddr,
    ) {
        loop {
            match listener.accept().await {
                Ok((public_stream, remote_addr)) => {
                    let gateway = self.clone();
                    let acceptor = acceptor.clone();
                    tokio::spawn(async move {
                        if let Err(error) = gateway
                            .handle_https_public_connection(
                                remote_port,
                                public_stream,
                                acceptor,
                                remote_addr,
                            )
                            .await
                        {
                            warn!(
                                target: "gate_gateway",
                                remote_addr = %remote_addr,
                                error = %error,
                                "HTTPS TLS listener forwarding failed"
                            );
                        }
                    });
                }
                Err(source) => {
                    warn!(
                        target: "gate_gateway",
                        addr = %bound_addr,
                        error = %source,
                        "HTTPS TLS listener accept failed"
                    );
                    break;
                }
            }
        }
    }

    async fn handle_http_public_connection(
        &self,
        remote_port: u16,
        public_stream: TcpStream,
        remote_addr: SocketAddr,
    ) -> anyhow::Result<()> {
        self.handle_http_public_stream(
            remote_port,
            public_stream,
            remote_addr,
            HttpGatewayConnection::plain(),
        )
        .await
    }

    async fn handle_https_public_connection(
        &self,
        remote_port: u16,
        public_stream: TcpStream,
        acceptor: TlsAcceptor,
        remote_addr: SocketAddr,
    ) -> anyhow::Result<()> {
        let _ = public_stream.set_nodelay(true);
        let started = Instant::now();
        let tls_stream = acceptor.accept(public_stream).await.map_err(|source| {
            self.inner
                .https_counters
                .tls_errors
                .fetch_add(1, Ordering::Relaxed);
            anyhow::anyhow!("TLS handshake failed: {source}")
        })?;
        self.inner
            .https_counters
            .tls_handshake_total
            .fetch_add(1, Ordering::Relaxed);
        let connection = tls_connection_metadata(&tls_stream);
        info!(
            target: "gate_gateway",
            remote_addr = %remote_addr,
            tls_version = ?connection.tls_version,
            sni = ?connection.sni,
            handshake_time_ms = elapsed_millis(started),
            "HTTPS TLS handshake succeeded"
        );

        self.handle_http_public_stream(remote_port, tls_stream, remote_addr, connection)
            .await
    }

    async fn handle_http_public_stream<S>(
        &self,
        remote_port: u16,
        mut public_stream: S,
        remote_addr: SocketAddr,
        connection: HttpGatewayConnection,
    ) -> anyhow::Result<()>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let _active = ActiveHttpGatewayRequest::new(Arc::clone(&self.inner));
        let started = Instant::now();
        let head_bytes = match read_http_gateway_head(&mut public_stream).await {
            Ok(Some(head)) => head,
            Ok(None) => return Ok(()),
            Err(source) => {
                let bytes =
                    write_http_gateway_error(&mut public_stream, 400, "Bad Request").await?;
                self.record_http_access(
                    None,
                    HttpAccessLog {
                        method: "BAD".to_string(),
                        path: String::new(),
                        host: String::new(),
                        tunnel_id: None,
                        status: 400,
                        latency_ms: elapsed_millis(started),
                        bytes,
                        timestamp_millis: Utc::now().timestamp_millis(),
                        scheme: connection.scheme,
                        tls_version: connection.tls_version.clone(),
                        sni: connection.sni.clone(),
                    },
                )
                .await;
                return Err(source.into());
            }
        };

        let request = match parse_http_gateway_request(&head_bytes) {
            Ok(request) => request,
            Err(source) => {
                let bytes =
                    write_http_gateway_error(&mut public_stream, 400, "Bad Request").await?;
                self.record_http_access(
                    None,
                    HttpAccessLog {
                        method: "BAD".to_string(),
                        path: String::new(),
                        host: String::new(),
                        tunnel_id: None,
                        status: 400,
                        latency_ms: elapsed_millis(started),
                        bytes,
                        timestamp_millis: Utc::now().timestamp_millis(),
                        scheme: connection.scheme,
                        tls_version: connection.tls_version.clone(),
                        sni: connection.sni.clone(),
                    },
                )
                .await;
                return Err(anyhow::anyhow!(source));
            }
        };

        if let Some(bytes) = self
            .try_serve_acme_http01(&mut public_stream, &request)
            .await?
        {
            self.record_http_access(
                None,
                HttpAccessLog {
                    method: request.method,
                    path: request.path,
                    host: request.host,
                    tunnel_id: None,
                    status: 200,
                    latency_ms: elapsed_millis(started),
                    bytes,
                    timestamp_millis: Utc::now().timestamp_millis(),
                    scheme: connection.scheme,
                    tls_version: connection.tls_version.clone(),
                    sni: connection.sni.clone(),
                },
            )
            .await;
            return Ok(());
        }

        let route_host = if request.host.is_empty() {
            connection.sni.as_deref().unwrap_or_default()
        } else {
            request.host.as_str()
        };
        let Some(tunnel_id) = self
            .resolve_http_tunnel_id(remote_port, route_host, &request.path)
            .await
        else {
            let bytes = write_http_gateway_error(&mut public_stream, 404, "Not Found").await?;
            self.record_http_access(
                None,
                HttpAccessLog {
                    method: request.method,
                    path: request.path,
                    host: request.host,
                    tunnel_id: None,
                    status: 404,
                    latency_ms: elapsed_millis(started),
                    bytes,
                    timestamp_millis: Utc::now().timestamp_millis(),
                    scheme: connection.scheme,
                    tls_version: connection.tls_version.clone(),
                    sni: connection.sni.clone(),
                },
            )
            .await;
            return Ok(());
        };

        let tunnel = {
            let tunnels = self.inner.tunnels.lock().await;
            tunnels
                .get(&tunnel_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("HTTP tunnel is not registered"))?
        };
        let config = tunnel.config.lock().await.clone();
        if !config.enabled {
            self.record_connection_error(&tunnel).await;
            let bytes =
                write_http_gateway_error(&mut public_stream, 503, "Service Unavailable").await?;
            self.record_http_access(
                Some(&tunnel),
                HttpAccessLog {
                    method: request.method,
                    path: request.path,
                    host: request.host,
                    tunnel_id: Some(tunnel_id),
                    status: 503,
                    latency_ms: elapsed_millis(started),
                    bytes,
                    timestamp_millis: Utc::now().timestamp_millis(),
                    scheme: connection.scheme,
                    tls_version: connection.tls_version.clone(),
                    sni: connection.sni.clone(),
                },
            )
            .await;
            return Ok(());
        }

        tunnel.total_connections.fetch_add(1, Ordering::Relaxed);
        self.inner
            .counters
            .total_connections
            .fetch_add(1, Ordering::Relaxed);

        if !try_acquire_connection(&tunnel.active_connections, config.runtime.max_connections) {
            self.record_connection_error(&tunnel).await;
            let bytes =
                write_http_gateway_error(&mut public_stream, 503, "Service Unavailable").await?;
            self.record_http_access(
                Some(&tunnel),
                HttpAccessLog {
                    method: request.method,
                    path: request.path,
                    host: request.host,
                    tunnel_id: Some(tunnel_id),
                    status: 503,
                    latency_ms: elapsed_millis(started),
                    bytes,
                    timestamp_millis: Utc::now().timestamp_millis(),
                    scheme: connection.scheme,
                    tls_version: connection.tls_version.clone(),
                    sni: connection.sni.clone(),
                },
            )
            .await;
            return Ok(());
        }
        self.inner
            .counters
            .active_connections
            .fetch_add(1, Ordering::Relaxed);
        tunnel
            .last_activity
            .store(now_millis_u64(), Ordering::Relaxed);

        let relay_head_bytes = rewrite_http_request_path(&head_bytes, config.path.as_deref())
            .unwrap_or_else(|_| head_bytes.clone());

        let mut relay_stream = match self.next_relay_worker(&tunnel).await {
            Ok(stream) => stream,
            Err(error) => {
                warn!(
                    target: "gate_gateway",
                    tunnel_id = %tunnel_id,
                    host = %request.host,
                    error = %error,
                    "HTTP tunnel relay worker is unavailable"
                );
                tunnel.active_connections.fetch_sub(1, Ordering::Relaxed);
                self.inner
                    .counters
                    .active_connections
                    .fetch_sub(1, Ordering::Relaxed);
                self.record_connection_error(&tunnel).await;
                let bytes =
                    write_http_gateway_error(&mut public_stream, 502, "Bad Gateway").await?;
                self.record_http_access(
                    Some(&tunnel),
                    HttpAccessLog {
                        method: request.method,
                        path: request.path,
                        host: request.host,
                        tunnel_id: Some(tunnel_id),
                        status: 502,
                        latency_ms: elapsed_millis(started),
                        bytes,
                        timestamp_millis: Utc::now().timestamp_millis(),
                        scheme: connection.scheme,
                        tls_version: connection.tls_version.clone(),
                        sni: connection.sni.clone(),
                    },
                )
                .await;
                return Ok(());
            }
        };

        let relay_result = async {
            let protocol = ProtocolBuilder::new().build();
            let start = Message::request(
                Command::TunnelRelayStart,
                Body::Json(json!({
                    "relayId": Uuid::new_v4().to_string(),
                    "tunnelId": config.tunnel_id.clone(),
                    "protocol": config.protocol.clone(),
                    "remoteAddr": remote_addr.to_string(),
                    "localHost": config.local_host.clone(),
                    "localPort": config.local_port,
                    "host": config.host.clone(),
                    "path": config.path.clone(),
                    "metadata": config.metadata.clone()
                })),
                Metadata::default(),
            );
            write_message(&mut relay_stream, &protocol, &start).await?;
            relay_stream.write_all(&relay_head_bytes).await?;
            record_tunnel_bytes(
                &tunnel,
                &self.inner.counters,
                CopyDirection::BytesIn,
                relay_head_bytes.len() as u64,
            );

            let mut outcome = self
                .relay_bidirectional(public_stream, relay_stream, &tunnel, config.runtime)
                .await?;
            outcome.bytes_in = outcome.bytes_in.saturating_add(relay_head_bytes.len() as u64);
            Ok::<RelayCopyOutcome, anyhow::Error>(outcome)
        }
        .await;

        tunnel.active_connections.fetch_sub(1, Ordering::Relaxed);
        self.inner
            .counters
            .active_connections
            .fetch_sub(1, Ordering::Relaxed);

        match relay_result {
            Ok(outcome) => {
                self.record_http_access(
                    Some(&tunnel),
                    HttpAccessLog {
                        method: request.method,
                        path: request.path,
                        host: request.host,
                        tunnel_id: Some(tunnel_id),
                        status: 200,
                        latency_ms: elapsed_millis(started),
                        bytes: outcome.bytes_in.saturating_add(outcome.bytes_out),
                        timestamp_millis: Utc::now().timestamp_millis(),
                        scheme: connection.scheme,
                        tls_version: connection.tls_version.clone(),
                        sni: connection.sni.clone(),
                    },
                )
                .await;
                Ok(())
            }
            Err(error) => {
                self.record_connection_error(&tunnel).await;
                self.record_http_access(
                    Some(&tunnel),
                    HttpAccessLog {
                        method: request.method,
                        path: request.path,
                        host: request.host,
                        tunnel_id: Some(tunnel_id),
                        status: 502,
                        latency_ms: elapsed_millis(started),
                        bytes: head_bytes.len() as u64,
                        timestamp_millis: Utc::now().timestamp_millis(),
                        scheme: connection.scheme,
                        tls_version: connection.tls_version.clone(),
                        sni: connection.sni.clone(),
                    },
                )
                .await;
                Err(error)
            }
        }
    }

    async fn handle_public_connection(
        &self,
        tunnel_id: String,
        public_stream: TcpStream,
        remote_addr: SocketAddr,
    ) -> anyhow::Result<()> {
        let tunnel = {
            let tunnels = self.inner.tunnels.lock().await;
            tunnels
                .get(&tunnel_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("tunnel is not registered"))?
        };
        let config = tunnel.config.lock().await.clone();
        if !config.enabled {
            self.record_connection_error(&tunnel).await;
            return Err(anyhow::anyhow!("tunnel is stopped"));
        }

        tunnel.total_connections.fetch_add(1, Ordering::Relaxed);
        self.inner
            .counters
            .total_connections
            .fetch_add(1, Ordering::Relaxed);

        if !try_acquire_connection(&tunnel.active_connections, config.runtime.max_connections) {
            self.record_connection_error(&tunnel).await;
            return Err(anyhow::anyhow!(
                "tunnel connection limit reached: max_connections={}",
                config.runtime.max_connections
            ));
        }
        self.inner
            .counters
            .active_connections
            .fetch_add(1, Ordering::Relaxed);
        tunnel
            .last_activity
            .store(now_millis_u64(), Ordering::Relaxed);

        let result = async {
            let mut relay_stream = self.next_relay_worker(&tunnel).await?;
            let protocol = ProtocolBuilder::new().build();
            let start = Message::request(
                Command::TunnelRelayStart,
                Body::Json(json!({
                    "relayId": Uuid::new_v4().to_string(),
                    "tunnelId": config.tunnel_id.clone(),
                    "protocol": config.protocol.clone(),
                    "remoteAddr": remote_addr.to_string(),
                    "localHost": config.local_host.clone(),
                    "localPort": config.local_port,
                    "host": config.host.clone(),
                    "path": config.path.clone(),
                    "metadata": config.metadata.clone()
                })),
                Metadata::default(),
            );
            write_message(&mut relay_stream, &protocol, &start).await?;

            info!(
                target: "gate_gateway",
                tunnel_id = %tunnel_id,
                remote_addr = %remote_addr,
                "Tunnel relay forwarding started"
            );

            let outcome = self
                .relay_bidirectional(public_stream, relay_stream, &tunnel, config.runtime)
                .await?;
            info!(
                target: "gate_gateway",
                tunnel_id = %tunnel_id,
                remote_addr = %remote_addr,
                upload_bytes = outcome.bytes_in,
                download_bytes = outcome.bytes_out,
                duration_ms = outcome.duration_ms,
                "Tunnel relay forwarding completed"
            );
            Ok::<(), anyhow::Error>(())
        }
        .await;

        tunnel.active_connections.fetch_sub(1, Ordering::Relaxed);
        self.inner
            .counters
            .active_connections
            .fetch_sub(1, Ordering::Relaxed);

        if result.is_err() {
            self.record_connection_error(&tunnel).await;
        }

        result
    }

    async fn relay_bidirectional<S>(
        &self,
        public_stream: S,
        mut relay_stream: TcpStream,
        tunnel: &Arc<TunnelSession>,
        runtime: TunnelRuntimeConfig,
    ) -> anyhow::Result<RelayCopyOutcome>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let _ = relay_stream.set_nodelay(true);

        let started = Instant::now();
        let idle_timeout = Duration::from_millis(runtime.idle_timeout_ms);
        let (mut public_read, mut public_write) = tokio::io::split(public_stream);
        let (mut relay_read, mut relay_write) = relay_stream.split();

        // 两个方向独立复制；任一方向 EOF 后只关闭对端写半边，保留另一方向继续排空。
        let upload = copy_direction(
            &mut public_read,
            &mut relay_write,
            tunnel,
            &self.inner.counters,
            CopyDirection::BytesIn,
            idle_timeout,
            runtime.bandwidth_limit_bps,
        );
        let download = copy_direction(
            &mut relay_read,
            &mut public_write,
            tunnel,
            &self.inner.counters,
            CopyDirection::BytesOut,
            idle_timeout,
            runtime.bandwidth_limit_bps,
        );
        tokio::pin!(upload);
        tokio::pin!(download);

        let mut bytes_in = None;
        let mut bytes_out = None;
        loop {
            tokio::select! {
                result = &mut upload, if bytes_in.is_none() => {
                    bytes_in = Some(result.map_err(|source| {
                        anyhow::anyhow!(describe_io_error("upload", &source))
                    })?);
                }
                result = &mut download, if bytes_out.is_none() => {
                    bytes_out = Some(result.map_err(|source| {
                        anyhow::anyhow!(describe_io_error("download", &source))
                    })?);
                }
            }

            if bytes_in.is_some() && bytes_out.is_some() {
                break;
            }
        }

        Ok(RelayCopyOutcome {
            bytes_in: bytes_in.unwrap_or_default(),
            bytes_out: bytes_out.unwrap_or_default(),
            duration_ms: started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
        })
    }

    async fn next_relay_worker(&self, tunnel: &Arc<TunnelSession>) -> anyhow::Result<TcpStream> {
        tunnel
            .relay_workers
            .lock()
            .await
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("no relay worker is available for tunnel"))
    }

    async fn record_connection_error(&self, tunnel: &Arc<TunnelSession>) {
        tunnel.failed_connections.fetch_add(1, Ordering::Relaxed);
        tunnel.errors.fetch_add(1, Ordering::Relaxed);
        tunnel
            .last_activity
            .store(now_millis_u64(), Ordering::Relaxed);
        self.inner
            .counters
            .failed_connections
            .fetch_add(1, Ordering::Relaxed);
        self.inner.counters.errors.fetch_add(1, Ordering::Relaxed);
    }

    pub async fn create_domain(&self, body: &Value) -> anyhow::Result<Value> {
        let host = required_string(body, &["host", "domain"])?;
        let tunnel_id = optional_string_any(body, &["tunnelId", "tunnel_id"]);
        let repository = self.domain_repository()?;
        let host = ManagedHost::new(&host).map_err(|error| anyhow::anyhow!(error.to_string()))?;
        let mut builder = ManagedDomain::builder(domain_id_for_host(host.as_str())?, host)
            .record_type(ManagedRecordType::A)
            .status(ManagedDomainStatus::Active);
        if let Some(tunnel_id) = tunnel_id {
            builder = builder.tunnel_id(
                ManagedTunnelId::new(tunnel_id)
                    .map_err(|error| anyhow::anyhow!(error.to_string()))?,
            );
        }
        let domain = builder
            .build()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        let domain = repository
            .create(domain)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        Ok(domain_json(&domain))
    }

    pub async fn bind_domain(&self, body: &Value) -> anyhow::Result<Value> {
        let host = required_string(body, &["host", "domain"])?;
        let tunnel_id = required_string(body, &["tunnelId", "tunnel_id"])?;
        let domain = self.bind_domain_to_tunnel(&host, &tunnel_id)?;
        Ok(domain_json(&domain))
    }

    pub async fn unbind_domain(&self, body: &Value) -> anyhow::Result<Value> {
        let host = required_string(body, &["host", "domain"])?;
        let repository = self.domain_repository()?;
        let host = ManagedHost::new(&host).map_err(|error| anyhow::anyhow!(error.to_string()))?;
        let domain = repository
            .find_by_host(&host)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?
            .ok_or_else(|| anyhow::anyhow!("domain not found: {}", host.as_str()))?;
        let domain = repository
            .unbind_tunnel(domain.id())
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        Ok(domain_json(&domain))
    }

    pub async fn delete_domain(&self, body: &Value) -> anyhow::Result<Value> {
        let host = required_string(body, &["host", "domain"])?;
        let repository = self.domain_repository()?;
        let host = ManagedHost::new(&host).map_err(|error| anyhow::anyhow!(error.to_string()))?;
        let domain = repository
            .find_by_host(&host)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?
            .ok_or_else(|| anyhow::anyhow!("domain not found: {}", host.as_str()))?;
        let domain = repository
            .delete(domain.id())
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        Ok(domain_json(&domain))
    }

    fn domain_repository(&self) -> anyhow::Result<&SqliteDomainRepository> {
        self.inner
            .domain_repository
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("domain repository is unavailable"))
    }

    async fn bind_configured_domain(&self, config: &TunnelConfig) -> anyhow::Result<()> {
        if self.inner.domain_repository.is_none() {
            return Ok(());
        }
        let Some(host) = config.host.as_deref() else {
            return Ok(());
        };
        if host.trim().is_empty() {
            return Ok(());
        }
        self.bind_domain_to_tunnel(host, &config.tunnel_id)?;
        Ok(())
    }

    async fn ensure_https_certificate(&self, config: &TunnelConfig) -> anyhow::Result<()> {
        let host = config
            .host
            .as_deref()
            .map(normalize_http_host)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| anyhow::anyhow!("HTTPS tunnel requires a bound domain"))?;

        if let Ok(stored) = self.inner.certificate_store.load(&host) {
            CertificateValidator::default().validate_record(&stored.record, &host)?;
            self.inner.tls_cache.upsert(stored)?;
            return Ok(());
        }

        if self.acme_auto_enabled() && self.acme_http01_available().await {
            self.request_and_store_certificate(&host).await?;
            self.reload_tls_cache_from_store()?;
            return Ok(());
        }

        let reason = if self.acme_auto_enabled() {
            "certificate is missing and ACME HTTP-01 cannot bind its verification port; \
             upload a certificate manually, use an HTTP tunnel on a high port, or free port 80 for ACME"
        } else {
            "certificate is missing and ACME is not configured; upload a certificate or use HTTP on a high port"
        };
        self.save_certificate_error_record(&host, reason)?;
        Err(anyhow::anyhow!("{reason}"))
    }

    async fn request_and_store_certificate(&self, domain: &str) -> anyhow::Result<()> {
        let contact = self
            .acme_contact()
            .ok_or_else(|| anyhow::anyhow!("GATE_ACME_EMAIL is required for ACME issuance"))?;
        let acme_port = acme_http_port();
        if acme_port != 0 {
            self.ensure_http_listener(acme_port).await?;
        }

        let provider = acme_provider();
        let issued = provider
            .request_certificate_http01(
                AcmeCertificateRequest {
                    domain: domain.to_string(),
                    san: vec![domain.to_string()],
                    challenge_type: ChallengeType::Http01,
                    preferred_chain: None,
                },
                contact,
                self.inner.acme_challenges.as_ref(),
            )
            .await
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;

        let mut record = CertificateParser::parse_pem(domain, &issued.certificate_pem)?;
        record.renew_time = Some(Utc::now());
        record.status = CertificateStatus::Active;
        record.last_error = None;
        self.inner.certificate_store.save(&StoredCertificate {
            record,
            certificate_pem: issued.certificate_pem,
            private_key_pem: issued.private_key_pem,
        })?;
        Ok(())
    }

    fn reload_tls_cache_from_store(&self) -> anyhow::Result<()> {
        let mut certificates = Vec::new();
        for record in self.inner.certificate_store.list()? {
            match self.inner.certificate_store.load(&record.domain) {
                Ok(stored) => {
                    if let Err(error) = CertificateValidator::default()
                        .validate_record(&stored.record, &record.domain)
                    {
                        warn!(
                            target: "gate_gateway",
                            domain = %record.domain,
                            error = %error,
                            "HTTPS certificate validation failed; skipped loading"
                        );
                        continue;
                    }
                    certificates.push(stored);
                }
                Err(error) => {
                    warn!(
                        target: "gate_gateway",
                        domain = %record.domain,
                        error = %error,
                        "HTTPS certificate material missing; skipped loading"
                    );
                }
            }
        }
        self.inner.tls_cache.replace_all(certificates)?;
        Ok(())
    }

    fn build_tls_acceptor(&self) -> anyhow::Result<TlsAcceptor> {
        let resolver = Arc::new(GatewaySniResolver {
            cache: Arc::clone(&self.inner.tls_cache),
        });
        let provider = ring::default_provider();
        let builder = ServerConfig::builder_with_provider(Arc::new(provider))
            .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])?;
        let mut config = builder.with_no_client_auth().with_cert_resolver(resolver);
        config.alpn_protocols = vec![b"http/1.1".to_vec()];
        Ok(TlsAcceptor::from(Arc::new(config)))
    }

    async fn try_serve_acme_http01<S>(
        &self,
        stream: &mut S,
        request: &HttpGatewayRequest,
    ) -> anyhow::Result<Option<u64>>
    where
        S: AsyncWrite + Unpin,
    {
        let Some(key_authorization) = self
            .inner
            .acme_challenges
            .lookup(&request.host, &request.path)
        else {
            return Ok(None);
        };

        let body = format!("{key_authorization}\n");
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nContent-Type: text/plain\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).await?;
        stream.flush().await?;
        Ok(Some(response.len() as u64))
    }

    async fn acme_http01_available(&self) -> bool {
        let port = acme_http_port();
        if port == 0 {
            return false;
        }
        can_bind_public_port(self.inner.bind_ip, port).await
    }

    fn acme_auto_enabled(&self) -> bool {
        let explicit = std::env::var("GATE_ACME_AUTO")
            .ok()
            .map(|value| value.eq_ignore_ascii_case("true") || value == "1");
        explicit.unwrap_or_else(|| self.acme_contact().is_some())
    }

    fn acme_contact(&self) -> Option<AcmeAccountContact> {
        std::env::var("GATE_ACME_EMAIL")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .map(|email| AcmeAccountContact { email })
    }

    fn save_certificate_error_record(&self, domain: &str, reason: &str) -> anyhow::Result<()> {
        let mut record = self
            .inner
            .certificate_store
            .query(domain)?
            .unwrap_or_else(|| failed_certificate_record(domain));
        record.status = CertificateStatus::Failed;
        record.renew_time = Some(Utc::now());
        record.last_error = Some(reason.to_string());
        self.inner.certificate_store.save_record(&record)?;
        Ok(())
    }

    async fn latest_tls_version(&self) -> Option<String> {
        self.inner
            .http_access_logs
            .lock()
            .await
            .iter()
            .rev()
            .find_map(|log| log.tls_version.clone())
    }

    fn certificate_expire_days(&self) -> HashMap<String, i64> {
        self.inner.tls_cache.certificate_expire_days()
    }

    async fn run_certificate_renew_cycle(&self) {
        let certificates = match self.inner.certificate_store.list() {
            Ok(certificates) => certificates,
            Err(error) => {
                warn!(
                    target: "gate_gateway",
                    error = %error,
                    "HTTPS certificate renewal scan failed"
                );
                return;
            }
        };

        let scheduler = RenewScheduler::new(self.inner.renew_config);
        let report = scheduler.execute_due(&certificates, Utc::now(), self).await;
        for attempt in report.attempts {
            match attempt.status {
                gate_server_tls::renew::RenewAttemptStatus::Success => {
                    info!(
                        target: "gate_gateway",
                        domain = %attempt.domain,
                        "HTTPS certificate renewal succeeded"
                    );
                }
                gate_server_tls::renew::RenewAttemptStatus::Failed => {
                    let error = attempt.error.unwrap_or_else(|| "renew failed".to_string());
                    let _ = self.save_certificate_error_record(&attempt.domain, &error);
                    warn!(
                        target: "gate_gateway",
                        domain = %attempt.domain,
                        error = %error,
                        "HTTPS certificate renewal failed"
                    );
                }
            }
        }
        let _ = self.reload_tls_cache_from_store();
    }

    fn release_domains_for_tunnel(&self, tunnel_id: &str) -> anyhow::Result<()> {
        let Some(repository) = &self.inner.domain_repository else {
            return Ok(());
        };
        let tunnel_id = ManagedTunnelId::new(tunnel_id)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        let bound = repository
            .find_by_tunnel(&tunnel_id)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        for domain in bound {
            if let Ok(current) = repository
                .find_by_id(domain.id())
                .map_err(|error| anyhow::anyhow!(error.to_string()))
            {
                if let Some(mut current) = current {
                    let _ = current.unbind();
                    let _ = repository.update(current);
                }
            }
        }
        Ok(())
    }

    fn bind_domain_to_tunnel(&self, host: &str, tunnel_id: &str) -> anyhow::Result<ManagedDomain> {
        let repository = self.domain_repository()?;
        let host = ManagedHost::new(host).map_err(|error| anyhow::anyhow!(error.to_string()))?;
        let tunnel_id =
            ManagedTunnelId::new(tunnel_id).map_err(|error| anyhow::anyhow!(error.to_string()))?;

        if let Some(existing) = repository
            .find_by_host(&host)
            .map_err(|error| anyhow::anyhow!(error.to_string()))?
        {
            if existing.tunnel_id() == Some(&tunnel_id) {
                return Ok(existing);
            }
            let mut domain = existing;
            domain
                .rebind(tunnel_id)
                .map_err(|error| anyhow::anyhow!(error.to_string()))?;
            return repository
                .update(domain)
                .map_err(|error| anyhow::anyhow!(error.to_string()));
        }

        let domain = ManagedDomain::builder(domain_id_for_host(host.as_str())?, host)
            .tunnel_id(tunnel_id)
            .record_type(ManagedRecordType::A)
            .status(ManagedDomainStatus::Active)
            .build()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        repository
            .create(domain)
            .map_err(|error| anyhow::anyhow!(error.to_string()))
    }

    async fn resolve_http_tunnel_id(
        &self,
        remote_port: u16,
        host: &str,
        request_path: &str,
    ) -> Option<String> {
        let normalized_host = normalize_http_host(host);
        if !normalized_host.is_empty() {
            if let Some(repository) = &self.inner.domain_repository {
                if let Ok(host) = ManagedHost::new(&normalized_host) {
                    if let Ok(Some(domain)) = repository.find_by_host(&host) {
                        if domain.is_enabled() {
                            if let Some(tunnel_id) = domain.tunnel_id() {
                                let tunnel_id = tunnel_id.as_str().to_string();
                                if self
                                    .http_tunnel_matches_request(
                                        &tunnel_id,
                                        remote_port,
                                        request_path,
                                    )
                                    .await
                                {
                                    return Some(tunnel_id);
                                }
                            }
                        }
                    }
                }
            }
        }

        let tunnels = self.inner.tunnels.lock().await;
        let mut matched = Vec::<(String, usize)>::new();
        for (tunnel_id, tunnel) in tunnels.iter() {
            let config = tunnel.config.lock().await.clone();
            if !config.enabled
                || config.remote_port != remote_port
                || !is_http_tunnel_protocol(&config.protocol)
            {
                continue;
            }

            let lifecycle = tunnel.lifecycle.lock().await.clone();
            if lifecycle.status.is_terminal() {
                continue;
            }

            if !http_path_matches(config.path.as_deref(), request_path) {
                continue;
            }

            let host_matches = config
                .host
                .as_deref()
                .map(|configured| http_host_matches(configured, &normalized_host))
                .unwrap_or(false);
            let has_workers = !tunnel.relay_workers.lock().await.is_empty();
            if host_matches {
                matched.push((tunnel_id.clone(), http_path_prefix_len(config.path.as_deref())));
                continue;
            }

            if !http_tunnel_is_routable(&lifecycle, has_workers) {
                continue;
            }

            if http_tunnel_host_unrestricted(config.host.as_deref()) {
                matched.push((tunnel_id.clone(), http_path_prefix_len(config.path.as_deref())));
            }
        }

        matched
            .into_iter()
            .max_by_key(|(_, score)| *score)
            .map(|(tunnel_id, _)| tunnel_id)
    }

    async fn purge_stale_http_tunnels_on_port(&self, active_tunnel_id: &str, remote_port: u16) {
        let stale_tunnel_ids = {
            let tunnels = self.inner.tunnels.lock().await;
            let mut stale_tunnel_ids = Vec::new();
            for (tunnel_id, tunnel) in tunnels.iter() {
                if tunnel_id == active_tunnel_id {
                    continue;
                }
                let config = tunnel.config.lock().await;
                if config.remote_port != remote_port
                    || !is_http_tunnel_protocol(&config.protocol)
                {
                    continue;
                }
                let lifecycle = tunnel.lifecycle.lock().await;
                if lifecycle.status.is_terminal()
                    || matches!(lifecycle.status, TunnelSessionStatus::Offline)
                {
                    stale_tunnel_ids.push(tunnel_id.clone());
                }
            }
            stale_tunnel_ids
        };

        for tunnel_id in stale_tunnel_ids {
            let _ = self.stop_tunnel(&tunnel_id).await;
        }
    }

    async fn http_tunnel_matches_request(
        &self,
        tunnel_id: &str,
        remote_port: u16,
        request_path: &str,
    ) -> bool {
        let tunnel = {
            let tunnels = self.inner.tunnels.lock().await;
            tunnels.get(tunnel_id).cloned()
        };
        let Some(tunnel) = tunnel else {
            return false;
        };
        let config = tunnel.config.lock().await;
        config.enabled
            && config.remote_port == remote_port
            && is_http_tunnel_protocol(&config.protocol)
            && http_path_matches(config.path.as_deref(), request_path)
    }

    async fn http_tunnel_matches_port(&self, tunnel_id: &str, remote_port: u16) -> bool {
        let tunnel = {
            let tunnels = self.inner.tunnels.lock().await;
            tunnels.get(tunnel_id).cloned()
        };
        let Some(tunnel) = tunnel else {
            return false;
        };
        let config = tunnel.config.lock().await;
        config.enabled
            && config.remote_port == remote_port
            && is_http_tunnel_protocol(&config.protocol)
    }

    async fn stop_listener_if_unused(&self, remote_port: u16) -> bool {
        let tunnels = self.inner.tunnels.lock().await;
        for tunnel in tunnels.values() {
            if tunnel.config.lock().await.remote_port == remote_port {
                return false;
            }
        }
        drop(tunnels);

        if let Some(listener) = self.inner.listeners.lock().await.remove(&remote_port) {
            listener.handle.abort();
            return true;
        }
        false
    }

    async fn record_http_access(&self, tunnel: Option<&Arc<TunnelSession>>, log: HttpAccessLog) {
        self.inner
            .http_counters
            .requests_total
            .fetch_add(1, Ordering::Relaxed);
        self.inner
            .http_counters
            .bytes_total
            .fetch_add(log.bytes, Ordering::Relaxed);
        self.inner
            .http_counters
            .latency_total_ms
            .fetch_add(log.latency_ms, Ordering::Relaxed);
        if log.scheme == "https" {
            self.inner
                .https_counters
                .https_requests
                .fetch_add(1, Ordering::Relaxed);
        }
        *self
            .inner
            .http_status_codes
            .lock()
            .await
            .entry(log.status)
            .or_insert(0) += 1;

        if let Some(tunnel) = tunnel {
            tunnel.http_requests.fetch_add(1, Ordering::Relaxed);
            tunnel.http_bytes.fetch_add(log.bytes, Ordering::Relaxed);
            tunnel
                .http_latency_ms
                .fetch_add(log.latency_ms, Ordering::Relaxed);
            if log.status >= 400 {
                tunnel.http_errors.fetch_add(1, Ordering::Relaxed);
            }
        }

        info!(
            target: "gate_http_access",
            method = %log.method,
            path = %log.path,
            host = %log.host,
            scheme = %log.scheme,
            tls_version = ?log.tls_version,
            sni = ?log.sni,
            status = log.status,
            latency_ms = log.latency_ms,
            bytes = log.bytes,
            tunnel_id = ?log.tunnel_id,
            "HTTP Access"
        );

        let mut logs = self.inner.http_access_logs.lock().await;
        logs.push_back(log);
        while logs.len() > HTTP_ACCESS_LOG_LIMIT {
            let _ = logs.pop_front();
        }
    }

    async fn mark_tunnel_status(&self, tunnel_id: &str, status: TunnelSessionStatus) {
        let tunnel = {
            let tunnels = self.inner.tunnels.lock().await;
            tunnels.get(tunnel_id).cloned()
        };

        if let Some(tunnel) = tunnel {
            let mut lifecycle = tunnel.lifecycle.lock().await;
            lifecycle.status = status;
            lifecycle.last_heartbeat = Utc::now().timestamp_millis();
        }
    }

    async fn client_id_for_session(&self, session_id: &str) -> Option<String> {
        self.inner
            .sessions
            .lock()
            .await
            .get(session_id)
            .map(|session| session.client_id.clone())
    }

    async fn register_tunnel_on_client_session(&self, session_id: &str, tunnel_id: &str) {
        if let Some(session) = self.inner.sessions.lock().await.get_mut(session_id) {
            if !session
                .registered_tunnels
                .iter()
                .any(|registered| registered == tunnel_id)
            {
                session.registered_tunnels.push(tunnel_id.to_string());
            }
        }
    }

    async fn recover_tunnels_for_session(&self, session_id: &str, client_id: &str, now: i64) {
        let tunnels = {
            let tunnels = self.inner.tunnels.lock().await;
            tunnels.values().cloned().collect::<Vec<_>>()
        };

        for tunnel in tunnels {
            let mut lifecycle = tunnel.lifecycle.lock().await;
            if lifecycle.session_id != session_id || lifecycle.status.is_terminal() {
                continue;
            }
            lifecycle.client_id = client_id.to_string();
            lifecycle.status = TunnelSessionStatus::Recovering;
            lifecycle.last_heartbeat = now;
        }
    }

    async fn apply_heartbeat_to_tunnels(
        &self,
        session_id: &str,
        client_id: &str,
        registered_tunnels: &[String],
        now: i64,
    ) {
        let tunnels = {
            let tunnels = self.inner.tunnels.lock().await;
            tunnels.values().cloned().collect::<Vec<_>>()
        };

        for tunnel in tunnels {
            let config = tunnel.config.lock().await.clone();
            if config.session_id != session_id {
                continue;
            }
            let heartbeat_matches = registered_tunnels.is_empty()
                || registered_tunnels
                    .iter()
                    .any(|registered| registered == &config.tunnel_id);
            let mut lifecycle = tunnel.lifecycle.lock().await;
            if lifecycle.status.is_terminal() {
                continue;
            }
            lifecycle.client_id = client_id.to_string();
            lifecycle.session_id = session_id.to_string();
            lifecycle.last_heartbeat = now;
            lifecycle.status = if heartbeat_matches && config.enabled {
                TunnelSessionStatus::Online
            } else {
                TunnelSessionStatus::Recovering
            };
        }
    }

    fn start_heartbeat_watcher(&self) {
        let Ok(handle) = tokio::runtime::Handle::try_current() else {
            return;
        };
        let inner = Arc::downgrade(&self.inner);
        handle.spawn(async move {
            loop {
                sleep(HEARTBEAT_SWEEP_INTERVAL).await;
                let Some(inner) = inner.upgrade() else {
                    break;
                };
                TunnelGateway { inner }.expire_heartbeat_timeouts().await;
            }
        });
    }

    fn start_certificate_renew_watcher(&self) {
        let Ok(handle) = tokio::runtime::Handle::try_current() else {
            return;
        };
        let inner = Arc::downgrade(&self.inner);
        handle.spawn(async move {
            loop {
                let Some(current) = inner.upgrade() else {
                    break;
                };
                let interval = current.renew_config.check_interval_seconds.max(60) as u64;
                drop(current);
                sleep(Duration::from_secs(interval)).await;
                let Some(current) = inner.upgrade() else {
                    break;
                };
                TunnelGateway { inner: current }
                    .run_certificate_renew_cycle()
                    .await;
            }
        });
    }

    pub async fn shutdown(&self) {
        // 关闭监听任务以打破 Gateway 与 JoinHandle 之间的持有环，并停止接收新连接。
        let listeners = {
            let mut listeners = self.inner.listeners.lock().await;
            std::mem::take(&mut *listeners)
        };
        for listener in listeners.into_values() {
            listener.handle.abort();
        }
    }
}

impl Default for TunnelGateway {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait::async_trait]
impl CertificateRenewer for TunnelGateway {
    async fn renew_certificate(&self, domain: &str) -> Result<(), String> {
        self.request_and_store_certificate(domain)
            .await
            .map_err(|error| error.to_string())
    }
}

#[derive(Debug, Default)]
struct GatewayHttp01Challenges {
    values: StdRwLock<HashMap<String, String>>,
}

impl GatewayHttp01Challenges {
    fn lookup(&self, domain: &str, path: &str) -> Option<String> {
        let token = path.strip_prefix(ACME_HTTP01_PREFIX)?;
        let key = acme_challenge_key(domain, token);
        self.values.read().ok()?.get(&key).cloned()
    }
}

#[async_trait::async_trait]
impl Http01ChallengeStore for GatewayHttp01Challenges {
    async fn put_http01_challenge(
        &self,
        domain: &str,
        token: &str,
        key_authorization: &str,
    ) -> Result<(), gate_server_tls::AcmeError> {
        self.values
            .write()
            .map_err(|error| gate_server_tls::AcmeError::Execution {
                provider: "letsencrypt".to_string(),
                reason: error.to_string(),
            })?
            .insert(
                acme_challenge_key(domain, token),
                key_authorization.to_string(),
            );
        Ok(())
    }

    async fn remove_http01_challenge(
        &self,
        domain: &str,
        token: &str,
    ) -> Result<(), gate_server_tls::AcmeError> {
        self.values
            .write()
            .map_err(|error| gate_server_tls::AcmeError::Execution {
                provider: "letsencrypt".to_string(),
                reason: error.to_string(),
            })?
            .remove(&acme_challenge_key(domain, token));
        Ok(())
    }
}

#[derive(Debug, Default)]
struct GatewayTlsCertificateCache {
    entries: StdRwLock<HashMap<String, Arc<GatewayTlsCertificateEntry>>>,
}

impl GatewayTlsCertificateCache {
    fn is_empty(&self) -> bool {
        self.entries
            .read()
            .map(|entries| entries.is_empty())
            .unwrap_or(true)
    }

    fn replace_all(&self, certificates: Vec<StoredCertificate>) -> anyhow::Result<()> {
        let mut next = HashMap::new();
        for certificate in certificates {
            insert_tls_certificate(&mut next, certificate)?;
        }
        *self
            .entries
            .write()
            .map_err(|error| anyhow::anyhow!(error.to_string()))? = next;
        Ok(())
    }

    fn upsert(&self, certificate: StoredCertificate) -> anyhow::Result<()> {
        let mut entries = self
            .entries
            .write()
            .map_err(|error| anyhow::anyhow!(error.to_string()))?;
        insert_tls_certificate(&mut entries, certificate)
    }

    fn resolve(&self, server_name: Option<&str>) -> Option<Arc<GatewayTlsCertificateEntry>> {
        let entries = self.entries.read().ok()?;
        if let Some(server_name) = server_name {
            let server_name = normalize_http_host(server_name);
            if let Some(entry) = entries.get(&server_name) {
                return Some(Arc::clone(entry));
            }
            if let Some(wildcard) = wildcard_name(&server_name) {
                if let Some(entry) = entries.get(&wildcard) {
                    return Some(Arc::clone(entry));
                }
            }
        }
        entries.values().next().cloned()
    }

    fn certificate_expire_days(&self) -> HashMap<String, i64> {
        let Ok(entries) = self.entries.read() else {
            return HashMap::new();
        };
        let mut seen = HashSet::new();
        let mut values = HashMap::new();
        for entry in entries.values() {
            let fingerprint = entry.record.fingerprint.sha256.clone();
            if !seen.insert(fingerprint) {
                continue;
            }
            values.insert(
                entry.record.domain.clone(),
                entry
                    .record
                    .expire_time
                    .signed_duration_since(Utc::now())
                    .num_days(),
            );
        }
        values
    }
}

#[derive(Debug)]
struct GatewayTlsCertificateEntry {
    record: CertificateRecord,
    key: Arc<CertifiedKey>,
}

#[derive(Debug)]
struct GatewaySniResolver {
    cache: Arc<GatewayTlsCertificateCache>,
}

impl ResolvesServerCert for GatewaySniResolver {
    fn resolve(&self, client_hello: ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
        let server_name = client_hello.server_name().map(normalize_http_host);
        self.cache
            .resolve(server_name.as_deref())
            .map(|entry| Arc::clone(&entry.key))
    }
}

fn insert_tls_certificate(
    entries: &mut HashMap<String, Arc<GatewayTlsCertificateEntry>>,
    certificate: StoredCertificate,
) -> anyhow::Result<()> {
    let key = parse_certified_key(&certificate)?;
    let entry = Arc::new(GatewayTlsCertificateEntry {
        record: certificate.record,
        key,
    });
    for name in certificate_names(&entry.record) {
        entries.insert(name, Arc::clone(&entry));
    }
    Ok(())
}

fn parse_certified_key(certificate: &StoredCertificate) -> anyhow::Result<Arc<CertifiedKey>> {
    let mut cert_reader = Cursor::new(certificate.certificate_pem.as_bytes());
    let cert_chain = rustls_pemfile::certs(&mut cert_reader).collect::<Result<Vec<_>, _>>()?;
    if cert_chain.is_empty() {
        return Err(anyhow::anyhow!(
            "certificate chain for `{}` is empty",
            certificate.record.domain
        ));
    }

    let mut key_reader = Cursor::new(certificate.private_key_pem.as_bytes());
    let private_key = rustls_pemfile::private_key(&mut key_reader)?
        .ok_or_else(|| anyhow::anyhow!("private key is missing"))?;
    let signing_key = ring::sign::any_supported_type(&private_key)?;
    Ok(Arc::new(CertifiedKey::new(cert_chain, signing_key)))
}

fn certificate_names(record: &CertificateRecord) -> Vec<String> {
    let mut names = Vec::new();
    names.push(normalize_http_host(&record.domain));
    for name in &record.san {
        let name = normalize_http_host(name);
        if !names.iter().any(|existing| existing == &name) {
            names.push(name);
        }
    }
    names
}

fn wildcard_name(host: &str) -> Option<String> {
    let (_, suffix) = host.split_once('.')?;
    if suffix.is_empty() {
        None
    } else {
        Some(format!("*.{suffix}"))
    }
}

fn tls_connection_metadata(stream: &TlsStream<TcpStream>) -> HttpGatewayConnection {
    let (_, connection) = stream.get_ref();
    HttpGatewayConnection::tls(
        connection.protocol_version().map(protocol_version_name),
        connection.server_name().map(ToString::to_string),
    )
}

fn protocol_version_name(version: ProtocolVersion) -> String {
    match version {
        ProtocolVersion::TLSv1_2 => "TLSv1.2".to_string(),
        ProtocolVersion::TLSv1_3 => "TLSv1.3".to_string(),
        other => format!("{other:?}"),
    }
}

fn acme_challenge_key(domain: &str, token: &str) -> String {
    format!("{}:{token}", normalize_http_host(domain))
}

fn acme_provider() -> LetsEncryptProvider {
    if let Some(url) = std::env::var("GATE_ACME_DIRECTORY_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
    {
        return LetsEncryptProvider::with_directory_url(url);
    }
    if std::env::var("GATE_ACME_STAGING")
        .ok()
        .map(|value| value.eq_ignore_ascii_case("true") || value == "1")
        .unwrap_or(false)
    {
        return LetsEncryptProvider::staging();
    }
    LetsEncryptProvider::default()
}

fn acme_http_port() -> u16 {
    std::env::var("GATE_ACME_HTTP01_PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(0)
}

fn default_public_port(protocol: &str) -> u16 {
    match protocol {
        "https" => DEFAULT_HTTPS_PUBLIC_PORT,
        "http" => DEFAULT_HTTP_PUBLIC_PORT,
        _ => 0,
    }
}

fn align_tunnel_public_port(protocol: &str, remote_port: u16) -> u16 {
    if !matches!(protocol, "http" | "https") {
        return remote_port;
    }
    if remote_port == 0 {
        return default_public_port(protocol);
    }
    if matches!(remote_port, 80 | 443) {
        return default_public_port(protocol);
    }
    remote_port
}

fn normalize_http_tunnel_port(protocol: &str, remote_port: u16) -> u16 {
    align_tunnel_public_port(protocol, remote_port)
}

async fn can_bind_public_port(bind_ip: IpAddr, port: u16) -> bool {
    if port == 0 {
        return false;
    }
    match TcpListener::bind(SocketAddr::new(bind_ip, port)).await {
        Ok(listener) => {
            drop(listener);
            true
        }
        Err(_) => false,
    }
}

fn failed_certificate_record(domain: &str) -> CertificateRecord {
    let now = Utc::now();
    CertificateRecord {
        domain: domain.to_string(),
        issuer: "unknown".to_string(),
        expire_time: now,
        create_time: now,
        renew_time: None,
        status: CertificateStatus::Failed,
        fingerprint: CertificateFingerprint {
            sha256: String::new(),
        },
        algorithm: CertificateAlgorithm::Unknown("unknown".to_string()),
        san: vec![domain.to_string()],
        cert_path: None,
        key_path: None,
        serial_number: None,
        last_error: None,
    }
}

fn certificate_store_root() -> std::path::PathBuf {
    if let Some(value) = std::env::var_os("GATE_CERTIFICATE_STORE") {
        return std::path::PathBuf::from(value);
    }
    if let Some(value) = std::env::var_os("GATE_CERT_DIR") {
        return std::path::PathBuf::from(value);
    }
    gateway_data_dir().join("certificates")
}

fn parse_tunnel_config(session_id: &str, body: &Value) -> anyhow::Result<TunnelConfig> {
    let tunnel_id = body
        .get("id")
        .or_else(|| body.get("tunnelId"))
        .and_then(Value::as_str)
        .ok_or_else(|| anyhow::anyhow!("tunnel id is required"))?
        .trim()
        .to_string();
    if tunnel_id.is_empty() {
        return Err(anyhow::anyhow!("tunnel id is required"));
    }

    let protocol = body
        .get("protocol")
        .and_then(Value::as_str)
        .unwrap_or("tcp")
        .trim()
        .to_ascii_lowercase();
    if !matches!(protocol.as_str(), "tcp" | "http" | "https") {
        return Err(anyhow::anyhow!("unsupported tunnel protocol: {protocol}"));
    }

    let remote_port = json_u16(body, "remotePort")
        .or_else(|| json_u16(body, "remote_port"))
        .ok_or_else(|| anyhow::anyhow!("remote port is required"))?;
    let local_port = json_u16(body, "localPort")
        .or_else(|| json_u16(body, "local_port"))
        .ok_or_else(|| anyhow::anyhow!("local port is required"))?;
    let local_host = body
        .get("localHost")
        .or_else(|| body.get("local_host"))
        .and_then(Value::as_str)
        .unwrap_or("127.0.0.1")
        .trim()
        .to_string();

    if local_host.is_empty() {
        return Err(anyhow::anyhow!("local host is required"));
    }

    let remote_port = if matches!(protocol.as_str(), "http" | "https") {
        normalize_http_tunnel_port(&protocol, remote_port)
    } else {
        remote_port
    };

    Ok(TunnelConfig {
        tunnel_id,
        session_id: session_id.to_string(),
        protocol,
        remote_port,
        local_host,
        local_port,
        host: optional_string(body, "host"),
        path: optional_string(body, "path"),
        metadata: body.get("metadata").cloned().unwrap_or_else(|| json!({})),
        enabled: true,
        updated_at: Utc::now().timestamp_millis(),
        runtime: parse_runtime_config(body),
    })
}

fn parse_runtime_config(body: &Value) -> TunnelRuntimeConfig {
    let runtime_body = body
        .get("runtimeConfig")
        .or_else(|| body.get("runtime_config"))
        .or_else(|| body.get("runtime"))
        .unwrap_or(body);
    let mut runtime = TunnelRuntimeConfig::default();

    if let Some(value) = json_u64_any(runtime_body, &["maxConnections", "max_connections"])
        .or_else(|| json_u64_any(body, &["maxConnections", "max_connections"]))
        .filter(|value| *value > 0)
    {
        runtime.max_connections = value;
    }

    if let Some(value) = json_u64_any(
        runtime_body,
        &[
            "idleTimeoutMs",
            "idle_timeout_ms",
            "idleTimeout",
            "idle_timeout",
        ],
    )
    .or_else(|| {
        json_u64_any(
            body,
            &[
                "idleTimeoutMs",
                "idle_timeout_ms",
                "idleTimeout",
                "idle_timeout",
            ],
        )
    })
    .filter(|value| *value > 0)
    {
        runtime.idle_timeout_ms = value;
    }

    if let Some(value) = json_u64_any(
        runtime_body,
        &[
            "bandwidthLimitBps",
            "bandwidth_limit_bps",
            "bandwidthLimit",
            "bandwidth_limit",
        ],
    )
    .or_else(|| {
        json_u64_any(
            body,
            &[
                "bandwidthLimitBps",
                "bandwidth_limit_bps",
                "bandwidthLimit",
                "bandwidth_limit",
            ],
        )
    }) {
        runtime.bandwidth_limit_bps = value;
    }

    runtime
}

fn heartbeat_client_id(body: &Value) -> Option<String> {
    optional_string(body, "clientId")
        .or_else(|| optional_string(body, "client_id"))
        .or_else(|| {
            body.get("client")
                .and_then(|value| optional_string(value, "id"))
        })
}

fn heartbeat_client_status(body: &Value) -> Option<String> {
    optional_string(body, "clientStatus")
        .or_else(|| optional_string(body, "client_status"))
        .or_else(|| optional_string(body, "status"))
        .or_else(|| {
            body.get("client")
                .and_then(|value| optional_string(value, "status"))
        })
}

fn heartbeat_runtime_status(body: &Value) -> Value {
    body.get("runtimeStatus")
        .or_else(|| body.get("runtime_status"))
        .or_else(|| body.get("runtime"))
        .cloned()
        .unwrap_or_else(|| json!({}))
}

fn heartbeat_registered_tunnels(body: &Value) -> Vec<String> {
    body.get("registeredTunnels")
        .or_else(|| body.get("registered_tunnels"))
        .or_else(|| body.get("tunnels"))
        .map(collect_tunnel_ids)
        .unwrap_or_default()
}

fn collect_tunnel_ids(value: &Value) -> Vec<String> {
    match value {
        Value::Array(items) => items
            .iter()
            .filter_map(|item| {
                item.as_str()
                    .map(str::to_string)
                    .or_else(|| optional_string(item, "id"))
                    .or_else(|| optional_string(item, "tunnelId"))
                    .or_else(|| optional_string(item, "tunnel_id"))
            })
            .collect(),
        Value::Object(_) => optional_string(value, "id")
            .or_else(|| optional_string(value, "tunnelId"))
            .or_else(|| optional_string(value, "tunnel_id"))
            .into_iter()
            .collect(),
        _ => Vec::new(),
    }
}

fn json_u16(body: &Value, key: &str) -> Option<u16> {
    let value = body.get(key)?.as_u64()?;
    u16::try_from(value).ok().filter(|port| *port > 0)
}

fn json_u64_any(body: &Value, keys: &[&str]) -> Option<u64> {
    keys.iter().find_map(|key| body.get(*key)?.as_u64())
}

fn optional_string(body: &Value, key: &str) -> Option<String> {
    body.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn try_acquire_connection(counter: &AtomicU64, max_connections: u64) -> bool {
    counter
        .fetch_update(Ordering::AcqRel, Ordering::Relaxed, |current| {
            (current < max_connections).then_some(current + 1)
        })
        .is_ok()
}

async fn copy_direction<R, W>(
    reader: &mut R,
    writer: &mut W,
    tunnel: &Arc<TunnelSession>,
    counters: &GatewayCounters,
    direction: CopyDirection,
    idle_timeout: Duration,
    bandwidth_limit_bps: u64,
) -> io::Result<u64>
where
    R: AsyncRead + Unpin,
    W: AsyncWrite + Unpin,
{
    let started = Instant::now();
    let mut copied = 0_u64;
    let mut buffer = vec![0_u8; RELAY_BUFFER_SIZE];

    loop {
        let read = if idle_timeout.is_zero() {
            reader.read(&mut buffer).await
        } else {
            match timeout(idle_timeout, reader.read(&mut buffer)).await {
                Ok(result) => result,
                Err(_) => Err(io::Error::new(ErrorKind::TimedOut, "relay idle timeout")),
            }
        }?;

        if read == 0 {
            writer.shutdown().await?;
            return Ok(copied);
        }

        writer.write_all(&buffer[..read]).await?;
        copied = copied.saturating_add(read as u64);
        record_tunnel_bytes(tunnel, counters, direction, read as u64);
        throttle_bandwidth(copied, started, bandwidth_limit_bps).await;
    }
}

fn record_tunnel_bytes(
    tunnel: &Arc<TunnelSession>,
    counters: &GatewayCounters,
    direction: CopyDirection,
    bytes: u64,
) {
    if bytes == 0 {
        return;
    }

    match direction {
        CopyDirection::BytesIn => {
            tunnel.bytes_in.fetch_add(bytes, Ordering::Relaxed);
            counters.bytes_in.fetch_add(bytes, Ordering::Relaxed);
        }
        CopyDirection::BytesOut => {
            tunnel.bytes_out.fetch_add(bytes, Ordering::Relaxed);
            counters.bytes_out.fetch_add(bytes, Ordering::Relaxed);
        }
    }
    tunnel
        .last_activity
        .store(now_millis_u64(), Ordering::Relaxed);
}

async fn throttle_bandwidth(copied: u64, started: Instant, bandwidth_limit_bps: u64) {
    if bandwidth_limit_bps == 0 {
        return;
    }

    let expected = Duration::from_secs_f64(copied as f64 / bandwidth_limit_bps as f64);
    let elapsed = started.elapsed();
    if expected > elapsed {
        sleep(expected - elapsed).await;
    }
}

fn describe_io_error(direction: &str, source: &io::Error) -> String {
    let reason = match source.kind() {
        ErrorKind::BrokenPipe => "broken pipe",
        ErrorKind::ConnectionReset => "connection reset",
        ErrorKind::TimedOut => "timeout",
        ErrorKind::UnexpectedEof => "eof",
        _ => "io error",
    };
    format!("relay {direction} failed: {reason}: {source}")
}

fn now_millis_u64() -> u64 {
    Utc::now().timestamp_millis().max(0) as u64
}

#[derive(Debug, Clone)]
struct HttpGatewayRequest {
    method: String,
    path: String,
    host: String,
}

struct ActiveHttpGatewayRequest {
    inner: Arc<GatewayInner>,
}

impl ActiveHttpGatewayRequest {
    fn new(inner: Arc<GatewayInner>) -> Self {
        inner
            .http_counters
            .active_requests
            .fetch_add(1, Ordering::Relaxed);
        Self { inner }
    }
}

impl Drop for ActiveHttpGatewayRequest {
    fn drop(&mut self) {
        let _ = self.inner.http_counters.active_requests.fetch_update(
            Ordering::Relaxed,
            Ordering::Relaxed,
            |value| value.checked_sub(1),
        );
    }
}

async fn read_http_gateway_head<S>(stream: &mut S) -> io::Result<Option<Vec<u8>>>
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
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "connection closed while reading HTTP headers",
            ));
        }
        buffer.push(byte[0]);
        if buffer.len() > HTTP_HEADER_LIMIT {
            return Err(io::Error::new(
                ErrorKind::InvalidData,
                "HTTP header limit exceeded",
            ));
        }
        if buffer.ends_with(b"\r\n\r\n") {
            return Ok(Some(buffer));
        }
    }
}

async fn write_http_gateway_error(
    stream: &mut (impl AsyncWrite + Unpin),
    status: u16,
    message: &str,
) -> io::Result<u64> {
    let body = format!("{message}\n");
    let response = format!(
        "HTTP/1.1 {status} {message}\r\nContent-Length: {}\r\nContent-Type: text/plain; charset=utf-8\r\nConnection: close\r\n\r\n{}",
        body.len(),
        body
    );
    stream.write_all(response.as_bytes()).await?;
    stream.flush().await?;
    Ok(response.len() as u64)
}

fn parse_http_gateway_request(bytes: &[u8]) -> Result<HttpGatewayRequest, String> {
    let text = std::str::from_utf8(bytes).map_err(|error| error.to_string())?;
    let mut lines = text.split("\r\n");
    let request_line = lines
        .next()
        .ok_or_else(|| "missing request line".to_string())?;
    let parts = request_line.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err("invalid request line".to_string());
    }

    let mut host = String::new();
    for line in lines {
        if line.is_empty() {
            break;
        }
        let Some((name, value)) = line.split_once(':') else {
            continue;
        };
        if name.eq_ignore_ascii_case("Host") {
            host = normalize_http_host(value);
            break;
        }
    }

    if host.is_empty() {
        host = host_from_absolute_uri(parts[1]).unwrap_or_default();
    }

    Ok(HttpGatewayRequest {
        method: parts[0].to_string(),
        path: target_to_http_path(parts[1]),
        host,
    })
}

fn host_from_absolute_uri(target: &str) -> Option<String> {
    let rest = target
        .strip_prefix("http://")
        .or_else(|| target.strip_prefix("https://"))?;
    let host = rest.split('/').next().unwrap_or_default();
    Some(normalize_http_host(host))
}

fn normalize_http_path_prefix(path: Option<&str>) -> String {
    let path = path.unwrap_or("/").trim();
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

fn http_path_matches(configured_path: Option<&str>, request_path: &str) -> bool {
    let prefix = normalize_http_path_prefix(configured_path);
    if prefix == "/" {
        return true;
    }
    request_path == prefix || request_path.starts_with(&format!("{prefix}/"))
}

fn http_path_prefix_len(configured_path: Option<&str>) -> usize {
    let prefix = normalize_http_path_prefix(configured_path);
    if prefix == "/" {
        1
    } else {
        prefix.len()
    }
}

fn strip_http_path_prefix(configured_path: Option<&str>, request_path: &str) -> String {
    let prefix = normalize_http_path_prefix(configured_path);
    if prefix == "/" {
        return request_path.to_string();
    }
    if request_path == prefix {
        return "/".to_string();
    }
    if let Some(rest) = request_path.strip_prefix(&format!("{prefix}/")) {
        if rest.is_empty() {
            return "/".to_string();
        }
        return format!("/{rest}");
    }
    request_path.to_string()
}

fn rewrite_http_request_path(head_bytes: &[u8], configured_path: Option<&str>) -> Result<Vec<u8>, String> {
    let prefix = normalize_http_path_prefix(configured_path);
    if prefix == "/" {
        return Ok(head_bytes.to_vec());
    }

    let text = std::str::from_utf8(head_bytes).map_err(|error| error.to_string())?;
    let line_end = if text.contains("\r\n") { "\r\n" } else { "\n" };
    let lines = text.split(line_end).collect::<Vec<_>>();
    let request_line = lines
        .first()
        .ok_or_else(|| "missing request line".to_string())?;
    let parts = request_line.split_whitespace().collect::<Vec<_>>();
    if parts.len() != 3 {
        return Err("invalid request line".to_string());
    }

    let old_path = target_to_http_path(parts[1]);
    let new_path = strip_http_path_prefix(configured_path, &old_path);
    if new_path == old_path {
        return Ok(head_bytes.to_vec());
    }

    let mut rebuilt = format!("{} {} {}", parts[0], new_path, parts[2]);
    for line in lines.iter().skip(1) {
        rebuilt.push_str(line_end);
        rebuilt.push_str(line);
    }
    Ok(rebuilt.into_bytes())
}

fn target_to_http_path(target: &str) -> String {
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
    }
    "/".to_string()
}

fn normalize_http_host(host: &str) -> String {
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

fn http_host_matches(configured: &str, host: &str) -> bool {
    let configured = normalize_http_host(configured);
    if configured == "*" {
        return true;
    }
    if let Some(suffix) = configured.strip_prefix("*.") {
        return host == suffix || host.ends_with(&format!(".{suffix}"));
    }
    configured == host
}

#[derive(Debug, Clone)]
struct HttpTunnelRouteCandidate {
    tunnel_id: String,
    host_unrestricted: bool,
    lifecycle: TunnelLifecycle,
    has_workers: bool,
}

fn http_tunnel_host_unrestricted(host: Option<&str>) -> bool {
    host.map(str::trim).is_none_or(str::is_empty)
}

fn http_tunnel_is_routable(lifecycle: &TunnelLifecycle, has_workers: bool) -> bool {
    if lifecycle.status.is_terminal() {
        return false;
    }
    if matches!(
        lifecycle.status,
        TunnelSessionStatus::Online
            | TunnelSessionStatus::Recovering
            | TunnelSessionStatus::Initializing
    ) {
        return true;
    }
    has_workers
}

fn is_literal_http_host(host: &str) -> bool {
    if host.is_empty() {
        return true;
    }
    if host.parse::<IpAddr>().is_ok() {
        return true;
    }
    host.starts_with('[') && host.ends_with(']') && host[1..host.len() - 1].parse::<IpAddr>().is_ok()
}

fn select_http_tunnel_candidate(
    candidates: &[HttpTunnelRouteCandidate],
    normalized_host: &str,
) -> Option<String> {
    if candidates.is_empty() {
        return None;
    }
    if candidates.len() == 1 {
        return Some(candidates[0].tunnel_id.clone());
    }

    if is_literal_http_host(normalized_host) {
        let unrestricted = candidates
            .iter()
            .filter(|candidate| candidate.host_unrestricted)
            .collect::<Vec<_>>();
        if unrestricted.len() == 1 {
            return Some(unrestricted[0].tunnel_id.clone());
        }
    }

    let active = candidates
        .iter()
        .filter(|candidate| {
            candidate.has_workers
                || matches!(
                    candidate.lifecycle.status,
                    TunnelSessionStatus::Online | TunnelSessionStatus::Recovering
                )
        })
        .collect::<Vec<_>>();
    if active.len() == 1 {
        return Some(active[0].tunnel_id.clone());
    }

    let pool: Vec<&HttpTunnelRouteCandidate> = if active.is_empty() {
        candidates.iter().collect()
    } else {
        active
    };
    pool.iter()
        .max_by_key(|candidate| {
            (
                candidate.has_workers,
                matches!(candidate.lifecycle.status, TunnelSessionStatus::Online),
                candidate.lifecycle.last_heartbeat,
            )
        })
        .map(|candidate| candidate.tunnel_id.clone())
}

fn is_http_tunnel_protocol(protocol: &str) -> bool {
    matches!(protocol, "http" | "https")
}

fn required_string(body: &Value, keys: &[&str]) -> anyhow::Result<String> {
    optional_string_any(body, keys)
        .ok_or_else(|| anyhow::anyhow!("{} is required", keys.first().copied().unwrap_or("value")))
}

fn optional_string_any(body: &Value, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| optional_string(body, key))
}

fn domain_id_for_host(host: &str) -> anyhow::Result<ManagedDomainId> {
    ManagedDomainId::new(format!("domain:{}", sanitize_domain_key(host)))
        .map_err(|error| anyhow::anyhow!(error.to_string()))
}

fn sanitize_domain_key(host: &str) -> String {
    host.chars()
        .map(|value| match value {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => value,
            _ => '_',
        })
        .collect()
}

fn domain_json(domain: &ManagedDomain) -> Value {
    json!({
        "id": domain.id().as_str(),
        "host": domain.host().as_str(),
        "tunnelId": domain.tunnel_id().map(ManagedTunnelId::as_str),
        "status": domain.status().as_str(),
        "bindStatus": domain.bind_status().as_str()
    })
}

fn average_u64(total: u64, count: u64) -> f64 {
    if count == 0 {
        0.0
    } else {
        total as f64 / count as f64
    }
}

fn ratio_u64(part: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        part as f64 / total as f64
    }
}

fn elapsed_millis(started: Instant) -> u64 {
    started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
}

fn default_domain_repository() -> Option<SqliteDomainRepository> {
    #[cfg(test)]
    {
        None
    }

    #[cfg(not(test))]
    {
        SqliteDomainRepository::open(gateway_domain_store_path()).ok()
    }
}

#[cfg(not(test))]
fn gateway_domain_store_path() -> std::path::PathBuf {
    if let Some(value) = std::env::var_os("GATE_SERVER_DOMAIN_DB") {
        return std::path::PathBuf::from(value);
    }

    gateway_data_dir().join("server-domains.sqlite3")
}

fn gateway_data_dir() -> std::path::PathBuf {
    if let Some(program_data) = std::env::var_os("PROGRAMDATA") {
        return std::path::PathBuf::from(program_data).join("Gate");
    }
    if let Some(appdata) = std::env::var_os("APPDATA") {
        return std::path::PathBuf::from(appdata).join("Gate");
    }
    if let Some(xdg_data_home) = std::env::var_os("XDG_DATA_HOME") {
        return std::path::PathBuf::from(xdg_data_home).join("Gate");
    }
    if let Some(home) = std::env::var_os("HOME") {
        return std::path::PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("Gate");
    }

    let path = std::path::Path::new(".gate").to_path_buf();
    let _ = std::fs::create_dir_all(&path);
    path
}

pub async fn read_message(
    stream: &mut TcpStream,
    protocol: &ProtocolManager,
) -> anyhow::Result<Option<Message>> {
    let mut length = [0_u8; 4];
    match stream.read_exact(&mut length).await {
        Ok(_) => {}
        Err(source) if source.kind() == ErrorKind::UnexpectedEof => return Ok(None),
        Err(source) => return Err(source.into()),
    }

    let length = u32::from_be_bytes(length) as usize;
    let mut payload = vec![0_u8; length];
    stream.read_exact(&mut payload).await?;
    Ok(Some(protocol.decode(&payload)?))
}

pub async fn write_message(
    stream: &mut TcpStream,
    protocol: &ProtocolManager,
    message: &Message,
) -> anyhow::Result<()> {
    let payload = protocol.encode(message)?;
    let frame = Frame::new(payload)?;
    let bytes = FrameEncoder::encode(&frame);
    stream.write_all(&bytes).await?;
    stream.flush().await?;
    Ok(())
}

pub fn response_for(request: &Message, body: Value) -> Message {
    let mut response = Message::new(
        MessageType::Response,
        request.header.command.clone(),
        Body::Json(body),
        Metadata::default(),
    );
    response.header.request_id = request.header.request_id;
    response
}

pub fn json_body(message: &Message) -> Value {
    match &message.body {
        Body::Json(value) => value.clone(),
        _ => Value::Null,
    }
}

pub fn ok(data: Value) -> Value {
    json!({ "ok": true, "data": data })
}

pub fn err(code: &str, message: &str) -> Value {
    json!({
        "ok": false,
        "error": {
            "code": code,
            "message": message
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use rcgen::generate_simple_self_signed;
    use rustls::pki_types::{CertificateDer, ServerName};
    use rustls::{ClientConfig, RootCertStore};
    use tokio_rustls::client::TlsStream as ClientTlsStream;
    use tokio_rustls::TlsConnector;

    #[test]
    fn http_path_matches_prefix_routes() {
        assert!(http_path_matches(Some("/hello"), "/hello"));
        assert!(http_path_matches(Some("/hello"), "/hello/world"));
        assert!(!http_path_matches(Some("/hello"), "/"));
        assert!(!http_path_matches(Some("/hello"), "/other"));
        assert!(http_path_matches(Some("/"), "/anything"));
    }

    #[test]
    fn strip_http_path_prefix_rewrites_to_backend_root() {
        assert_eq!(strip_http_path_prefix(Some("/hello"), "/hello"), "/");
        assert_eq!(
            strip_http_path_prefix(Some("/hello"), "/hello/api"),
            "/api"
        );
        assert_eq!(strip_http_path_prefix(Some("/"), "/hello"), "/hello");
    }

    #[test]
    fn rewrite_http_request_path_updates_request_line() {
        let head = b"GET /hello HTTP/1.1\r\nHost: maven666.top\r\n\r\n";
        let rewritten = rewrite_http_request_path(head, Some("/hello")).expect("rewrite");
        let text = std::str::from_utf8(&rewritten).expect("utf8");
        assert!(text.starts_with("GET / HTTP/1.1\r\n"));
        assert!(text.contains("Host: maven666.top"));
    }

    #[tokio::test]
    async fn gateway_relays_public_connection_to_waiting_worker() -> anyhow::Result<()> {
        let gateway = TunnelGateway::new();
        let session_id = gateway.create_session().await;
        let remote_port = unused_loopback_port().await?;
        let tunnel_id = "relay-test";
        let body = tunnel_body(tunnel_id, remote_port, 18080);

        gateway.register_tunnel(&session_id, &body).await?;
        let (relay_server, mut relay_client) = tcp_pair().await?;
        gateway
            .attach_relay_worker(tunnel_id, &session_id, relay_server)
            .await?;

        let worker = tokio::spawn(async move {
            let protocol = ProtocolBuilder::new().build();
            let start = read_message(&mut relay_client, &protocol)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing relay start"))?;
            assert_eq!(start.header.command, Command::TunnelRelayStart);

            let mut request = [0_u8; 4];
            relay_client.read_exact(&mut request).await?;
            assert_eq!(&request, b"ping");
            relay_client.write_all(b"pong").await?;
            relay_client.shutdown().await?;
            Ok::<(), anyhow::Error>(())
        });

        let mut public = TcpStream::connect(("127.0.0.1", remote_port)).await?;
        public.write_all(b"ping").await?;
        let mut response = [0_u8; 4];
        public.read_exact(&mut response).await?;

        assert_eq!(&response, b"pong");

        drop(public);
        worker.await??;
        let statistics = gateway.statistics().await;
        assert_eq!(statistics["tunnels"][0]["bytesIn"], 4);
        assert_eq!(statistics["tunnels"][0]["bytesOut"], 4);
        gateway.stop_tunnel(tunnel_id).await?;
        Ok(())
    }

    #[tokio::test]
    async fn https_gateway_forwards_basic_request_through_http_pipeline() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let gateway = TunnelGateway::with_certificate_store_root(temp_dir.path().to_path_buf());
        let session_id = gateway.create_session().await;
        let remote_port = unused_loopback_port().await?;
        let certificate = save_test_certificate(&gateway, "secure.example.com")?;

        gateway
            .register_tunnel(
                &session_id,
                &https_tunnel_body("secure-https", remote_port, 18443, "secure.example.com"),
            )
            .await?;

        let (relay_server, relay_client) = tcp_pair().await?;
        gateway
            .attach_relay_worker("secure-https", &session_id, relay_server)
            .await?;
        let worker = tokio::spawn(http_worker_response(
            relay_client,
            "secure.example.com",
            "secure",
        ));

        let mut public = tls_client(remote_port, "secure.example.com", &certificate).await?;
        public
            .write_all(
                b"GET /hello HTTP/1.1\r\nHost: secure.example.com\r\nConnection: close\r\n\r\n",
            )
            .await?;
        let response = read_tls_response(&mut public).await?;
        assert!(response.contains("200 OK"));
        assert!(response.ends_with("secure"));
        drop(public);
        worker.await??;

        let statistics = wait_for_http_requests(&gateway, 1).await?;
        assert_eq!(statistics["https"]["tls_handshake_total"], 1);
        assert_eq!(statistics["https"]["tls_errors"], 0);
        assert_eq!(statistics["https"]["https_requests"], 1);
        assert!(statistics["https"]["tls_version"]
            .as_str()
            .unwrap_or("")
            .starts_with("TLS"));
        assert!(statistics["https"]["certificate_expire_days"]["secure.example.com"].is_i64());

        gateway.stop_tunnel("secure-https").await?;
        Ok(())
    }

    #[tokio::test]
    async fn https_gateway_routes_multiple_domains_by_sni_and_host() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let gateway = TunnelGateway::with_certificate_store_root(temp_dir.path().to_path_buf());
        let session_id = gateway.create_session().await;
        let remote_port = unused_loopback_port().await?;
        let api_certificate = save_test_certificate(&gateway, "api.example.com")?;
        let admin_certificate = save_test_certificate(&gateway, "admin.example.com")?;

        gateway
            .register_tunnel(
                &session_id,
                &https_tunnel_body("api-https", remote_port, 18444, "api.example.com"),
            )
            .await?;
        gateway
            .register_tunnel(
                &session_id,
                &https_tunnel_body("admin-https", remote_port, 18445, "admin.example.com"),
            )
            .await?;

        let (api_relay_server, api_relay_client) = tcp_pair().await?;
        gateway
            .attach_relay_worker("api-https", &session_id, api_relay_server)
            .await?;
        let api_worker = tokio::spawn(http_worker_response(
            api_relay_client,
            "api.example.com",
            "api",
        ));
        let mut api_public = tls_client(remote_port, "api.example.com", &api_certificate).await?;
        api_public
            .write_all(b"GET / HTTP/1.1\r\nHost: api.example.com\r\nConnection: close\r\n\r\n")
            .await?;
        assert!(read_tls_response(&mut api_public).await?.ends_with("api"));
        drop(api_public);
        api_worker.await??;

        let (admin_relay_server, admin_relay_client) = tcp_pair().await?;
        gateway
            .attach_relay_worker("admin-https", &session_id, admin_relay_server)
            .await?;
        let admin_worker = tokio::spawn(http_worker_response(
            admin_relay_client,
            "admin.example.com",
            "admin",
        ));
        let mut admin_public =
            tls_client(remote_port, "admin.example.com", &admin_certificate).await?;
        admin_public
            .write_all(
                b"GET /dashboard HTTP/1.1\r\nHost: admin.example.com\r\nConnection: close\r\n\r\n",
            )
            .await?;
        assert!(read_tls_response(&mut admin_public)
            .await?
            .ends_with("admin"));
        drop(admin_public);
        admin_worker.await??;

        let statistics = wait_for_http_requests(&gateway, 2).await?;
        assert_eq!(statistics["https"]["tls_handshake_total"], 2);
        assert_eq!(statistics["https"]["https_requests"], 2);

        gateway.stop_tunnel("api-https").await?;
        gateway.stop_tunnel("admin-https").await?;
        Ok(())
    }

    #[tokio::test]
    async fn https_gateway_reloads_certificate_store_without_listener_restart() -> anyhow::Result<()>
    {
        let temp_dir = tempfile::tempdir()?;
        let gateway = TunnelGateway::with_certificate_store_root(temp_dir.path().to_path_buf());
        let session_id = gateway.create_session().await;
        let remote_port = unused_loopback_port().await?;
        let initial = save_test_certificate(&gateway, "reload.example.com")?;

        gateway
            .register_tunnel(
                &session_id,
                &https_tunnel_body("reload-https", remote_port, 18446, "reload.example.com"),
            )
            .await?;

        let replacement = save_test_certificate(&gateway, "reload.example.com")?;
        assert_ne!(
            initial.record.fingerprint.sha256,
            replacement.record.fingerprint.sha256
        );
        gateway.reload_tls_cache_from_store()?;

        let public = tls_client(remote_port, "reload.example.com", &replacement).await?;
        drop(public);

        let statistics = wait_for_https_handshakes(&gateway, 1).await?;
        assert_eq!(statistics["https"]["tls_handshake_total"], 1);
        gateway.stop_tunnel("reload-https").await?;
        Ok(())
    }

    #[tokio::test]
    async fn heartbeat_timeout_marks_tunnel_offline() -> anyhow::Result<()> {
        let gateway = TunnelGateway::with_heartbeat_timeout(1);
        let session_id = gateway.create_session().await;
        let remote_port = unused_loopback_port().await?;
        let tunnel_id = "heartbeat-timeout";

        gateway
            .register_tunnel(&session_id, &tunnel_body(tunnel_id, remote_port, 18080))
            .await?;
        sleep(Duration::from_millis(3)).await;
        gateway.expire_heartbeat_timeouts().await;

        let statistics = gateway.statistics().await;
        assert_eq!(statistics["tunnels"][0]["status"], "Offline");
        gateway.stop_tunnel(tunnel_id).await?;
        Ok(())
    }

    #[tokio::test]
    async fn session_recovery_restores_tunnel_owner() -> anyhow::Result<()> {
        let gateway = TunnelGateway::new();
        let session_id = gateway
            .create_or_restore_session(Some("client-a".to_string()), None)
            .await;
        let remote_port = unused_loopback_port().await?;
        let tunnel_id = "session-recovery";
        let body = tunnel_body(tunnel_id, remote_port, 18080);

        gateway.register_tunnel(&session_id, &body).await?;
        gateway.close_session(&session_id).await;
        let restored = gateway
            .create_or_restore_session(Some("client-a".to_string()), Some(session_id.clone()))
            .await;
        assert_eq!(restored, session_id);
        gateway.register_tunnel(&restored, &body).await?;

        let statistics = gateway.statistics().await;
        assert_eq!(statistics["tunnels"][0]["sessionId"], restored);
        assert_eq!(statistics["tunnels"][0]["status"], "Online");
        gateway.stop_tunnel(tunnel_id).await?;
        Ok(())
    }

    #[tokio::test]
    async fn multiple_public_connections_are_tracked() -> anyhow::Result<()> {
        let gateway = TunnelGateway::new();
        let session_id = gateway.create_session().await;
        let remote_port = unused_loopback_port().await?;
        let tunnel_id = "multi-connection";
        let mut body = tunnel_body(tunnel_id, remote_port, 18080);
        body["runtime"] = json!({ "maxConnections": 2 });
        gateway.register_tunnel(&session_id, &body).await?;

        let mut workers = Vec::new();
        for index in 0..2 {
            let (relay_server, mut relay_client) = tcp_pair().await?;
            gateway
                .attach_relay_worker(tunnel_id, &session_id, relay_server)
                .await?;
            workers.push(tokio::spawn(async move {
                let protocol = ProtocolBuilder::new().build();
                let _start = read_message(&mut relay_client, &protocol)
                    .await?
                    .ok_or_else(|| anyhow::anyhow!("missing relay start"))?;
                let mut request = [0_u8; 5];
                relay_client.read_exact(&mut request).await?;
                relay_client
                    .write_all(format!("pong{index}").as_bytes())
                    .await?;
                relay_client.shutdown().await?;
                Ok::<(), anyhow::Error>(())
            }));
        }

        let first = tokio::spawn(connect_and_roundtrip(remote_port, b"ping0".to_vec()));
        let second = tokio::spawn(connect_and_roundtrip(remote_port, b"ping1".to_vec()));
        let mut responses = vec![first.await??, second.await??];
        responses.sort();

        assert_eq!(responses, vec![b"pong0".to_vec(), b"pong1".to_vec()]);
        for worker in workers {
            worker.await??;
        }

        wait_for_active_connections(&gateway, 0).await?;
        let statistics = gateway.statistics().await;
        assert_eq!(statistics["tunnels"][0]["connectionsTotal"], 2);
        assert_eq!(statistics["tunnels"][0]["activeConnections"], 0);
        gateway.stop_tunnel(tunnel_id).await?;
        Ok(())
    }

    #[tokio::test]
    async fn connection_close_releases_active_connection() -> anyhow::Result<()> {
        let gateway = TunnelGateway::new();
        let session_id = gateway.create_session().await;
        let remote_port = unused_loopback_port().await?;
        let tunnel_id = "close-test";

        gateway
            .register_tunnel(&session_id, &tunnel_body(tunnel_id, remote_port, 18080))
            .await?;
        let (relay_server, mut relay_client) = tcp_pair().await?;
        gateway
            .attach_relay_worker(tunnel_id, &session_id, relay_server)
            .await?;

        let worker = tokio::spawn(async move {
            let protocol = ProtocolBuilder::new().build();
            let _start = read_message(&mut relay_client, &protocol)
                .await?
                .ok_or_else(|| anyhow::anyhow!("missing relay start"))?;
            relay_client.shutdown().await?;
            Ok::<(), anyhow::Error>(())
        });

        let mut public = TcpStream::connect(("127.0.0.1", remote_port)).await?;
        let mut buffer = [0_u8; 1];
        let read = public.read(&mut buffer).await?;
        assert_eq!(read, 0);
        drop(public);
        worker.await??;

        wait_for_active_connections(&gateway, 0).await?;
        let statistics = gateway.statistics().await;
        assert_eq!(statistics["tunnels"][0]["activeConnections"], 0);
        gateway.stop_tunnel(tunnel_id).await?;
        Ok(())
    }

    #[tokio::test]
    async fn http_gateway_routes_multiple_hosts_on_shared_port() -> anyhow::Result<()> {
        let gateway = TunnelGateway::new();
        let session_id = gateway.create_session().await;
        let remote_port = unused_loopback_port().await?;

        gateway
            .register_tunnel(
                &session_id,
                &http_tunnel_body("api-http", remote_port, 18081, "api.example.com"),
            )
            .await?;
        gateway
            .register_tunnel(
                &session_id,
                &http_tunnel_body("admin-http", remote_port, 18082, "admin.example.com"),
            )
            .await?;

        let (api_relay_server, api_relay_client) = tcp_pair().await?;
        gateway
            .attach_relay_worker("api-http", &session_id, api_relay_server)
            .await?;
        let api_worker = tokio::spawn(http_worker_response(
            api_relay_client,
            "api.example.com",
            "api",
        ));

        let mut api_public = TcpStream::connect(("127.0.0.1", remote_port)).await?;
        api_public
            .write_all(b"GET /v1 HTTP/1.1\r\nHost: api.example.com\r\nConnection: close\r\n\r\n")
            .await?;
        let mut api_response = String::new();
        api_public.read_to_string(&mut api_response).await?;
        assert!(api_response.contains("200 OK"));
        assert!(api_response.ends_with("api"));
        drop(api_public);
        api_worker.await??;

        let (admin_relay_server, admin_relay_client) = tcp_pair().await?;
        gateway
            .attach_relay_worker("admin-http", &session_id, admin_relay_server)
            .await?;
        let admin_worker = tokio::spawn(http_worker_response(
            admin_relay_client,
            "admin.example.com",
            "admin",
        ));

        let mut admin_public = TcpStream::connect(("127.0.0.1", remote_port)).await?;
        admin_public
            .write_all(
                b"GET /dashboard HTTP/1.1\r\nHost: admin.example.com\r\nConnection: close\r\n\r\n",
            )
            .await?;
        let mut admin_response = String::new();
        admin_public.read_to_string(&mut admin_response).await?;
        assert!(admin_response.contains("200 OK"));
        assert!(admin_response.ends_with("admin"));
        drop(admin_public);
        admin_worker.await??;

        let statistics = wait_for_http_requests(&gateway, 2).await?;
        assert_eq!(statistics["http"]["requestsTotal"], 2);
        assert_eq!(statistics["http"]["statusCodes"]["200"], 2);
        gateway.stop_tunnel("api-http").await?;
        gateway.stop_tunnel("admin-http").await?;
        Ok(())
    }

    #[tokio::test]
    async fn http_gateway_routes_ip_host_to_single_tunnel_without_domain() -> anyhow::Result<()> {
        let gateway = TunnelGateway::new();
        let session_id = gateway.create_session().await;
        let remote_port = unused_loopback_port().await?;
        let tunnel_id = "ip-http";

        gateway
            .register_tunnel(
                &session_id,
                &http_tunnel_body_without_host(tunnel_id, remote_port, 18084),
            )
            .await?;

        let (relay_server, relay_client) = tcp_pair().await?;
        gateway
            .attach_relay_worker(tunnel_id, &session_id, relay_server)
            .await?;
        let worker = tokio::spawn(http_worker_response(relay_client, "127.0.0.1", "ip"));

        let mut public = TcpStream::connect(("127.0.0.1", remote_port)).await?;
        public
            .write_all(b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
            .await?;
        let mut response = String::new();
        public.read_to_string(&mut response).await?;
        assert!(response.contains("200 OK"));
        assert!(response.ends_with("ip"));
        drop(public);
        worker.await??;

        gateway.stop_tunnel(tunnel_id).await?;
        Ok(())
    }

    #[tokio::test]
    async fn http_gateway_purges_stale_tunnel_and_routes_active_ip_request() -> anyhow::Result<()> {
        let gateway = TunnelGateway::new();
        let session_id = gateway.create_session().await;
        let remote_port = unused_loopback_port().await?;

        gateway
            .register_tunnel(
                &session_id,
                &http_tunnel_body("stale-http", remote_port, 18085, "stale.example.com"),
            )
            .await?;
        gateway
            .mark_tunnel_status("stale-http", TunnelSessionStatus::Offline)
            .await;

        gateway
            .register_tunnel(
                &session_id,
                &http_tunnel_body_without_host("active-http", remote_port, 18086),
            )
            .await?;

        let statistics = gateway.statistics().await;
        assert_eq!(statistics["tunnels"].as_array().map(Vec::len), Some(1));

        let (relay_server, relay_client) = tcp_pair().await?;
        gateway
            .attach_relay_worker("active-http", &session_id, relay_server)
            .await?;
        let worker = tokio::spawn(http_worker_response(relay_client, "127.0.0.1", "active"));

        let mut public = TcpStream::connect(("127.0.0.1", remote_port)).await?;
        public
            .write_all(b"GET / HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
            .await?;
        let mut response = String::new();
        public.read_to_string(&mut response).await?;
        assert!(response.contains("200 OK"));
        assert!(response.ends_with("active"));
        drop(public);
        worker.await??;

        gateway.stop_tunnel("active-http").await?;
        Ok(())
    }

    #[tokio::test]
    async fn domain_binding_is_persisted_and_can_be_unbound_deleted() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let db_path = temp_dir.path().join("domains.sqlite3");
        let repository = SqliteDomainRepository::open(&db_path)?;
        let gateway = TunnelGateway::with_domain_repository(repository);
        let session_id = gateway.create_session().await;
        let remote_port = unused_loopback_port().await?;

        gateway
            .register_tunnel(
                &session_id,
                &http_tunnel_body("persist-http", remote_port, 18083, "persist.example.com"),
            )
            .await?;

        let reopened = SqliteDomainRepository::open(&db_path)?;
        let domains = reopened.list()?;
        assert_eq!(domains.len(), 1);
        assert_eq!(domains[0].host().as_str(), "persist.example.com");
        assert_eq!(
            domains[0].tunnel_id().map(ManagedTunnelId::as_str),
            Some("persist-http")
        );

        let unbound = gateway
            .unbind_domain(&json!({ "host": "persist.example.com" }))
            .await?;
        assert_eq!(unbound["bindStatus"], "UNBOUND");

        let deleted = gateway
            .delete_domain(&json!({ "host": "persist.example.com" }))
            .await?;
        assert_eq!(deleted["host"], "persist.example.com");
        assert!(SqliteDomainRepository::open(&db_path)?.list()?.is_empty());

        gateway.stop_tunnel("persist-http").await?;
        Ok(())
    }

    async fn connect_and_roundtrip(remote_port: u16, payload: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        let mut public = TcpStream::connect(("127.0.0.1", remote_port)).await?;
        public.write_all(&payload).await?;
        let mut response = [0_u8; 5];
        public.read_exact(&mut response).await?;
        Ok(response.to_vec())
    }

    async fn wait_for_active_connections(
        gateway: &TunnelGateway,
        expected: u64,
    ) -> anyhow::Result<()> {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let statistics = gateway.statistics().await;
            let active = statistics["tunnels"][0]["activeConnections"]
                .as_u64()
                .unwrap_or_default();
            if active == expected {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err(anyhow::anyhow!(
                    "active connection count did not reach {expected}, last={active}"
                ));
            }
            sleep(Duration::from_millis(10)).await;
        }
    }

    async fn wait_for_http_requests(
        gateway: &TunnelGateway,
        expected: u64,
    ) -> anyhow::Result<Value> {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let statistics = gateway.statistics().await;
            let total = statistics["http"]["requestsTotal"]
                .as_u64()
                .unwrap_or_default();
            if total == expected {
                return Ok(statistics);
            }
            if Instant::now() >= deadline {
                return Err(anyhow::anyhow!(
                    "HTTP request count did not reach {expected}, last={total}"
                ));
            }
            sleep(Duration::from_millis(10)).await;
        }
    }

    async fn wait_for_https_handshakes(
        gateway: &TunnelGateway,
        expected: u64,
    ) -> anyhow::Result<Value> {
        let deadline = Instant::now() + Duration::from_secs(2);
        loop {
            let statistics = gateway.statistics().await;
            let total = statistics["https"]["tls_handshake_total"]
                .as_u64()
                .unwrap_or_default();
            if total == expected {
                return Ok(statistics);
            }
            if Instant::now() >= deadline {
                return Err(anyhow::anyhow!(
                    "HTTPS handshake count did not reach {expected}, last={total}"
                ));
            }
            sleep(Duration::from_millis(10)).await;
        }
    }

    fn tunnel_body(tunnel_id: &str, remote_port: u16, local_port: u16) -> Value {
        json!({
            "id": tunnel_id,
            "protocol": "tcp",
            "remotePort": remote_port,
            "localHost": "127.0.0.1",
            "localPort": local_port
        })
    }

    fn http_tunnel_body(tunnel_id: &str, remote_port: u16, local_port: u16, host: &str) -> Value {
        json!({
            "id": tunnel_id,
            "protocol": "http",
            "remotePort": remote_port,
            "localHost": "127.0.0.1",
            "localPort": local_port,
            "host": host,
            "path": "/"
        })
    }

    fn http_tunnel_body_without_host(tunnel_id: &str, remote_port: u16, local_port: u16) -> Value {
        json!({
            "id": tunnel_id,
            "protocol": "http",
            "remotePort": remote_port,
            "localHost": "127.0.0.1",
            "localPort": local_port,
            "path": "/"
        })
    }

    fn https_tunnel_body(tunnel_id: &str, remote_port: u16, local_port: u16, host: &str) -> Value {
        let mut body = http_tunnel_body(tunnel_id, remote_port, local_port, host);
        body["protocol"] = json!("https");
        body
    }

    fn save_test_certificate(
        gateway: &TunnelGateway,
        domain: &str,
    ) -> anyhow::Result<StoredCertificate> {
        let certificate = generate_simple_self_signed(vec![domain.to_string()])?;
        let certificate_pem = certificate.serialize_pem()?;
        let private_key_pem = certificate.serialize_private_key_pem();
        let mut record = CertificateParser::parse_pem(domain, &certificate_pem)?;
        record.status = CertificateStatus::Active;
        record.last_error = None;
        let stored = StoredCertificate {
            record,
            certificate_pem,
            private_key_pem,
        };
        gateway.inner.certificate_store.save(&stored)?;
        Ok(stored)
    }

    async fn tls_client(
        remote_port: u16,
        server_name: &str,
        certificate: &StoredCertificate,
    ) -> anyhow::Result<ClientTlsStream<TcpStream>> {
        let mut roots = RootCertStore::empty();
        roots.add(first_certificate_der(&certificate.certificate_pem)?)?;
        let provider = ring::default_provider();
        let config = ClientConfig::builder_with_provider(Arc::new(provider))
            .with_protocol_versions(&[&rustls::version::TLS13, &rustls::version::TLS12])?
            .with_root_certificates(roots)
            .with_no_client_auth();
        let connector = TlsConnector::from(Arc::new(config));
        let stream = TcpStream::connect(("127.0.0.1", remote_port)).await?;
        let server_name = ServerName::try_from(server_name.to_string())?;
        Ok(connector.connect(server_name, stream).await?)
    }

    fn first_certificate_der(pem: &str) -> anyhow::Result<CertificateDer<'static>> {
        let mut reader = Cursor::new(pem.as_bytes());
        let certificate = rustls_pemfile::certs(&mut reader)
            .next()
            .transpose()?
            .ok_or_else(|| anyhow::anyhow!("missing generated certificate"))?;
        Ok(certificate)
    }

    async fn read_tls_response(stream: &mut ClientTlsStream<TcpStream>) -> anyhow::Result<String> {
        let mut response = String::new();
        stream.read_to_string(&mut response).await?;
        Ok(response)
    }

    async fn http_worker_response(
        mut relay_client: TcpStream,
        expected_host: &'static str,
        body: &'static str,
    ) -> anyhow::Result<()> {
        let protocol = ProtocolBuilder::new().build();
        let start = read_message(&mut relay_client, &protocol)
            .await?
            .ok_or_else(|| anyhow::anyhow!("missing relay start"))?;
        assert_eq!(start.header.command, Command::TunnelRelayStart);

        let head = read_http_test_head(&mut relay_client).await?;
        let head = String::from_utf8(head)?;
        assert!(head.contains(&format!("Host: {expected_host}")));

        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        relay_client.write_all(response.as_bytes()).await?;
        relay_client.shutdown().await?;
        Ok(())
    }

    async fn read_http_test_head(stream: &mut TcpStream) -> anyhow::Result<Vec<u8>> {
        let mut buffer = Vec::new();
        let mut byte = [0_u8; 1];
        loop {
            stream.read_exact(&mut byte).await?;
            buffer.push(byte[0]);
            if buffer.ends_with(b"\r\n\r\n") {
                return Ok(buffer);
            }
        }
    }

    async fn unused_loopback_port() -> anyhow::Result<u16> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        drop(listener);
        Ok(port)
    }

    async fn tcp_pair() -> anyhow::Result<(TcpStream, TcpStream)> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let client = tokio::spawn(async move { TcpStream::connect(addr).await });
        let (server, _) = listener.accept().await?;
        let client = client.await??;
        Ok((server, client))
    }
}
