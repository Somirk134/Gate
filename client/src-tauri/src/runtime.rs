use chrono::Utc;
use gate_communication::{TcpTransport, Transport, TransportEndpoint};
#[cfg(test)]
use gate_engine::runtime::{
    HttpRouteConfig, HttpRuntimeConfig, HttpTunnelRuntime, RuntimeConfig, TimeoutConfig,
    TunnelRuntime,
};
use gate_protocol::{
    Body, Command, Frame, FrameEncoder, Message, Metadata, ProtocolBuilder, ProtocolManager,
};
use gate_server_domain::{
    model::{
        Domain as ManagedDomain, DomainId as ManagedDomainId, DomainStatus as ManagedDomainStatus,
        Host as ManagedHost, RecordType as ManagedRecordType, TunnelId as ManagedTunnelId,
    },
    repository::{DomainRepository, SqliteRepository as SqliteDomainRepository},
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
#[cfg(test)]
use std::net::{IpAddr, Ipv4Addr};
use std::{
    collections::{BTreeMap, BTreeSet},
    env, fs,
    net::SocketAddr,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};
use sysinfo::{Disks, System};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{watch, Mutex},
    task::JoinHandle,
    time::sleep,
};
use uuid::Uuid;

const STORE_VERSION: u32 = 1;
const MAX_LOGS: usize = 500;
const RELAY_WORKERS_PER_TUNNEL: usize = 4;
const RECONNECT_DELAYS_SECS: [u64; 6] = [1, 2, 5, 10, 30, 60];

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RuntimeLog {
    level: String,
    source: String,
    message: String,
    timestamp: i64,
    #[serde(
        default,
        rename = "tunnelId",
        alias = "tunnel_id",
        skip_serializing_if = "Option::is_none"
    )]
    tunnel_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTunnelRequest {
    pub name: Option<String>,
    pub protocol: Option<String>,
    pub local_host: Option<String>,
    pub local_port: Option<u16>,
    pub remote_port: Option<u16>,
    pub host: Option<String>,
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct HttpRequestRecord {
    method: String,
    url: String,
    host: String,
    status: u16,
    latency_ms: u64,
    client_ip: String,
    traffic_bytes: u64,
    timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TunnelRecord {
    id: String,
    name: String,
    protocol: String,
    status: String,
    local_host: String,
    local_port: u16,
    remote_port: u16,
    host: Option<String>,
    path: Option<String>,
    upload_speed_bps: f64,
    download_speed_bps: f64,
    #[serde(default)]
    upload_bytes: u64,
    #[serde(default)]
    download_bytes: u64,
    connections: u64,
    uptime_seconds: u64,
    request_count: u64,
    success_count: u64,
    total_latency_ms: u64,
    started_at: Option<i64>,
    created_at: i64,
    updated_at: i64,
    last_sample_at: i64,
    #[serde(default)]
    tls_session_count: u64,
    #[serde(default)]
    tls_handshake_count: u64,
    #[serde(default)]
    tls_error_count: u64,
    #[serde(default)]
    tls_version: Option<String>,
    #[serde(default)]
    certificate_status: Option<String>,
    #[serde(default)]
    certificate_expire_days: Option<i64>,
    #[serde(default)]
    certificate_issuer: Option<String>,
    recent_requests: Vec<HttpRequestRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RuntimeDomainRecord {
    domain: String,
    tunnel_id: String,
    protocol: String,
    status: String,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RuntimeCertificateReference {
    domain: String,
    tunnel_id: String,
    store_root: PathBuf,
    status: String,
    updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServerRecord {
    id: String,
    name: String,
    kind: String,
    host: String,
    port: u16,
    token: String,
    region: String,
    remark: String,
    tags: Vec<String>,
    heartbeat_interval: u64,
    reconnect_interval: u64,
    auto_connect: bool,
    status: String,
    last_error: Option<String>,
    last_rtt_ms: Option<u64>,
    session_id: Option<String>,
    last_checked_at: Option<i64>,
    last_connected_at: Option<i64>,
    created_at: i64,
    updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateServerRequest {
    pub name: String,
    pub kind: Option<String>,
    pub host: String,
    pub port: u16,
    pub token: String,
    pub region: Option<String>,
    pub remark: Option<String>,
    pub tags: Option<Vec<String>>,
    pub heartbeat_interval: Option<u64>,
    pub reconnect_interval: Option<u64>,
    pub auto_connect: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateServerRequest {
    pub name: Option<String>,
    pub kind: Option<String>,
    pub host: Option<String>,
    pub port: Option<u16>,
    pub token: Option<String>,
    pub region: Option<String>,
    pub remark: Option<String>,
    pub tags: Option<Vec<String>>,
    pub heartbeat_interval: Option<u64>,
    pub reconnect_interval: Option<u64>,
    pub auto_connect: Option<bool>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
struct RuntimeCounters {
    connection_total: u64,
    reconnect_total: u64,
    disconnect_total: u64,
    request_total: u64,
    response_total: u64,
    event_total: u64,
    auth_success: u64,
    auth_failure: u64,
    heartbeat_ping: u64,
    heartbeat_pong: u64,
    heartbeat_timeout: u64,
    last_rtt_ms: Option<u64>,
    average_rtt_ms: f64,
    #[serde(default)]
    recovery_success: u64,
    #[serde(default)]
    recovery_failure: u64,
    #[serde(default)]
    last_recovery_time_ms: Option<u64>,
    #[serde(default)]
    total_recovery_time_ms: u64,
    #[serde(default)]
    tls_session_total: u64,
    #[serde(default)]
    tls_handshake_total: u64,
}

#[derive(Debug, Clone)]
struct SystemSnapshot {
    cpu_usage: f32,
    memory_usage: f32,
    disk_usage: f32,
    memory_bytes: u64,
}

struct RelayTunnelRuntime {
    shutdown: watch::Sender<bool>,
    tasks: Vec<JoinHandle<()>>,
    stats: Arc<RelayTunnelStats>,
}

#[derive(Debug, Default)]
struct RelayTunnelStats {
    active_connections: AtomicU64,
    upload_bytes: AtomicU64,
    download_bytes: AtomicU64,
    total_connections: AtomicU64,
    failed_connections: AtomicU64,
    started_at_millis: AtomicU64,
}

#[derive(Debug, Clone)]
struct RelayWorkerConfig {
    worker_id: usize,
    server_host: String,
    server_port: u16,
    token: String,
    session_id: String,
    tunnel_id: String,
    protocol: String,
    target_addr: SocketAddr,
}

enum LocalTunnelRuntime {
    #[cfg(test)]
    Tcp(TunnelRuntime),
    #[cfg(test)]
    Http(HttpTunnelRuntime),
    Remote(RelayTunnelRuntime),
}

#[derive(Debug, Clone)]
struct LocalTunnelRuntimeSnapshot {
    active_connections: u64,
    upload_bytes: u64,
    download_bytes: u64,
    current_speed_bps: u64,
    runtime_seconds: u64,
    request_count: u64,
    success_count: u64,
    total_latency_ms: u64,
    recent_requests: Vec<HttpRequestRecord>,
}

impl RelayTunnelRuntime {
    async fn shutdown(self) -> Result<(), String> {
        let _ = self.shutdown.send(true);
        for task in self.tasks {
            task.abort();
            let _ = task.await;
        }
        Ok(())
    }

    fn snapshot(&self) -> LocalTunnelRuntimeSnapshot {
        let started_at = self.stats.started_at_millis.load(Ordering::Relaxed);
        let now = Utc::now().timestamp_millis().max(0) as u64;
        let runtime_seconds = now.saturating_sub(started_at) / 1000;
        let upload_bytes = self.stats.upload_bytes.load(Ordering::Relaxed);
        let download_bytes = self.stats.download_bytes.load(Ordering::Relaxed);

        LocalTunnelRuntimeSnapshot {
            active_connections: self.stats.active_connections.load(Ordering::Relaxed),
            upload_bytes,
            download_bytes,
            current_speed_bps: 0,
            runtime_seconds,
            request_count: self.stats.total_connections.load(Ordering::Relaxed),
            success_count: self
                .stats
                .total_connections
                .load(Ordering::Relaxed)
                .saturating_sub(self.stats.failed_connections.load(Ordering::Relaxed)),
            total_latency_ms: 0,
            recent_requests: Vec::new(),
        }
    }
}

impl LocalTunnelRuntime {
    async fn shutdown(self) -> Result<(), String> {
        match self {
            #[cfg(test)]
            Self::Tcp(runtime) => runtime.shutdown().await.map_err(|error| error.to_string()),
            #[cfg(test)]
            Self::Http(runtime) => runtime.shutdown().await.map_err(|error| error.to_string()),
            Self::Remote(runtime) => runtime.shutdown().await,
        }
    }

    fn snapshot(&self) -> LocalTunnelRuntimeSnapshot {
        match self {
            #[cfg(test)]
            Self::Tcp(runtime) => {
                let metrics = runtime.metrics();
                LocalTunnelRuntimeSnapshot {
                    active_connections: metrics.active_connection,
                    upload_bytes: metrics.upload,
                    download_bytes: metrics.download,
                    current_speed_bps: metrics.current_speed,
                    runtime_seconds: metrics.runtime.as_secs(),
                    request_count: 0,
                    success_count: 0,
                    total_latency_ms: 0,
                    recent_requests: Vec::new(),
                }
            }
            #[cfg(test)]
            Self::Http(runtime) => {
                let metrics = runtime.metrics();
                let http = runtime.http_metrics();
                let recent_requests = http
                    .recent_requests
                    .into_iter()
                    .rev()
                    .take(24)
                    .map(|request| HttpRequestRecord {
                        method: request.method,
                        url: request.url,
                        host: request.host,
                        status: request.status,
                        latency_ms: request.latency_ms,
                        client_ip: request.client_ip,
                        traffic_bytes: request.upload_bytes.saturating_add(request.download_bytes),
                        timestamp: request.timestamp_millis,
                    })
                    .collect::<Vec<_>>();

                LocalTunnelRuntimeSnapshot {
                    active_connections: metrics.active_connection,
                    upload_bytes: metrics.upload,
                    download_bytes: metrics.download,
                    current_speed_bps: metrics.current_speed,
                    runtime_seconds: metrics.runtime.as_secs(),
                    request_count: http.request_count,
                    success_count: http.success_count,
                    total_latency_ms: (http.average_latency_ms * http.request_count as f64) as u64,
                    recent_requests,
                }
            }
            Self::Remote(runtime) => runtime.snapshot(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredRuntime {
    version: u32,
    config: BTreeMap<String, String>,
    counters: RuntimeCounters,
    tunnels: BTreeMap<String, TunnelRecord>,
    #[serde(default)]
    domains: BTreeMap<String, RuntimeDomainRecord>,
    #[serde(default)]
    certificate_references: BTreeMap<String, RuntimeCertificateReference>,
    #[serde(default)]
    servers: BTreeMap<String, ServerRecord>,
    #[serde(default)]
    active_server_id: Option<String>,
    logs: Vec<RuntimeLog>,
}

impl Default for StoredRuntime {
    fn default() -> Self {
        Self {
            version: STORE_VERSION,
            config: default_config(),
            counters: RuntimeCounters::default(),
            tunnels: BTreeMap::new(),
            domains: BTreeMap::new(),
            certificate_references: BTreeMap::new(),
            servers: BTreeMap::new(),
            active_server_id: None,
            logs: Vec::new(),
        }
    }
}

struct RuntimeInner {
    transport: Option<Arc<TcpTransport>>,
    server_addr: Option<String>,
    session_id: Option<String>,
    connected: bool,
    started_at: Instant,
    counters: RuntimeCounters,
    config: BTreeMap<String, String>,
    tunnels: BTreeMap<String, TunnelRecord>,
    domains: BTreeMap<String, RuntimeDomainRecord>,
    certificate_references: BTreeMap<String, RuntimeCertificateReference>,
    servers: BTreeMap<String, ServerRecord>,
    active_server_id: Option<String>,
    logs: Vec<RuntimeLog>,
}

impl RuntimeInner {
    fn from_stored(stored: StoredRuntime) -> Self {
        let stored_active_server_id = stored.active_server_id;
        let mut servers = stored.servers;
        for server in servers.values_mut() {
            if matches!(
                server.status.as_str(),
                "connected" | "connecting" | "recovering"
            ) {
                server.status = "recovering".to_string();
                server.last_error = Some("客户端重启后等待自动恢复".to_string());
            }
        }
        let mut tunnels = stored.tunnels;
        for tunnel in tunnels.values_mut() {
            if matches!(
                tunnel.status.as_str(),
                "running" | "starting" | "restarting" | "stopping" | "recovering"
            ) {
                tunnel.status = "recovering".to_string();
                tunnel.upload_speed_bps = 0.0;
                tunnel.download_speed_bps = 0.0;
                tunnel.connections = 0;
            }
        }
        let active_server_id = stored_active_server_id.filter(|server_id| {
            servers
                .get(server_id)
                .map(|server| server.status == "recovering" || server.auto_connect)
                .unwrap_or(false)
        });
        let session_id = active_server_id
            .as_deref()
            .and_then(|server_id| servers.get(server_id))
            .and_then(|server| server.session_id.clone());

        Self {
            transport: None,
            server_addr: None,
            session_id,
            connected: false,
            started_at: Instant::now(),
            counters: stored.counters,
            config: stored.config,
            tunnels,
            domains: stored.domains,
            certificate_references: stored.certificate_references,
            servers,
            active_server_id,
            logs: stored.logs,
        }
    }

    fn snapshot(&self) -> StoredRuntime {
        StoredRuntime {
            version: STORE_VERSION,
            config: self.config.clone(),
            counters: self.counters.clone(),
            tunnels: self.tunnels.clone(),
            domains: self.domains.clone(),
            certificate_references: self.certificate_references.clone(),
            servers: self.servers.clone(),
            active_server_id: self.active_server_id.clone(),
            logs: self.logs.clone(),
        }
    }

    fn log(&mut self, level: &str, source: &str, message: &str) {
        self.logs.push(RuntimeLog {
            level: level.to_string(),
            source: source.to_string(),
            message: message.to_string(),
            timestamp: Utc::now().timestamp_millis(),
            tunnel_id: None,
        });

        if self.logs.len() > MAX_LOGS {
            let overflow = self.logs.len() - MAX_LOGS;
            self.logs.drain(0..overflow);
        }
    }

    fn log_tunnel(&mut self, level: &str, tunnel_id: &str, message: &str) {
        self.logs.push(RuntimeLog {
            level: level.to_string(),
            source: "tunnel".to_string(),
            message: message.to_string(),
            timestamp: Utc::now().timestamp_millis(),
            tunnel_id: Some(tunnel_id.to_string()),
        });

        if self.logs.len() > MAX_LOGS {
            let overflow = self.logs.len() - MAX_LOGS;
            self.logs.drain(0..overflow);
        }
    }

    fn sync_tunnel_state(&mut self) {
        let now = Utc::now().timestamp_millis();

        for tunnel in self.tunnels.values_mut() {
            if tunnel.status != "running" {
                continue;
            }

            if let Some(started_at) = tunnel.started_at {
                tunnel.uptime_seconds = ((now - started_at).max(0) / 1000) as u64;
            }

            tunnel.last_sample_at = now;
        }
    }

    fn sync_domain_index_for_tunnel(&mut self, tunnel_id: &str) {
        let now = Utc::now().timestamp_millis();
        self.domains
            .retain(|_, domain| domain.tunnel_id != tunnel_id);
        self.certificate_references
            .retain(|_, certificate| certificate.tunnel_id != tunnel_id);

        let Some(tunnel) = self.tunnels.get(tunnel_id) else {
            return;
        };
        let Some(domain) = tunnel
            .host
            .as_ref()
            .filter(|value| !value.trim().is_empty())
        else {
            return;
        };

        self.domains.insert(
            domain.clone(),
            RuntimeDomainRecord {
                domain: domain.clone(),
                tunnel_id: tunnel.id.clone(),
                protocol: tunnel.protocol.clone(),
                status: tunnel.status.clone(),
                created_at: tunnel.created_at,
                updated_at: now,
            },
        );

        if tunnel.protocol == "https" {
            self.certificate_references.insert(
                domain.clone(),
                RuntimeCertificateReference {
                    domain: domain.clone(),
                    tunnel_id: tunnel.id.clone(),
                    store_root: certificate_store_root(),
                    status: tunnel
                        .certificate_status
                        .clone()
                        .unwrap_or_else(|| "missing".to_string()),
                    updated_at: now,
                },
            );
        }
    }
}

pub struct ClientRuntimeState {
    storage_path: PathBuf,
    domain_repository: Option<SqliteDomainRepository>,
    inner: Mutex<RuntimeInner>,
    local_tunnels: Mutex<BTreeMap<String, LocalTunnelRuntime>>,
}

impl Default for ClientRuntimeState {
    fn default() -> Self {
        let storage_path = runtime_store_path();
        let domain_repository = SqliteDomainRepository::open(domain_store_path()).ok();
        let stored = load_stored_runtime(&storage_path);
        let mut inner = RuntimeInner::from_stored(stored);
        if let Some(repository) = &domain_repository {
            hydrate_domains_from_repository(repository, &mut inner);
            let _ = sync_domain_repository(repository, &inner.domains);
        }

        Self {
            storage_path,
            domain_repository,
            inner: Mutex::new(inner),
            local_tunnels: Mutex::new(BTreeMap::new()),
        }
    }
}

impl ClientRuntimeState {
    pub async fn connect(&self, server_addr: String, token: String) -> Result<String, String> {
        let (transport, session_id) = establish_connection(&server_addr, &token).await?;

        let mut inner = self.inner.lock().await;
        inner.transport = Some(transport);
        inner.server_addr = Some(server_addr);
        inner.session_id = Some(session_id.clone());
        inner.connected = true;
        inner.active_server_id = None;
        inner.counters.connection_total += 1;
        inner.counters.auth_success += 1;
        inner.log("info", "connection", "client connected");
        inner.log("info", "authentication", "session established");
        persist_runtime(&self.storage_path, &inner)?;
        Ok(session_id)
    }

    pub async fn list_servers(&self) -> Value {
        let inner = self.inner.lock().await;
        json!({
            "items": inner.servers.values().map(|server| server_json(server)).collect::<Vec<_>>(),
            "activeServerId": inner.active_server_id,
            "connected": inner.connected
        })
    }

    pub async fn create_server(&self, request: CreateServerRequest) -> Result<String, String> {
        let now = Utc::now().timestamp_millis();
        let host = normalize_host(&request.host)?;
        let port = normalize_port(request.port)?;
        let token = normalize_token(&request.token)?;
        let name = normalize_server_name(&request.name, &host, port);
        let id = Uuid::new_v4().to_string();
        let server = ServerRecord {
            id: id.clone(),
            name,
            kind: normalize_server_kind(request.kind),
            host,
            port,
            token,
            region: request.region.unwrap_or_default(),
            remark: request.remark.unwrap_or_default(),
            tags: normalize_tags(request.tags.unwrap_or_default()),
            heartbeat_interval: request.heartbeat_interval.unwrap_or(30),
            reconnect_interval: request.reconnect_interval.unwrap_or(5),
            auto_connect: request.auto_connect.unwrap_or(false),
            status: "disconnected".to_string(),
            last_error: None,
            last_rtt_ms: None,
            session_id: None,
            last_checked_at: None,
            last_connected_at: None,
            created_at: now,
            updated_at: now,
        };

        let mut inner = self.inner.lock().await;
        inner.servers.insert(id.clone(), server);
        inner.log("info", "server", "server configuration created");
        persist_runtime(&self.storage_path, &inner)?;
        Ok(id)
    }

    pub async fn update_server(
        &self,
        server_id: String,
        patch: UpdateServerRequest,
    ) -> Result<(), String> {
        let mut inner = self.inner.lock().await;
        let server = inner
            .servers
            .get_mut(&server_id)
            .ok_or_else(|| "server not found".to_string())?;

        if let Some(name) = patch.name {
            server.name = normalize_server_name(&name, &server.host, server.port);
        }
        if let Some(kind) = patch.kind {
            server.kind = normalize_server_kind(Some(kind));
        }
        if let Some(host) = patch.host {
            server.host = normalize_host(&host)?;
            if server.name.trim().is_empty() {
                server.name = normalize_server_name("", &server.host, server.port);
            }
        }
        if let Some(port) = patch.port {
            server.port = normalize_port(port)?;
        }
        if let Some(token) = patch.token {
            server.token = normalize_token(&token)?;
        }
        if let Some(region) = patch.region {
            server.region = region.trim().to_string();
        }
        if let Some(remark) = patch.remark {
            server.remark = remark.trim().to_string();
        }
        if let Some(tags) = patch.tags {
            server.tags = normalize_tags(tags);
        }
        if let Some(interval) = patch.heartbeat_interval {
            server.heartbeat_interval = interval.max(1);
        }
        if let Some(interval) = patch.reconnect_interval {
            server.reconnect_interval = interval.max(1);
        }
        if let Some(auto_connect) = patch.auto_connect {
            server.auto_connect = auto_connect;
        }
        server.updated_at = Utc::now().timestamp_millis();
        server.last_error = None;

        inner.log("info", "server", "server configuration updated");
        persist_runtime(&self.storage_path, &inner)?;
        Ok(())
    }

    pub async fn delete_server(&self, server_id: String) -> Result<(), String> {
        let transport = {
            let mut inner = self.inner.lock().await;
            if !inner.servers.contains_key(&server_id) {
                return Err("server not found".to_string());
            }
            let should_disconnect = inner.active_server_id.as_deref() == Some(server_id.as_str());
            inner.servers.remove(&server_id);
            if should_disconnect {
                inner.active_server_id = None;
                inner.server_addr = None;
                inner.session_id = None;
                inner.connected = false;
                inner.counters.disconnect_total += 1;
                inner.log(
                    "info",
                    "server",
                    "active server disconnected before deletion",
                );
            }
            inner.log("info", "server", "server configuration deleted");
            let transport = if should_disconnect {
                inner.transport.take()
            } else {
                None
            };
            persist_runtime(&self.storage_path, &inner)?;
            transport
        };

        if let Some(transport) = transport {
            transport
                .disconnect()
                .await
                .map_err(|error| error.to_string())?;
        }

        Ok(())
    }

    pub async fn connect_server(&self, server_id: String) -> Result<String, String> {
        let (server_addr, token) = {
            let mut inner = self.inner.lock().await;
            let server = inner
                .servers
                .get_mut(&server_id)
                .ok_or_else(|| "server not found".to_string())?;
            server.status = "connecting".to_string();
            server.last_error = None;
            server.updated_at = Utc::now().timestamp_millis();
            let server_addr = format!("{}:{}", server.host, server.port);
            let token = server.token.clone();
            persist_runtime(&self.storage_path, &inner)?;
            (server_addr, token)
        };

        match establish_connection(&server_addr, &token).await {
            Ok((transport, session_id)) => {
                let now = Utc::now().timestamp_millis();
                let mut inner = self.inner.lock().await;

                for server in inner.servers.values_mut() {
                    if server.status == "connected" || server.status == "connecting" {
                        server.status = "disconnected".to_string();
                        server.session_id = None;
                    }
                }

                if let Some(server) = inner.servers.get_mut(&server_id) {
                    server.status = "connected".to_string();
                    server.session_id = Some(session_id.clone());
                    server.last_connected_at = Some(now);
                    server.last_checked_at = Some(now);
                    server.last_error = None;
                    server.updated_at = now;
                }

                inner.transport = Some(transport);
                inner.server_addr = Some(server_addr);
                inner.session_id = Some(session_id.clone());
                inner.connected = true;
                inner.active_server_id = Some(server_id);
                inner.counters.connection_total += 1;
                inner.counters.auth_success += 1;
                inner.log("info", "server", "server connected");
                inner.log("info", "authentication", "session established");
                persist_runtime(&self.storage_path, &inner)?;
                Ok(session_id)
            }
            Err(error) => {
                let mut inner = self.inner.lock().await;
                if let Some(server) = inner.servers.get_mut(&server_id) {
                    server.status = "error".to_string();
                    server.last_error = Some(error.clone());
                    server.last_checked_at = Some(Utc::now().timestamp_millis());
                    server.updated_at = Utc::now().timestamp_millis();
                    server.session_id = None;
                }
                inner.counters.auth_failure += 1;
                inner.log(
                    "error",
                    "server",
                    &format!("server connection failed: {error}"),
                );
                persist_runtime(&self.storage_path, &inner)?;
                Err(error)
            }
        }
    }

    pub async fn disconnect_server(&self, server_id: String) -> Result<(), String> {
        self.shutdown_all_local_tunnels().await?;
        let transport = {
            let mut inner = self.inner.lock().await;
            if !inner.servers.contains_key(&server_id) {
                return Err("server not found".to_string());
            }

            let is_active = inner.active_server_id.as_deref() == Some(server_id.as_str());
            if let Some(server) = inner.servers.get_mut(&server_id) {
                server.status = "disconnected".to_string();
                server.session_id = None;
                server.last_error = None;
                server.updated_at = Utc::now().timestamp_millis();
            }

            if is_active {
                stop_running_tunnels(&mut inner);
                inner.connected = false;
                inner.active_server_id = None;
                inner.session_id = None;
                inner.server_addr = None;
                inner.counters.disconnect_total += 1;
                inner.log("info", "server", "server disconnected");
                let transport = inner.transport.take();
                persist_runtime(&self.storage_path, &inner)?;
                transport
            } else {
                persist_runtime(&self.storage_path, &inner)?;
                None
            }
        };

        if let Some(transport) = transport {
            transport
                .disconnect()
                .await
                .map_err(|error| error.to_string())?;
        }

        Ok(())
    }

    pub async fn test_server(&self, server_id: String) -> Result<Value, String> {
        let (server_addr, token) = {
            let inner = self.inner.lock().await;
            let server = inner
                .servers
                .get(&server_id)
                .ok_or_else(|| "server not found".to_string())?;
            (
                format!("{}:{}", server.host, server.port),
                server.token.clone(),
            )
        };

        let started = Instant::now();
        match establish_connection(&server_addr, &token).await {
            Ok((transport, session_id)) => {
                let rtt_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
                let _ = transport.disconnect().await;
                let now = Utc::now().timestamp_millis();
                let mut inner = self.inner.lock().await;
                if let Some(server) = inner.servers.get_mut(&server_id) {
                    server.last_rtt_ms = Some(rtt_ms);
                    server.last_checked_at = Some(now);
                    server.last_error = None;
                    if server.status == "error" {
                        server.status = "disconnected".to_string();
                    }
                    server.updated_at = now;
                }
                inner.log("info", "server", "server connection test completed");
                persist_runtime(&self.storage_path, &inner)?;
                Ok(json!({
                    "ok": true,
                    "rttMs": rtt_ms,
                    "sessionId": session_id,
                    "checkedAt": now
                }))
            }
            Err(error) => {
                let now = Utc::now().timestamp_millis();
                let mut inner = self.inner.lock().await;
                if let Some(server) = inner.servers.get_mut(&server_id) {
                    server.status = "error".to_string();
                    server.last_error = Some(error.clone());
                    server.last_checked_at = Some(now);
                    server.updated_at = now;
                }
                inner.log(
                    "error",
                    "server",
                    &format!("server connection test failed: {error}"),
                );
                persist_runtime(&self.storage_path, &inner)?;
                Ok(json!({
                    "ok": false,
                    "error": error,
                    "checkedAt": now
                }))
            }
        }
    }

    pub async fn disconnect(&self) -> Result<(), String> {
        self.shutdown_all_local_tunnels().await?;
        let transport = {
            let mut inner = self.inner.lock().await;
            stop_running_tunnels(&mut inner);
            inner.connected = false;
            inner.session_id = None;
            let active_server_id = inner.active_server_id.take();
            if let Some(server_id) = active_server_id {
                if let Some(server) = inner.servers.get_mut(&server_id) {
                    server.status = "disconnected".to_string();
                    server.session_id = None;
                    server.updated_at = Utc::now().timestamp_millis();
                }
            }
            inner.server_addr = None;
            inner.counters.disconnect_total += 1;
            inner.log("info", "connection", "client disconnected");
            persist_runtime(&self.storage_path, &inner)?;
            inner.transport.take()
        };

        if let Some(transport) = transport {
            transport
                .disconnect()
                .await
                .map_err(|error| error.to_string())?;
        }

        Ok(())
    }

    pub async fn heartbeat(&self) -> Result<u64, String> {
        let started = Instant::now();
        let response = match self
            .request(Command::HeartbeatPing, self.heartbeat_payload().await)
            .await
        {
            Ok(response) => response,
            Err(error) => {
                self.record_control_disconnect(&format!("心跳失败，准备自动恢复：{error}"))
                    .await;
                self.restore_active_server_with_backoff(&default_reconnect_delays())
                    .await?;
                self.request(Command::HeartbeatPing, self.heartbeat_payload().await)
                    .await?
            }
        };
        ensure_ok(&response)?;

        let rtt_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
        let mut inner = self.inner.lock().await;
        inner.counters.heartbeat_ping += 1;
        inner.counters.heartbeat_pong += 1;
        inner.counters.last_rtt_ms = Some(rtt_ms);
        inner.counters.average_rtt_ms = if inner.counters.heartbeat_pong == 1 {
            rtt_ms as f64
        } else {
            ((inner.counters.average_rtt_ms * (inner.counters.heartbeat_pong - 1) as f64)
                + rtt_ms as f64)
                / inner.counters.heartbeat_pong as f64
        };
        inner.log("info", "heartbeat", "heartbeat completed");
        persist_runtime(&self.storage_path, &inner)?;
        Ok(rtt_ms)
    }

    async fn heartbeat_payload(&self) -> Value {
        let inner = self.inner.lock().await;
        let registered_tunnels = inner.tunnels.keys().cloned().collect::<Vec<_>>();
        let running_tunnels = inner
            .tunnels
            .values()
            .filter(|tunnel| tunnel.status == "running")
            .count();
        let active_connections = inner
            .tunnels
            .values()
            .map(|tunnel| tunnel.connections)
            .sum::<u64>();

        json!({
            "sentAt": Utc::now().timestamp_millis(),
            "clientId": runtime_client_id(inner.active_server_id.as_deref()),
            "clientStatus": if inner.connected { "online" } else { "recovering" },
            "registeredTunnels": registered_tunnels,
            "runtimeStatus": {
                "sessionId": inner.session_id.clone(),
                "runningTunnels": running_tunnels,
                "activeConnections": active_connections,
                "requestTotal": inner.counters.request_total,
                "responseTotal": inner.counters.response_total,
                "reconnectTotal": inner.counters.reconnect_total
            }
        })
    }

    // 控制连接恢复只复用现有 AuthLogin/TunnelRegister，不引入新的协议命令。
    async fn record_control_disconnect(&self, reason: &str) {
        let transport = {
            let mut inner = self.inner.lock().await;
            inner.connected = false;
            inner.counters.heartbeat_timeout = inner.counters.heartbeat_timeout.saturating_add(1);
            inner.counters.disconnect_total = inner.counters.disconnect_total.saturating_add(1);
            let active_server_id = inner.active_server_id.clone();
            if let Some(server_id) = active_server_id.as_deref() {
                if let Some(server) = inner.servers.get_mut(server_id) {
                    server.status = "recovering".to_string();
                    server.last_error = Some(reason.to_string());
                    server.last_checked_at = Some(Utc::now().timestamp_millis());
                    server.updated_at = Utc::now().timestamp_millis();
                }
            }
            inner.log("warn", "reconnect", reason);
            let transport = inner.transport.take();
            let _ = persist_runtime(&self.storage_path, &inner);
            transport
        };

        if let Some(transport) = transport {
            let _ = transport.disconnect().await;
        }
    }

    async fn restore_active_server_with_backoff(
        &self,
        delays: &[Duration],
    ) -> Result<String, String> {
        let (server_id, server_addr, token, previous_session_id, running_tunnels) = {
            let mut inner = self.inner.lock().await;
            let server_id = inner
                .active_server_id
                .clone()
                .ok_or_else(|| "没有可恢复的活动服务器".to_string())?;
            let server = inner
                .servers
                .get(&server_id)
                .ok_or_else(|| "活动服务器配置不存在".to_string())?;
            let server_addr = format!("{}:{}", server.host, server.port);
            let token = server.token.clone();
            let previous_session_id = inner
                .session_id
                .clone()
                .or_else(|| server.session_id.clone());
            let running_tunnels = inner
                .tunnels
                .values()
                .filter(|tunnel| matches!(tunnel.status.as_str(), "running" | "recovering"))
                .cloned()
                .collect::<Vec<_>>();
            if let Some(server) = inner.servers.get_mut(&server_id) {
                server.status = "recovering".to_string();
                server.updated_at = Utc::now().timestamp_millis();
            }
            inner.log("info", "reconnect", "开始恢复控制连接");
            persist_runtime(&self.storage_path, &inner)?;
            (
                server_id,
                server_addr,
                token,
                previous_session_id,
                running_tunnels,
            )
        };

        let recovery_started = Instant::now();
        self.shutdown_all_local_tunnels().await?;

        let mut last_error = None;
        for delay in delays {
            tokio::select! {
                _ = sleep(*delay) => {}
            }

            match establish_connection_with_session(
                &server_addr,
                &token,
                previous_session_id.clone(),
                Some(runtime_client_id(Some(&server_id))),
            )
            .await
            {
                Ok((transport, session_id)) => {
                    self.finish_control_recovery(
                        server_id,
                        server_addr,
                        token,
                        transport,
                        session_id.clone(),
                        running_tunnels,
                        recovery_started.elapsed(),
                    )
                    .await?;
                    return Ok(session_id);
                }
                Err(error) => {
                    last_error = Some(error.clone());
                    let mut inner = self.inner.lock().await;
                    inner.log(
                        "warn",
                        "reconnect",
                        &format!("控制连接恢复失败，等待下一次退避：{error}"),
                    );
                    let _ = persist_runtime(&self.storage_path, &inner);
                }
            }
        }

        {
            let mut inner = self.inner.lock().await;
            inner.counters.recovery_failure = inner.counters.recovery_failure.saturating_add(1);
            inner.log("error", "reconnect", "控制连接恢复失败");
            let _ = persist_runtime(&self.storage_path, &inner);
        }

        Err(format!(
            "控制连接恢复失败：{}",
            last_error.unwrap_or_else(|| "没有可用重试机会".to_string())
        ))
    }

    async fn finish_control_recovery(
        &self,
        server_id: String,
        server_addr: String,
        token: String,
        transport: Arc<TcpTransport>,
        session_id: String,
        running_tunnels: Vec<TunnelRecord>,
        recovery_elapsed: Duration,
    ) -> Result<(), String> {
        let recovery_time_ms = recovery_elapsed.as_millis().min(u128::from(u64::MAX)) as u64;
        {
            let now = Utc::now().timestamp_millis();
            let mut inner = self.inner.lock().await;
            if let Some(server) = inner.servers.get_mut(&server_id) {
                server.status = "connected".to_string();
                server.session_id = Some(session_id.clone());
                server.last_error = None;
                server.last_checked_at = Some(now);
                server.last_connected_at = Some(now);
                server.updated_at = now;
            }
            inner.transport = Some(transport);
            inner.server_addr = Some(server_addr.clone());
            inner.session_id = Some(session_id.clone());
            inner.connected = true;
            inner.active_server_id = Some(server_id.clone());
            inner.counters.reconnect_total = inner.counters.reconnect_total.saturating_add(1);
            inner.counters.auth_success = inner.counters.auth_success.saturating_add(1);
            inner.counters.recovery_success = inner.counters.recovery_success.saturating_add(1);
            inner.counters.last_recovery_time_ms = Some(recovery_time_ms);
            inner.counters.total_recovery_time_ms = inner
                .counters
                .total_recovery_time_ms
                .saturating_add(recovery_time_ms);
            inner.log("info", "reconnect", "控制连接恢复成功");
            persist_runtime(&self.storage_path, &inner)?;
        }

        for tunnel in running_tunnels {
            self.request_required(Command::TunnelRegister, tunnel_control_payload(&tunnel))
                .await?;
            match start_remote_tunnel_runtime(&tunnel, &server_addr, &token, &session_id).await {
                Ok(runtime) => {
                    self.local_tunnels
                        .lock()
                        .await
                        .insert(tunnel.id.clone(), runtime);
                    let mut inner = self.inner.lock().await;
                    let now = Utc::now().timestamp_millis();
                    if let Some(record) = inner.tunnels.get_mut(&tunnel.id) {
                        record.status = "running".to_string();
                        record.started_at = record.started_at.or(Some(now));
                        record.updated_at = now;
                    }
                    inner.log_tunnel("info", &tunnel.id, "tunnel recovered after reconnect");
                    persist_runtime(&self.storage_path, &inner)?;
                }
                Err(error) => {
                    self.record_tunnel_start_failure(&tunnel.id, &error).await;
                    return Err(error);
                }
            }
        }

        Ok(())
    }

    pub async fn create_tunnel(
        &self,
        local_port: u16,
        remote_port: u16,
        protocol: String,
        local_host: Option<String>,
        host: Option<String>,
        path: Option<String>,
    ) -> Result<String, String> {
        let tunnel_id = Uuid::new_v4().to_string();
        let protocol = normalize_protocol(&protocol)?;
        let local_host = local_host.unwrap_or_else(|| "127.0.0.1".to_string());
        let path = path.map(|value| normalize_http_path(&value)).or_else(|| {
            if protocol == "http" || protocol == "https" {
                Some("/".to_string())
            } else {
                None
            }
        });

        self.ensure_connected_server().await?;

        self.request_if_connected(
            Command::TunnelCreate,
            json!({
                "id": tunnel_id,
                "localHost": local_host.clone(),
                "localPort": local_port,
                "remotePort": remote_port,
                "protocol": protocol.clone(),
                "host": host.clone(),
                "path": path.clone()
            }),
        )
        .await?;

        let now = Utc::now().timestamp_millis();
        let tls_version = if protocol == "https" {
            Some("TLS".to_string())
        } else {
            None
        };
        let mut inner = self.inner.lock().await;
        inner.tunnels.insert(
            tunnel_id.clone(),
            TunnelRecord {
                id: tunnel_id.clone(),
                name: format!("{}-{}", protocol, local_port),
                protocol,
                status: "stopped".to_string(),
                local_host,
                local_port,
                remote_port,
                host,
                path,
                upload_speed_bps: 0.0,
                download_speed_bps: 0.0,
                upload_bytes: 0,
                download_bytes: 0,
                connections: 0,
                uptime_seconds: 0,
                request_count: 0,
                success_count: 0,
                total_latency_ms: 0,
                started_at: None,
                created_at: now,
                updated_at: now,
                last_sample_at: now,
                tls_session_count: 0,
                tls_handshake_count: 0,
                tls_error_count: 0,
                tls_version,
                certificate_status: None,
                certificate_expire_days: None,
                certificate_issuer: None,
                recent_requests: Vec::new(),
            },
        );
        inner.sync_domain_index_for_tunnel(&tunnel_id);
        sync_runtime_domains(&self.domain_repository, &inner.domains)?;
        inner.log("info", "tunnel", "tunnel configuration created");
        persist_runtime(&self.storage_path, &inner)?;
        Ok(tunnel_id)
    }

    pub async fn start_tunnel(&self, tunnel_id: String) -> Result<(), String> {
        self.ensure_tunnel_exists(&tunnel_id).await?;
        if let Err(error) = self.ensure_connected_server().await {
            self.record_tunnel_start_failure(&tunnel_id, &error).await;
            return Err(error);
        }

        let (tunnel, server_addr, token, session_id) =
            self.active_relay_context(&tunnel_id).await?;

        if let Err(error) = self
            .request_required(Command::TunnelStart, tunnel_control_payload(&tunnel))
            .await
        {
            let message = explain_server_control_error(&error);
            self.record_server_control_failure(&message).await;
            self.record_tunnel_start_failure(&tunnel_id, &message).await;
            return Err(message);
        }

        if !self.local_tunnels.lock().await.contains_key(&tunnel_id) {
            match start_remote_tunnel_runtime(&tunnel, &server_addr, &token, &session_id).await {
                Ok(runtime) => {
                    self.local_tunnels
                        .lock()
                        .await
                        .insert(tunnel_id.clone(), runtime);
                }
                Err(error) => {
                    let message = explain_local_runtime_error(&tunnel, &error);
                    let _ = self
                        .request_if_connected(Command::TunnelStop, json!({ "id": tunnel_id }))
                        .await;
                    self.record_tunnel_start_failure(&tunnel_id, &message).await;
                    return Err(message);
                }
            }
        }

        let mut inner = self.inner.lock().await;
        let now = Utc::now().timestamp_millis();
        let name = if let Some(tunnel) = inner.tunnels.get_mut(&tunnel_id) {
            tunnel.status = "running".to_string();
            tunnel.started_at = Some(now);
            tunnel.updated_at = now;
            tunnel.last_sample_at = now;
            tunnel.name.clone()
        } else {
            return Err("tunnel not found".to_string());
        };
        inner.sync_domain_index_for_tunnel(&tunnel_id);
        sync_runtime_domains(&self.domain_repository, &inner.domains)?;
        inner.log_tunnel("info", &tunnel_id, &format!("tunnel started: {name}"));
        persist_runtime(&self.storage_path, &inner)?;
        Ok(())
    }

    pub async fn stop_tunnel(&self, tunnel_id: String) -> Result<(), String> {
        self.ensure_tunnel_exists(&tunnel_id).await?;
        self.shutdown_local_tunnel(&tunnel_id).await?;
        self.request_required(Command::TunnelStop, json!({ "id": tunnel_id }))
            .await?;

        let mut inner = self.inner.lock().await;
        let name = if let Some(tunnel) = inner.tunnels.get_mut(&tunnel_id) {
            tunnel.status = "stopped".to_string();
            tunnel.upload_speed_bps = 0.0;
            tunnel.download_speed_bps = 0.0;
            tunnel.connections = 0;
            tunnel.started_at = None;
            tunnel.updated_at = Utc::now().timestamp_millis();
            tunnel.name.clone()
        } else {
            return Err("tunnel not found".to_string());
        };
        inner.sync_domain_index_for_tunnel(&tunnel_id);
        sync_runtime_domains(&self.domain_repository, &inner.domains)?;
        inner.log_tunnel("info", &tunnel_id, &format!("tunnel stopped: {name}"));
        persist_runtime(&self.storage_path, &inner)?;
        Ok(())
    }

    pub async fn restart_tunnel(&self, tunnel_id: String) -> Result<(), String> {
        self.ensure_tunnel_exists(&tunnel_id).await?;
        self.shutdown_local_tunnel(&tunnel_id).await?;
        self.start_tunnel(tunnel_id.clone()).await?;
        let mut inner = self.inner.lock().await;
        if let Some(tunnel) = inner.tunnels.get(&tunnel_id) {
            let name = tunnel.name.clone();
            inner.log_tunnel("info", &tunnel_id, &format!("tunnel restarted: {name}"));
            persist_runtime(&self.storage_path, &inner)?;
        }
        Ok(())
    }

    pub async fn edit_tunnel(
        &self,
        tunnel_id: String,
        patch: UpdateTunnelRequest,
    ) -> Result<(), String> {
        self.ensure_tunnel_exists(&tunnel_id).await?;

        let control_payload = {
            let mut inner = self.inner.lock().await;
            let tunnel = inner
                .tunnels
                .get_mut(&tunnel_id)
                .ok_or_else(|| "tunnel not found".to_string())?;
            if let Some(name) = patch.name {
                tunnel.name = name;
            }
            if let Some(protocol) = patch.protocol {
                tunnel.protocol = normalize_protocol(&protocol)?;
            }
            if let Some(local_host) = patch.local_host {
                tunnel.local_host = local_host;
            }
            if let Some(local_port) = patch.local_port {
                tunnel.local_port = local_port;
            }
            if let Some(remote_port) = patch.remote_port {
                tunnel.remote_port = remote_port;
            }
            if let Some(host) = patch.host {
                tunnel.host = Some(host);
            }
            if let Some(path) = patch.path {
                tunnel.path = Some(normalize_http_path(&path));
            }
            tunnel.tls_version = if tunnel.protocol == "https" {
                Some("TLS".to_string())
            } else {
                None
            };
            tunnel.updated_at = Utc::now().timestamp_millis();
            let control_payload = tunnel_control_payload(tunnel);
            inner.sync_domain_index_for_tunnel(&tunnel_id);
            sync_runtime_domains(&self.domain_repository, &inner.domains)?;
            inner.log("info", "tunnel", "tunnel configuration updated");
            persist_runtime(&self.storage_path, &inner)?;
            control_payload
        };

        self.request_if_connected(Command::TunnelRegister, control_payload)
            .await?;
        Ok(())
    }

    pub async fn delete_tunnel(&self, tunnel_id: String) -> Result<(), String> {
        self.ensure_tunnel_exists(&tunnel_id).await?;
        self.shutdown_local_tunnel(&tunnel_id).await?;
        self.request_if_connected(Command::TunnelStop, json!({ "id": tunnel_id }))
            .await?;

        let mut inner = self.inner.lock().await;
        inner.tunnels.remove(&tunnel_id);
        inner.sync_domain_index_for_tunnel(&tunnel_id);
        sync_runtime_domains(&self.domain_repository, &inner.domains)?;
        inner.log_tunnel("info", &tunnel_id, "tunnel configuration deleted");
        persist_runtime(&self.storage_path, &inner)?;
        Ok(())
    }

    pub async fn config(&self) -> Value {
        let inner = self.inner.lock().await;
        json!(&inner.config)
    }

    pub async fn set_config(&self, key: String, value: String) -> Result<(), String> {
        let mut inner = self.inner.lock().await;
        inner.config.insert(key, value);
        inner.log("info", "settings", "runtime config updated");
        persist_runtime(&self.storage_path, &inner)?;
        Ok(())
    }

    pub async fn statistics(&self) -> Value {
        self.sync_local_tunnel_metrics().await;
        let mut inner = self.inner.lock().await;
        inner.sync_tunnel_state();
        statistics_json(&inner)
    }

    pub async fn dashboard(&self) -> Value {
        self.sync_local_tunnel_metrics().await;
        let mut inner = self.inner.lock().await;
        inner.sync_tunnel_state();
        dashboard_json(&inner)
    }

    pub async fn health(&self) -> Value {
        let inner = self.inner.lock().await;
        health_json(&inner)
    }

    pub async fn metrics(&self) -> Value {
        self.sync_local_tunnel_metrics().await;
        let mut inner = self.inner.lock().await;
        inner.sync_tunnel_state();
        let now = Utc::now().timestamp_millis();
        let traffic = runtime_traffic(&inner);
        let http_requests_total = inner
            .tunnels
            .values()
            .filter(|tunnel| tunnel.protocol == "http" || tunnel.protocol == "https")
            .map(|tunnel| tunnel.request_count)
            .sum::<u64>();
        let http_active_requests = inner
            .tunnels
            .values()
            .filter(|tunnel| tunnel.protocol == "http" || tunnel.protocol == "https")
            .map(|tunnel| tunnel.connections)
            .sum::<u64>();
        let http_latency_total = inner
            .tunnels
            .values()
            .filter(|tunnel| tunnel.protocol == "http" || tunnel.protocol == "https")
            .map(|tunnel| tunnel.total_latency_ms)
            .sum::<u64>();
        let http_bandwidth_bytes = inner
            .tunnels
            .values()
            .filter(|tunnel| tunnel.protocol == "http" || tunnel.protocol == "https")
            .map(|tunnel| tunnel.upload_bytes.saturating_add(tunnel.download_bytes))
            .sum::<u64>();
        json!([
            metric(
                "gate.connection.current",
                "Current connections",
                "gauge",
                "connection",
                "count",
                if inner.connected { 1.0 } else { 0.0 },
                now
            ),
            metric(
                "gate.connection.rtt.average",
                "Average RTT",
                "gauge",
                "heartbeat",
                "milliseconds",
                inner.counters.average_rtt_ms,
                now
            ),
            metric(
                "gate.reconnect.count",
                "Reconnect count",
                "counter",
                "recovery",
                "count",
                inner.counters.reconnect_total as f64,
                now
            ),
            metric(
                "gate.recovery.time.last",
                "Last recovery time",
                "gauge",
                "recovery",
                "milliseconds",
                inner.counters.last_recovery_time_ms.unwrap_or_default() as f64,
                now
            ),
            metric(
                "gate.tunnel.count",
                "Tunnel count",
                "gauge",
                "tunnel",
                "count",
                inner.tunnels.len() as f64,
                now
            ),
            metric(
                "gate.tunnel.uptime.total",
                "Total tunnel uptime",
                "gauge",
                "tunnel",
                "seconds",
                inner
                    .tunnels
                    .values()
                    .map(|tunnel| tunnel.uptime_seconds)
                    .sum::<u64>() as f64,
                now
            ),
            metric(
                "gate.http.requests_total",
                "HTTP requests total",
                "counter",
                "tunnel",
                "count",
                http_requests_total as f64,
                now
            ),
            metric(
                "gate.http.active_requests",
                "Active HTTP requests",
                "gauge",
                "tunnel",
                "count",
                http_active_requests as f64,
                now
            ),
            metric(
                "gate.http.latency.average",
                "Average HTTP latency",
                "gauge",
                "tunnel",
                "milliseconds",
                if http_requests_total == 0 {
                    0.0
                } else {
                    http_latency_total as f64 / http_requests_total as f64
                },
                now
            ),
            metric(
                "gate.http.bandwidth.bytes",
                "HTTP bandwidth",
                "counter",
                "tunnel",
                "bytes",
                http_bandwidth_bytes as f64,
                now
            ),
            metric(
                "gate.traffic.bytes.total",
                "Runtime total traffic",
                "counter",
                "runtime",
                "bytes",
                traffic.total_bytes as f64,
                now
            ),
            metric(
                "gate.tls.handshake.count",
                "TLS handshakes",
                "counter",
                "runtime",
                "count",
                runtime_tls(&inner).handshake_count as f64,
                now
            )
        ])
    }

    pub async fn logs(&self) -> Value {
        self.sync_local_tunnel_metrics().await;
        let mut inner = self.inner.lock().await;
        inner.sync_tunnel_state();
        json!(&inner.logs)
    }

    pub async fn clear_logs(&self) -> Result<(), String> {
        let mut inner = self.inner.lock().await;
        inner.logs.clear();
        persist_runtime(&self.storage_path, &inner)?;
        Ok(())
    }

    pub async fn runtime_store_report(&self) -> Value {
        let inner = self.inner.lock().await;
        json!({
            "path": self.storage_path,
            "domainDatabasePath": domain_store_path(),
            "version": STORE_VERSION,
            "persisted": {
                "tunnel": { "ok": true, "count": inner.tunnels.len() },
                "runtime": { "ok": true, "count": 1 },
                "server": { "ok": true, "count": inner.servers.len(), "activeServerId": inner.active_server_id },
                "domain": { "ok": self.domain_repository.is_some(), "count": inner.domains.len(), "sqlitePath": domain_store_path() },
                "certificate": { "ok": true, "count": inner.certificate_references.len(), "storeRoot": certificate_store_root() },
                "settings": { "ok": true, "count": inner.config.len() },
                "logs": { "ok": true, "count": inner.logs.len() },
                "counters": { "ok": true, "count": 1 }
            },
            "restartRecovery": {
                "servers": inner.servers.keys().collect::<Vec<_>>(),
                "tunnels": inner.tunnels.keys().collect::<Vec<_>>(),
                "domains": inner.domains.keys().collect::<Vec<_>>(),
                "certificates": inner.certificate_references.keys().collect::<Vec<_>>()
            }
        })
    }

    pub async fn startup_diagnostics(&self) -> Value {
        let mut inner = self.inner.lock().await;
        let report = startup_diagnostics_json(&self.storage_path, &inner);
        inner.log("info", "diagnostics", "startup diagnostics completed");
        let _ = persist_runtime(&self.storage_path, &inner);
        report
    }

    pub async fn recover_after_startup(&self) -> Result<Option<String>, String> {
        let should_recover = {
            let mut inner = self.inner.lock().await;
            if inner.active_server_id.is_none() {
                let auto_server_id = inner
                    .servers
                    .iter()
                    .find(|(_, server)| server.auto_connect)
                    .map(|(id, _)| id.clone());
                inner.active_server_id = auto_server_id;
            }

            let Some(server_id) = inner.active_server_id.clone() else {
                return Ok(None);
            };
            let Some(server) = inner.servers.get(&server_id) else {
                inner.active_server_id = None;
                return Ok(None);
            };

            server.auto_connect || matches!(server.status.as_str(), "recovering" | "connected")
        };

        if !should_recover {
            return Ok(None);
        }

        self.restore_active_server_with_backoff(&default_reconnect_delays())
            .await
            .map(Some)
    }

    async fn shutdown_local_tunnel(&self, tunnel_id: &str) -> Result<(), String> {
        let runtime = self.local_tunnels.lock().await.remove(tunnel_id);
        if let Some(runtime) = runtime {
            runtime.shutdown().await?;
        }
        Ok(())
    }

    async fn shutdown_all_local_tunnels(&self) -> Result<(), String> {
        let runtimes = {
            let mut local_tunnels = self.local_tunnels.lock().await;
            std::mem::take(&mut *local_tunnels)
        };

        for runtime in runtimes.into_values() {
            runtime.shutdown().await?;
        }

        Ok(())
    }

    async fn record_tunnel_start_failure(&self, tunnel_id: &str, reason: &str) {
        let mut inner = self.inner.lock().await;
        if let Some(tunnel) = inner.tunnels.get_mut(tunnel_id) {
            tunnel.status = "stopped".to_string();
            tunnel.upload_speed_bps = 0.0;
            tunnel.download_speed_bps = 0.0;
            tunnel.connections = 0;
            tunnel.started_at = None;
            tunnel.updated_at = Utc::now().timestamp_millis();
        }
        inner.sync_domain_index_for_tunnel(tunnel_id);
        let _ = sync_runtime_domains(&self.domain_repository, &inner.domains);
        inner.log_tunnel(
            "error",
            tunnel_id,
            &format!("tunnel start failed: {reason}"),
        );
        let _ = persist_runtime(&self.storage_path, &inner);
    }

    async fn record_server_control_failure(&self, reason: &str) {
        let transport = {
            let mut inner = self.inner.lock().await;
            let active_server_id = inner.active_server_id.take();
            if let Some(server_id) = active_server_id.as_deref() {
                if let Some(server) = inner.servers.get_mut(server_id) {
                    server.status = "error".to_string();
                    server.last_error = Some(reason.to_string());
                    server.session_id = None;
                    server.updated_at = Utc::now().timestamp_millis();
                }
            }
            inner.connected = false;
            inner.session_id = None;
            inner.server_addr = None;
            inner.counters.disconnect_total = inner.counters.disconnect_total.saturating_add(1);
            inner.log("error", "server", reason);
            let transport = inner.transport.take();
            let _ = persist_runtime(&self.storage_path, &inner);
            transport
        };

        if let Some(transport) = transport {
            let _ = transport.disconnect().await;
        }
    }

    async fn sync_local_tunnel_metrics(&self) {
        let snapshots = {
            let local_tunnels = self.local_tunnels.lock().await;
            local_tunnels
                .iter()
                .map(|(id, runtime)| (id.clone(), runtime.snapshot()))
                .collect::<Vec<_>>()
        };

        if snapshots.is_empty() {
            return;
        }

        let mut inner = self.inner.lock().await;
        let now = Utc::now().timestamp_millis();
        for (id, snapshot) in snapshots {
            let Some(tunnel) = inner.tunnels.get_mut(&id) else {
                continue;
            };

            tunnel.connections = snapshot.active_connections;
            tunnel.upload_bytes = snapshot.upload_bytes;
            tunnel.download_bytes = snapshot.download_bytes;
            tunnel.upload_speed_bps = snapshot.current_speed_bps as f64;
            tunnel.download_speed_bps = 0.0;
            tunnel.uptime_seconds = snapshot.runtime_seconds;
            tunnel.request_count = snapshot.request_count;
            tunnel.success_count = snapshot.success_count;
            tunnel.total_latency_ms = snapshot.total_latency_ms;
            tunnel.recent_requests = snapshot.recent_requests;
            tunnel.last_sample_at = now;
        }

        let _ = persist_runtime(&self.storage_path, &inner);
    }

    async fn active_relay_context(
        &self,
        tunnel_id: &str,
    ) -> Result<(TunnelRecord, String, String, String), String> {
        let inner = self.inner.lock().await;
        let tunnel = inner
            .tunnels
            .get(tunnel_id)
            .cloned()
            .ok_or_else(|| "tunnel not found".to_string())?;
        let session_id = inner
            .session_id
            .clone()
            .ok_or_else(|| tunnel_server_not_ready_message(&inner))?;
        let server_id = inner
            .active_server_id
            .as_deref()
            .ok_or_else(|| tunnel_server_not_ready_message(&inner))?;
        let server = inner
            .servers
            .get(server_id)
            .ok_or_else(|| tunnel_server_not_ready_message(&inner))?;
        let server_addr = format!("{}:{}", server.host, server.port);
        Ok((tunnel, server_addr, server.token.clone(), session_id))
    }

    async fn ensure_tunnel_exists(&self, tunnel_id: &str) -> Result<(), String> {
        let inner = self.inner.lock().await;
        if inner.tunnels.contains_key(tunnel_id) {
            Ok(())
        } else {
            Err("tunnel not found".to_string())
        }
    }

    async fn ensure_connected_server(&self) -> Result<(), String> {
        let inner = self.inner.lock().await;
        if inner.connected && inner.transport.is_some() && inner.active_server_id.is_some() {
            Ok(())
        } else {
            Err(tunnel_server_not_ready_message(&inner))
        }
    }

    async fn request_if_connected(&self, command: Command, body: Value) -> Result<(), String> {
        let connected = {
            let inner = self.inner.lock().await;
            inner.transport.is_some()
        };

        if !connected {
            return Ok(());
        }

        let response = self.request(command, body).await?;
        ensure_ok(&response)
    }

    async fn request_required(&self, command: Command, body: Value) -> Result<(), String> {
        let response = self.request(command, body).await?;
        ensure_ok(&response)
    }

    async fn request(&self, command: Command, body: Value) -> Result<Value, String> {
        let transport = {
            let mut inner = self.inner.lock().await;
            inner.counters.request_total += 1;
            inner.transport.clone()
        }
        .ok_or_else(|| "runtime backend is not connected".to_string())?;

        let response = send_request(&transport, command, body).await?;
        let mut inner = self.inner.lock().await;
        inner.counters.response_total += 1;
        persist_runtime(&self.storage_path, &inner)?;
        Ok(response)
    }
}

#[cfg(test)]
async fn start_local_tunnel_runtime(tunnel: &TunnelRecord) -> Result<LocalTunnelRuntime, String> {
    let listen_addr = local_listen_addr(tunnel.remote_port);
    let target_addr = resolve_target_addr(&tunnel.local_host, tunnel.local_port).await?;
    verify_target_reachable(target_addr).await?;
    let timeout = local_runtime_timeout();

    match tunnel.protocol.as_str() {
        "tcp" => {
            let runtime = TunnelRuntime::new(
                RuntimeConfig::builder()
                    .name(tunnel.name.clone())
                    .listen_addr(listen_addr)
                    .target_addr(target_addr)
                    .timeout(timeout)
                    .build(),
            );
            runtime.start().await.map_err(|error| {
                format!(
                    "无法启动本地 TCP 监听 {} -> {}：{}",
                    listen_addr, target_addr, error
                )
            })?;
            Ok(LocalTunnelRuntime::Tcp(runtime))
        }
        "http" => {
            let mut route = HttpRouteConfig::new(tunnel.name.clone(), target_addr)
                .path_prefix(tunnel.path.as_deref().unwrap_or("/"));
            if let Some(host) = tunnel
                .host
                .as_deref()
                .filter(|value| !value.trim().is_empty())
            {
                route = route.host(host);
            }

            let runtime = HttpTunnelRuntime::new(
                HttpRuntimeConfig::builder()
                    .name(tunnel.name.clone())
                    .listen_addr(listen_addr)
                    .route(route)
                    .timeout(timeout)
                    .build(),
            );
            runtime.start().await.map_err(|error| {
                format!(
                    "无法启动本地 HTTP 监听 {} -> {}：{}",
                    listen_addr, target_addr, error
                )
            })?;
            Ok(LocalTunnelRuntime::Http(runtime))
        }
        "https" => Err(
            "HTTPS 隧道需要可用证书后才能启动本地 TLS 监听；请先用 HTTP 隧道完成连通性测试。"
                .to_string(),
        ),
        protocol => Err(format!("不支持的 Tunnel 协议：{protocol}")),
    }
}

async fn start_remote_tunnel_runtime(
    tunnel: &TunnelRecord,
    server_addr: &str,
    token: &str,
    session_id: &str,
) -> Result<LocalTunnelRuntime, String> {
    let (server_host, server_port) = parse_server_addr(server_addr)?;
    let target_addr = resolve_target_addr(&tunnel.local_host, tunnel.local_port).await?;
    verify_target_reachable(target_addr).await?;

    let (shutdown, shutdown_rx) = watch::channel(false);
    let stats = Arc::new(RelayTunnelStats::default());
    stats.started_at_millis.store(
        Utc::now().timestamp_millis().max(0) as u64,
        Ordering::Relaxed,
    );

    let worker_count = relay_workers_per_tunnel();
    let mut tasks = Vec::with_capacity(worker_count);
    for worker_id in 0..worker_count {
        let config = RelayWorkerConfig {
            worker_id,
            server_host: server_host.clone(),
            server_port,
            token: token.to_string(),
            session_id: session_id.to_string(),
            tunnel_id: tunnel.id.clone(),
            protocol: tunnel.protocol.clone(),
            target_addr,
        };
        let worker_stats = Arc::clone(&stats);
        let worker_shutdown = shutdown_rx.clone();
        tasks.push(tokio::spawn(async move {
            relay_worker_loop(config, worker_stats, worker_shutdown).await;
        }));
    }

    Ok(LocalTunnelRuntime::Remote(RelayTunnelRuntime {
        shutdown,
        tasks,
        stats,
    }))
}

async fn relay_worker_loop(
    config: RelayWorkerConfig,
    stats: Arc<RelayTunnelStats>,
    mut shutdown: watch::Receiver<bool>,
) {
    // relay worker 失败时按固定退避序列重连，成功转发后立即补回空闲 worker。
    let mut retry_attempt = 0_usize;
    loop {
        if *shutdown.borrow() {
            break;
        }

        let delay =
            match relay_worker_once(config.clone(), Arc::clone(&stats), shutdown.clone()).await {
                Ok(()) => {
                    retry_attempt = 0;
                    Duration::ZERO
                }
                Err(_) => {
                    stats.failed_connections.fetch_add(1, Ordering::Relaxed);
                    let delay = reconnect_delay(retry_attempt);
                    retry_attempt = retry_attempt.saturating_add(1);
                    delay
                }
            };

        if *shutdown.borrow() {
            break;
        }

        if delay.is_zero() {
            tokio::task::yield_now().await;
        } else {
            tokio::select! {
                _ = shutdown.changed() => {}
                _ = sleep(delay) => {}
            }
        }
    }
}

async fn relay_worker_once(
    config: RelayWorkerConfig,
    stats: Arc<RelayTunnelStats>,
    mut shutdown: watch::Receiver<bool>,
) -> Result<(), String> {
    let mut stream = TcpStream::connect((config.server_host.as_str(), config.server_port))
        .await
        .map_err(|error| format!("无法连接服务端 relay 入口：{error}"))?;
    stream
        .set_nodelay(true)
        .map_err(|error| format!("无法设置 relay TCP_NODELAY：{error}"))?;

    let protocol = ProtocolBuilder::new().build();
    let connect = Message::request(
        Command::TunnelRelayConnect,
        Body::Json(json!({
            "token": &config.token,
            "sessionId": &config.session_id,
            "tunnelId": &config.tunnel_id,
            "protocol": &config.protocol,
            "workerId": config.worker_id
        })),
        Metadata::default(),
    );
    write_protocol_message(&mut stream, &protocol, &connect).await?;

    let response = read_protocol_message(&mut stream, &protocol)
        .await?
        .ok_or_else(|| "服务端关闭了 relay 握手连接".to_string())?;
    match response.body {
        Body::Json(value) => ensure_ok(&value)?,
        _ => return Err("服务端 relay 握手响应不是 JSON".to_string()),
    }

    let start = tokio::select! {
        result = read_protocol_message(&mut stream, &protocol) => {
            result?.ok_or_else(|| "服务端关闭了空闲 relay worker".to_string())?
        }
        _ = shutdown.changed() => return Ok(()),
    };

    if start.header.command != Command::TunnelRelayStart {
        return Err(format!("收到不支持的 relay 指令：{}", start.header.command));
    }

    stats.total_connections.fetch_add(1, Ordering::Relaxed);
    let mut target = TcpStream::connect(config.target_addr)
        .await
        .map_err(|error| format!("无法连接本地服务 {}：{error}", config.target_addr))?;
    target
        .set_nodelay(true)
        .map_err(|error| format!("无法设置本地服务 TCP_NODELAY：{error}"))?;

    stats.active_connections.fetch_add(1, Ordering::Relaxed);
    let copy = tokio::io::copy_bidirectional(&mut stream, &mut target);
    tokio::pin!(copy);
    let result = tokio::select! {
        result = &mut copy => result.map_err(|error| format!("relay 双向转发失败：{error}")),
        _ = shutdown.changed() => Ok((0, 0)),
    };
    stats.active_connections.fetch_sub(1, Ordering::Relaxed);

    let (upload_bytes, download_bytes) = result?;
    stats
        .upload_bytes
        .fetch_add(upload_bytes, Ordering::Relaxed);
    stats
        .download_bytes
        .fetch_add(download_bytes, Ordering::Relaxed);

    Ok(())
}

#[cfg(test)]
fn local_listen_addr(remote_port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), remote_port)
}

async fn resolve_target_addr(host: &str, port: u16) -> Result<SocketAddr, String> {
    let host = host.trim();
    if host.is_empty() {
        return Err("本地服务地址不能为空".to_string());
    }

    let mut addrs = tokio::net::lookup_host((host, port))
        .await
        .map_err(|error| format!("无法解析本地服务地址 {host}:{port}：{error}"))?;
    addrs
        .next()
        .ok_or_else(|| format!("无法解析本地服务地址 {host}:{port}"))
}

async fn verify_target_reachable(target_addr: SocketAddr) -> Result<(), String> {
    match tokio::time::timeout(
        Duration::from_secs(2),
        tokio::net::TcpStream::connect(target_addr),
    )
    .await
    {
        Ok(Ok(_stream)) => Ok(()),
        Ok(Err(error)) => Err(format!("本地服务不可访问 {target_addr}：{error}")),
        Err(_) => Err(format!("本地服务连接超时 {target_addr}")),
    }
}

#[cfg(test)]
fn local_runtime_timeout() -> TimeoutConfig {
    TimeoutConfig::builder()
        .connect_timeout(Duration::from_secs(3))
        .idle_timeout(Duration::from_secs(60))
        .shutdown_timeout(Duration::from_secs(3))
        .build()
}

fn tunnel_server_not_ready_message(inner: &RuntimeInner) -> String {
    if inner.servers.is_empty() {
        return "没有可用服务器配置。请先到“服务器”页面添加并连接服务器；本机测试先运行 npm run dev:server。".to_string();
    }

    if let Some(server_id) = inner.active_server_id.as_deref() {
        if let Some(server) = inner.servers.get(server_id) {
            return format!(
                "服务器「{}」当前没有可用连接。请确认服务端正在运行（本机测试运行 npm run dev:server），然后到“服务器”页面重新连接 {}:{}。",
                server.name, server.host, server.port
            );
        }
    }

    let connected_server = inner
        .servers
        .values()
        .find(|server| server.status == "connected")
        .or_else(|| inner.servers.values().next());

    if let Some(server) = connected_server {
        return format!(
            "服务器未连接，Tunnel 无法启动。请先确认服务端进程已启动，再到“服务器”页面连接「{}」({}:{})。本机测试可运行 npm run dev:server。",
            server.name, server.host, server.port
        );
    }

    "服务器未连接，Tunnel 无法启动。请先启动服务端并在“服务器”页面连接成功。".to_string()
}

fn explain_server_control_error(error: &str) -> String {
    format!(
        "服务端不可用或控制连接已断开：{error}。请检查服务端是否正在运行；本机测试请先运行 npm run dev:server，然后到“服务器”页面重新连接。如果仍失败，请确认 Token 与 GATE_AUTH_TOKEN 一致。"
    )
}

fn explain_local_runtime_error(tunnel: &TunnelRecord, error: &str) -> String {
    let lower = error.to_ascii_lowercase();
    if lower.contains("10048")
        || lower.contains("address already in use")
        || lower.contains("only one usage")
        || lower.contains("通常每个套接字地址")
    {
        return format!(
            "端口 {} 已被占用，无法作为测试访问入口。请停止占用该端口的程序，或把 Tunnel 的公网端口改成其他端口。",
            tunnel.remote_port
        );
    }

    if error.contains("本地服务不可访问") || error.contains("本地服务连接超时") {
        return format!(
            "本地服务 {}:{} 不可访问，Tunnel 已停止。请先启动你的本地应用，再重新启动 Tunnel。详细信息：{}",
            tunnel.local_host, tunnel.local_port, error
        );
    }

    if error.contains("无法解析本地服务地址") {
        return format!(
            "本地服务地址配置有误：{}:{}。请检查 Host 和端口是否填写正确。详细信息：{}",
            tunnel.local_host, tunnel.local_port, error
        );
    }

    format!(
        "本地 Runtime 启动失败。请检查本地服务 {}:{}、公网端口 {} 和协议 {}。详细信息：{}",
        tunnel.local_host, tunnel.local_port, tunnel.remote_port, tunnel.protocol, error
    )
}

fn stop_running_tunnels(inner: &mut RuntimeInner) {
    let now = Utc::now().timestamp_millis();
    for tunnel in inner.tunnels.values_mut() {
        if tunnel.status == "running" {
            tunnel.status = "stopped".to_string();
            tunnel.upload_speed_bps = 0.0;
            tunnel.download_speed_bps = 0.0;
            tunnel.connections = 0;
            tunnel.started_at = None;
            tunnel.updated_at = now;
        }
    }
}

fn default_config() -> BTreeMap<String, String> {
    let mut config = BTreeMap::new();
    config.insert("runtime.mode".to_string(), "production".to_string());
    config.insert("authentication.required".to_string(), "true".to_string());
    config.insert("heartbeat.interval_ms".to_string(), "15000".to_string());
    config.insert("network.transport".to_string(), "tcp".to_string());
    config
}

fn default_reconnect_delays() -> Vec<Duration> {
    if let Ok(value) = env::var("GATE_RECONNECT_DELAYS_MS") {
        let delays = value
            .split(',')
            .filter_map(|item| item.trim().parse::<u64>().ok())
            .filter(|millis| *millis > 0)
            .map(Duration::from_millis)
            .collect::<Vec<_>>();
        if !delays.is_empty() {
            return delays;
        }
    }

    RECONNECT_DELAYS_SECS
        .iter()
        .copied()
        .map(Duration::from_secs)
        .collect()
}

fn relay_workers_per_tunnel() -> usize {
    env::var("GATE_RELAY_WORKERS_PER_TUNNEL")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .map(|value| value.clamp(1, 4096))
        .unwrap_or(RELAY_WORKERS_PER_TUNNEL)
}

fn reconnect_delay(attempt: usize) -> Duration {
    let index = attempt.min(RECONNECT_DELAYS_SECS.len().saturating_sub(1));
    Duration::from_secs(RECONNECT_DELAYS_SECS[index])
}

fn runtime_client_id(active_server_id: Option<&str>) -> String {
    active_server_id
        .map(|server_id| format!("gate-client:{server_id}"))
        .unwrap_or_else(|| "gate-client:standalone".to_string())
}

fn runtime_store_path() -> PathBuf {
    if let Some(appdata) = env::var_os("APPDATA") {
        return PathBuf::from(appdata)
            .join("Gate")
            .join("client-runtime.json");
    }

    if let Some(xdg_data_home) = env::var_os("XDG_DATA_HOME") {
        return PathBuf::from(xdg_data_home)
            .join("Gate")
            .join("client-runtime.json");
    }

    if let Some(home) = env::var_os("HOME") {
        return PathBuf::from(home)
            .join(".local")
            .join("share")
            .join("Gate")
            .join("client-runtime.json");
    }

    PathBuf::from("gate-client-runtime.json")
}

fn domain_store_path() -> PathBuf {
    if let Some(value) = env::var_os("GATE_DOMAIN_DB") {
        return PathBuf::from(value);
    }

    runtime_data_dir().join("domains.sqlite3")
}

fn runtime_data_dir() -> PathBuf {
    runtime_store_path()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(".gate"))
}

fn certificate_store_root() -> PathBuf {
    if let Some(value) = env::var_os("GATE_CERTIFICATE_STORE") {
        return PathBuf::from(value);
    }

    if let Some(value) = env::var_os("GATE_CERT_DIR") {
        return PathBuf::from(value);
    }

    runtime_data_dir().join("certificates")
}

fn load_stored_runtime(path: &Path) -> StoredRuntime {
    let Ok(bytes) = fs::read(path) else {
        return StoredRuntime::default();
    };

    serde_json::from_slice(&bytes).unwrap_or_default()
}

fn persist_runtime(path: &Path, inner: &RuntimeInner) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }

    let bytes = serde_json::to_vec_pretty(&inner.snapshot()).map_err(|error| error.to_string())?;
    fs::write(path, bytes).map_err(|error| error.to_string())
}

fn hydrate_domains_from_repository(repository: &SqliteDomainRepository, inner: &mut RuntimeInner) {
    let Ok(domains) = repository.list() else {
        return;
    };

    let restored = domains
        .iter()
        .filter_map(|domain| runtime_domain_from_managed(domain, &inner.tunnels))
        .collect::<BTreeMap<_, _>>();

    if !restored.is_empty() {
        inner.domains = restored;
    }
}

fn sync_runtime_domains(
    repository: &Option<SqliteDomainRepository>,
    domains: &BTreeMap<String, RuntimeDomainRecord>,
) -> Result<(), String> {
    if let Some(repository) = repository {
        sync_domain_repository(repository, domains)?;
    }
    Ok(())
}

fn sync_domain_repository(
    repository: &SqliteDomainRepository,
    domains: &BTreeMap<String, RuntimeDomainRecord>,
) -> Result<(), String> {
    let desired_ids = domains
        .values()
        .map(|record| domain_id_for_host(&record.domain))
        .collect::<BTreeSet<_>>();

    for existing in repository.list().map_err(|error| error.to_string())? {
        if !desired_ids.contains(existing.id().as_str()) {
            repository
                .delete(existing.id())
                .map_err(|error| error.to_string())?;
        }
    }

    for record in domains.values() {
        let domain = managed_domain_from_runtime(record)?;
        let exists = repository
            .find_by_id(domain.id())
            .map_err(|error| error.to_string())?
            .is_some();

        if exists {
            repository
                .update(domain)
                .map_err(|error| error.to_string())?;
        } else {
            repository
                .create(domain)
                .map_err(|error| error.to_string())?;
        }
    }

    Ok(())
}

fn managed_domain_from_runtime(record: &RuntimeDomainRecord) -> Result<ManagedDomain, String> {
    let id = ManagedDomainId::new(domain_id_for_host(&record.domain))
        .map_err(|error| error.to_string())?;
    let host = ManagedHost::new(&record.domain).map_err(|error| error.to_string())?;
    let tunnel_id = ManagedTunnelId::new(&record.tunnel_id).map_err(|error| error.to_string())?;

    ManagedDomain::builder(id, host)
        .tunnel_id(tunnel_id)
        .record_type(ManagedRecordType::A)
        .status(ManagedDomainStatus::Active)
        .build()
        .map_err(|error| error.to_string())
}

fn runtime_domain_from_managed(
    domain: &ManagedDomain,
    tunnels: &BTreeMap<String, TunnelRecord>,
) -> Option<(String, RuntimeDomainRecord)> {
    let tunnel_id = domain.tunnel_id()?.as_str().to_string();
    let tunnel = tunnels.get(&tunnel_id);
    let host = domain.host().as_str().to_string();
    let now = Utc::now().timestamp_millis();

    Some((
        host.clone(),
        RuntimeDomainRecord {
            domain: host,
            tunnel_id,
            protocol: tunnel
                .map(|tunnel| tunnel.protocol.clone())
                .unwrap_or_else(|| "https".to_string()),
            status: tunnel
                .map(|tunnel| tunnel.status.clone())
                .unwrap_or_else(|| managed_domain_status_label(domain.status()).to_string()),
            created_at: now,
            updated_at: now,
        },
    ))
}

fn managed_domain_status_label(status: &ManagedDomainStatus) -> &'static str {
    match status {
        ManagedDomainStatus::Active => "active",
        ManagedDomainStatus::Pending => "pending",
        ManagedDomainStatus::Disabled => "disabled",
        ManagedDomainStatus::Deleted => "deleted",
    }
}

fn domain_id_for_host(host: &str) -> String {
    format!("domain:{}", sanitize_domain_key(host))
}

fn sanitize_domain_key(host: &str) -> String {
    let sanitized = host
        .chars()
        .map(|value| match value {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_' => value,
            _ => '_',
        })
        .collect::<String>();

    if sanitized.is_empty() {
        "unknown".to_string()
    } else {
        sanitized
    }
}

fn parse_endpoint(server_addr: &str) -> Result<TransportEndpoint, String> {
    let (host, port) = parse_server_addr(server_addr)?;
    Ok(TransportEndpoint::Tcp { host, port })
}

fn parse_server_addr(server_addr: &str) -> Result<(String, u16), String> {
    let (host, port) = server_addr
        .rsplit_once(':')
        .ok_or_else(|| "server address must be host:port".to_string())?;
    let port = port
        .parse::<u16>()
        .map_err(|_| "server port must be a valid u16".to_string())?;
    Ok((host.to_string(), port))
}

fn normalize_protocol(protocol: &str) -> Result<String, String> {
    let protocol = protocol.trim().to_ascii_lowercase();
    match protocol.as_str() {
        "tcp" | "http" | "https" => Ok(protocol),
        _ => Err("protocol must be tcp, http, or https".to_string()),
    }
}

fn normalize_http_path(path: &str) -> String {
    let path = path.trim();
    if path.is_empty() {
        return "/".to_string();
    }
    if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    }
}

fn normalize_host(host: &str) -> Result<String, String> {
    let host = host.trim();
    if host.is_empty() {
        return Err("server host is required".to_string());
    }
    Ok(host.to_string())
}

fn normalize_token(token: &str) -> Result<String, String> {
    let token = token.trim();
    if token.is_empty() {
        return Err("server token is required".to_string());
    }
    Ok(token.to_string())
}

fn normalize_port(port: u16) -> Result<u16, String> {
    if port == 0 {
        return Err("server port must be between 1 and 65535".to_string());
    }
    Ok(port)
}

fn normalize_server_name(name: &str, host: &str, port: u16) -> String {
    let name = name.trim();
    if name.is_empty() {
        format!("{host}:{port}")
    } else {
        name.to_string()
    }
}

fn normalize_server_kind(kind: Option<String>) -> String {
    let kind = kind
        .unwrap_or_else(|| "personal".to_string())
        .trim()
        .to_ascii_lowercase();
    match kind.as_str() {
        "personal" | "cloud" | "nas" | "company" | "docker" | "kubernetes" => kind,
        _ => "personal".to_string(),
    }
}

fn normalize_tags(tags: Vec<String>) -> Vec<String> {
    let mut normalized = Vec::new();
    for tag in tags {
        let tag = tag.trim();
        if !tag.is_empty() && !normalized.iter().any(|value: &String| value == tag) {
            normalized.push(tag.to_string());
        }
    }
    normalized
}

fn server_json(server: &ServerRecord) -> Value {
    json!({
        "id": server.id,
        "name": server.name,
        "kind": server.kind,
        "host": server.host,
        "port": server.port,
        "token": server.token,
        "region": server.region,
        "remark": server.remark,
        "tags": server.tags,
        "heartbeatInterval": server.heartbeat_interval,
        "reconnectInterval": server.reconnect_interval,
        "autoConnect": server.auto_connect,
        "status": server.status,
        "lastError": server.last_error,
        "lastRttMs": server.last_rtt_ms,
        "sessionId": server.session_id,
        "lastCheckedAt": server.last_checked_at,
        "lastConnectedAt": server.last_connected_at,
        "createdAt": server.created_at,
        "updatedAt": server.updated_at
    })
}

async fn establish_connection(
    server_addr: &str,
    token: &str,
) -> Result<(Arc<TcpTransport>, String), String> {
    establish_connection_with_session(server_addr, token, None, None).await
}

async fn establish_connection_with_session(
    server_addr: &str,
    token: &str,
    previous_session_id: Option<String>,
    client_id: Option<String>,
) -> Result<(Arc<TcpTransport>, String), String> {
    let endpoint = parse_endpoint(server_addr)?;
    let transport = Arc::new(TcpTransport::new());
    transport
        .connect(endpoint)
        .await
        .map_err(|error| error.to_string())?;

    let mut auth = json!({ "token": token });
    if let Some(previous_session_id) = previous_session_id.filter(|value| !value.trim().is_empty())
    {
        auth["sessionId"] = json!(previous_session_id);
    }
    if let Some(client_id) = client_id.filter(|value| !value.trim().is_empty()) {
        auth["clientId"] = json!(client_id);
    }

    let response = send_request(&transport, Command::AuthLogin, auth).await?;
    ensure_ok(&response)?;

    let session_id = response
        .pointer("/data/sessionId")
        .and_then(Value::as_str)
        .unwrap_or("local-session")
        .to_string();

    Ok((transport, session_id))
}

async fn send_request(
    transport: &Arc<TcpTransport>,
    command: Command,
    body: Value,
) -> Result<Value, String> {
    let request = Message::request(command, Body::Json(body), Metadata::default());
    transport
        .send(request)
        .await
        .map_err(|error| error.to_string())?;
    let response = transport
        .receive()
        .await
        .map_err(|error| error.to_string())?
        .ok_or_else(|| "server closed the connection".to_string())?;
    match response.body {
        Body::Json(value) => Ok(value),
        Body::Empty => Ok(Value::Null),
        _ => Err("server response was not JSON".to_string()),
    }
}

async fn read_protocol_message(
    stream: &mut TcpStream,
    protocol: &ProtocolManager,
) -> Result<Option<Message>, String> {
    let mut length = [0_u8; 4];
    match stream.read_exact(&mut length).await {
        Ok(_) => {}
        Err(error) if error.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
        Err(error) => return Err(format!("读取协议帧失败：{error}")),
    }

    let length = u32::from_be_bytes(length) as usize;
    let mut payload = vec![0_u8; length];
    stream
        .read_exact(&mut payload)
        .await
        .map_err(|error| format!("读取协议载荷失败：{error}"))?;
    protocol
        .decode(&payload)
        .map(Some)
        .map_err(|error| format!("解析协议消息失败：{error}"))
}

async fn write_protocol_message(
    stream: &mut TcpStream,
    protocol: &ProtocolManager,
    message: &Message,
) -> Result<(), String> {
    let payload = protocol
        .encode(message)
        .map_err(|error| format!("编码协议消息失败：{error}"))?;
    let frame = Frame::new(payload).map_err(|error| format!("创建协议帧失败：{error}"))?;
    let bytes = FrameEncoder::encode(&frame);
    stream
        .write_all(&bytes)
        .await
        .map_err(|error| format!("写入协议帧失败：{error}"))?;
    stream
        .flush()
        .await
        .map_err(|error| format!("刷新协议帧失败：{error}"))
}

fn ensure_ok(response: &Value) -> Result<(), String> {
    if response.get("ok").and_then(Value::as_bool) == Some(true) {
        return Ok(());
    }

    if let Some(message) = response.pointer("/error/message").and_then(Value::as_str) {
        return Err(message.to_string());
    }

    Err(response.to_string())
}

fn tunnel_control_payload(tunnel: &TunnelRecord) -> Value {
    json!({
        "id": &tunnel.id,
        "tunnelId": &tunnel.id,
        "name": &tunnel.name,
        "protocol": &tunnel.protocol,
        "remotePort": tunnel.remote_port,
        "localHost": &tunnel.local_host,
        "localPort": tunnel.local_port,
        "host": &tunnel.host,
        "path": &tunnel.path,
        "metadata": {
            "createdAt": tunnel.created_at,
            "updatedAt": tunnel.updated_at
        }
    })
}

fn statistics_json(inner: &RuntimeInner) -> Value {
    let now = Utc::now().timestamp_millis();
    let system = system_snapshot();
    let traffic = runtime_traffic(inner);
    let tls = runtime_tls(inner);
    let uptime = inner.started_at.elapsed().as_secs();
    let tunnel_count = inner.tunnels.len() as u64;
    let running_tunnel = inner
        .tunnels
        .values()
        .filter(|tunnel| tunnel.status == "running")
        .count() as u64;
    let current_connection = if inner.connected { 1 } else { 0 };
    let upload_speed_bps = inner
        .tunnels
        .values()
        .map(|tunnel| tunnel.upload_speed_bps)
        .sum::<f64>();
    let download_speed_bps = inner
        .tunnels
        .values()
        .map(|tunnel| tunnel.download_speed_bps)
        .sum::<f64>();
    let current_tunnel_connections = inner
        .tunnels
        .values()
        .map(|tunnel| tunnel.connections)
        .sum::<u64>();
    let http_requests_total = inner
        .tunnels
        .values()
        .filter(|tunnel| tunnel.protocol == "http" || tunnel.protocol == "https")
        .map(|tunnel| tunnel.request_count)
        .sum::<u64>();
    let http_active_requests = inner
        .tunnels
        .values()
        .filter(|tunnel| tunnel.protocol == "http" || tunnel.protocol == "https")
        .map(|tunnel| tunnel.connections)
        .sum::<u64>();
    let http_latency_total = inner
        .tunnels
        .values()
        .filter(|tunnel| tunnel.protocol == "http" || tunnel.protocol == "https")
        .map(|tunnel| tunnel.total_latency_ms)
        .sum::<u64>();
    let http_bandwidth_bytes = inner
        .tunnels
        .values()
        .filter(|tunnel| tunnel.protocol == "http" || tunnel.protocol == "https")
        .map(|tunnel| {
            tunnel
                .upload_bytes
                .saturating_add(tunnel.download_bytes)
                .saturating_add(
                    tunnel
                        .recent_requests
                        .iter()
                        .map(|request| request.traffic_bytes)
                        .sum::<u64>(),
                )
        })
        .sum::<u64>();
    let mut http_status_codes = BTreeMap::<String, u64>::new();
    for request in inner
        .tunnels
        .values()
        .filter(|tunnel| tunnel.protocol == "http" || tunnel.protocol == "https")
        .flat_map(|tunnel| tunnel.recent_requests.iter())
    {
        *http_status_codes
            .entry(request.status.to_string())
            .or_insert(0) += 1;
    }

    json!({
        "collectedAt": now,
        "tunnel": {
            "tunnelCount": tunnel_count,
            "runningTunnel": running_tunnel,
            "stoppedTunnel": tunnel_count.saturating_sub(running_tunnel),
            "upload": upload_speed_bps,
            "download": download_speed_bps,
            "peakSpeedBps": upload_speed_bps.max(download_speed_bps),
            "averageSpeedBps": if running_tunnel == 0 { 0.0 } else { (upload_speed_bps + download_speed_bps) / running_tunnel as f64 },
            "runningTimeSeconds": inner.tunnels.values().map(|tunnel| tunnel.uptime_seconds).sum::<u64>(),
            "todayTraffic": traffic.today_bytes,
            "totalTraffic": traffic.total_bytes
        },
        "traffic": {
            "uploadBytes": traffic.upload_bytes,
            "downloadBytes": traffic.download_bytes,
            "uploadSpeedBps": upload_speed_bps,
            "downloadSpeedBps": download_speed_bps,
            "peakSpeedBps": upload_speed_bps.max(download_speed_bps),
            "averageSpeedBps": if running_tunnel == 0 { 0.0 } else { (upload_speed_bps + download_speed_bps) / running_tunnel as f64 },
            "todayTrafficBytes": traffic.today_bytes,
            "totalTrafficBytes": traffic.total_bytes
        },
        "connection": {
            "currentConnection": current_connection + current_tunnel_connections,
            "totalConnection": inner.counters.connection_total,
            "success": inner.counters.auth_success,
            "failure": inner.counters.auth_failure,
            "reconnect": inner.counters.reconnect_total,
            "disconnect": inner.counters.disconnect_total,
            "recoverySuccess": inner.counters.recovery_success,
            "recoveryFailure": inner.counters.recovery_failure,
            "recoveryTimeMs": inner.counters.last_recovery_time_ms.unwrap_or_default(),
            "averageRecoveryTimeMs": if inner.counters.recovery_success == 0 {
                0.0
            } else {
                inner.counters.total_recovery_time_ms as f64 / inner.counters.recovery_success as f64
            },
            "connectionDurationMs": if inner.connected { inner.started_at.elapsed().as_millis() as u64 } else { 0 },
            "averageRttMs": inner.counters.average_rtt_ms
        },
        "runtime": {
            "runningTask": running_tunnel,
            "workerCount": if inner.connected { 1 } else { 0 },
            "schedulerQueue": 0,
            "bufferUsage": 0,
            "sessionCount": inner.session_id.as_ref().map(|_| 1).unwrap_or(0),
            "runtimeUptimeSeconds": uptime
        },
        "http": {
            "requestsTotal": http_requests_total,
            "activeRequests": http_active_requests,
            "statusCodes": http_status_codes,
            "latency": {
                "totalMs": http_latency_total,
                "averageMs": if http_requests_total == 0 { 0.0 } else { http_latency_total as f64 / http_requests_total as f64 }
            },
            "bandwidth": {
                "bytes": http_bandwidth_bytes
            }
        },
        "tls": {
            "sessionCount": tls.session_count,
            "handshakeCount": tls.handshake_count,
            "errorCount": tls.error_count,
            "trafficBytes": tls.traffic_bytes
        },
        "system": {
            "cpuUsage": system.cpu_usage,
            "memoryUsage": system.memory_usage,
            "diskUsage": system.disk_usage,
            "threadCount": if inner.connected { 1 } else { 0 },
            "processUptimeSeconds": uptime,
            "openFile": 0
        },
        "client": {
            "onlineTimeSeconds": if inner.connected { uptime } else { 0 },
            "openProject": 0,
            "currentWorkspace": "",
            "uiFps": 0,
            "memoryBytes": system.memory_bytes
        }
    })
}

fn dashboard_json(inner: &RuntimeInner) -> Value {
    let now = Utc::now().timestamp_millis();
    let statistics = statistics_json(inner);
    let health = health_json(inner);
    let traffic = runtime_traffic(inner);
    let tunnel_count = inner.tunnels.len() as u64;
    let running_tunnel = inner
        .tunnels
        .values()
        .filter(|tunnel| tunnel.status == "running")
        .count() as u64;
    let upload_speed_bps = inner
        .tunnels
        .values()
        .map(|tunnel| tunnel.upload_speed_bps)
        .sum::<f64>();
    let download_speed_bps = inner
        .tunnels
        .values()
        .map(|tunnel| tunnel.download_speed_bps)
        .sum::<f64>();
    let has_realtime_speed = upload_speed_bps > 0.0 || download_speed_bps > 0.0;
    let has_connection_data =
        inner.connected || inner.counters.connection_total > 0 || inner.counters.auth_failure > 0;

    json!({
        "overview": {
            "tunnelCount": tunnel_count,
            "runningTunnel": running_tunnel,
            "currentConnection": if inner.connected { 1 } else { 0 },
            "todayTraffic": traffic.today_bytes,
            "totalTraffic": traffic.total_bytes,
            "averageRttMs": inner.counters.average_rtt_ms,
            "runtimeUptimeSeconds": inner.started_at.elapsed().as_secs(),
            "healthScore": health.get("score").and_then(Value::as_u64).unwrap_or(0)
        },
        "statistics": statistics,
        "realtimeSpeed": if has_realtime_speed {
            json!([{
                "timestamp": now,
                "uploadBps": upload_speed_bps,
                "downloadBps": download_speed_bps
            }])
        } else {
            json!([])
        },
        "connectionTrend": if has_connection_data {
            json!([{
                "timestamp": now,
                "current": if inner.connected { 1 } else { 0 },
                "success": inner.counters.auth_success,
                "failure": inner.counters.auth_failure,
                "reconnect": inner.counters.reconnect_total
            }])
        } else {
            json!([])
        },
        "trafficTrend": if has_realtime_speed {
            json!([{
                "timestamp": now,
                "uploadBytes": traffic.upload_bytes,
                "downloadBytes": traffic.download_bytes
            }])
        } else {
            json!([])
        },
        "tunnelStatus": if tunnel_count == 0 {
            json!([])
        } else {
            json!([
                { "label": "running", "count": running_tunnel },
                { "label": "stopped", "count": tunnel_count.saturating_sub(running_tunnel) }
            ])
        },
        "serverStatus": if has_connection_data {
            json!([
                { "label": "online", "count": if inner.connected { 1 } else { 0 } },
                { "label": "warning", "count": 0 },
                { "label": "offline", "count": if inner.connected { 0 } else { 1 } }
            ])
        } else {
            json!([])
        },
        "systemHealth": health,
        "tunnels": inner.tunnels.values().map(|tunnel| json!({
            "id": &tunnel.id,
            "name": &tunnel.name,
            "protocol": &tunnel.protocol,
            "status": &tunnel.status,
            "localHost": &tunnel.local_host,
            "localPort": tunnel.local_port,
            "remotePort": tunnel.remote_port,
            "host": &tunnel.host,
            "path": &tunnel.path,
            "uploadSpeedBps": tunnel.upload_speed_bps,
            "downloadSpeedBps": tunnel.download_speed_bps,
            "connections": tunnel.connections,
            "uptimeSeconds": tunnel.uptime_seconds,
            "requestCount": tunnel.request_count,
            "successRate": if tunnel.request_count == 0 { 0.0 } else { tunnel.success_count as f64 / tunnel.request_count as f64 },
            "averageResponseTimeMs": if tunnel.request_count == 0 { 0.0 } else { tunnel.total_latency_ms as f64 / tunnel.request_count as f64 },
            "trafficBytes": tunnel.upload_bytes.saturating_add(tunnel.download_bytes).saturating_add(tunnel.recent_requests.iter().map(|request| request.traffic_bytes).sum::<u64>()),
            "tls": if tunnel.protocol == "https" {
                json!({
                    "sessionCount": tunnel.tls_session_count,
                    "handshakeCount": tunnel.tls_handshake_count,
                    "errorCount": tunnel.tls_error_count,
                    "tlsVersion": tunnel.tls_version.as_deref().unwrap_or("TLS"),
                    "certificateStatus": tunnel.certificate_status.as_deref().unwrap_or("missing"),
                    "certificateExpireDays": tunnel.certificate_expire_days.unwrap_or(0),
                    "certificateIssuer": tunnel.certificate_issuer.as_deref().unwrap_or("")
                })
            } else {
                Value::Null
            },
            "recentLogs": inner.logs.iter().rev()
                .filter(|log| log.tunnel_id.as_deref() == Some(tunnel.id.as_str()))
                .take(8)
                .map(|log| json!({
                    "level": &log.level,
                    "source": &log.source,
                    "message": &log.message,
                    "timestamp": log.timestamp
                }))
                .collect::<Vec<_>>(),
            "recentRequests": &tunnel.recent_requests
        })).collect::<Vec<_>>(),
        "recentActivity": inner.logs.iter().rev().take(8).map(|log| json!({
            "id": format!("{}-{}", log.source, log.timestamp),
            "title": &log.message,
            "category": &log.source,
            "timestamp": log.timestamp
        })).collect::<Vec<_>>(),
        "generatedAt": now
    })
}

fn health_json(inner: &RuntimeInner) -> Value {
    let now = Utc::now().timestamp_millis();
    let system = system_snapshot();
    let status = if inner.connected {
        "healthy"
    } else {
        "offline"
    };
    let system_status = if system.cpu_usage >= 90.0 || system.memory_usage >= 90.0 {
        "warning"
    } else {
        "healthy"
    };
    let score = if inner.connected {
        let pressure = system.cpu_usage.max(system.memory_usage).round() as u64;
        100_u64.saturating_sub(pressure.saturating_sub(70))
    } else {
        0
    };
    json!({
        "overall": status,
        "score": score,
        "signals": [
            {
                "target": "connection",
                "status": status,
                "message": if inner.connected { "TCP connection is authenticated" } else { "TCP connection is offline" },
                "score": if inner.connected { 100 } else { 0 },
                "timestamp": now
            },
            {
                "target": "heartbeat",
                "status": if inner.counters.heartbeat_pong > 0 { "healthy" } else { status },
                "message": if inner.counters.heartbeat_pong > 0 { "Heartbeat completed at least once" } else { "Heartbeat has not completed yet" },
                "score": if inner.counters.heartbeat_pong > 0 { 100 } else { 0 },
                "timestamp": now
            },
            {
                "target": "runtime",
                "status": status,
                "message": "Client runtime state is available",
                "score": if inner.connected { 100 } else { 0 },
                "timestamp": now
            },
            {
                "target": "system",
                "status": system_status,
                "message": format!("CPU {:.0}% · Memory {:.0}%", system.cpu_usage, system.memory_usage),
                "score": 100_u64.saturating_sub(system.cpu_usage.max(system.memory_usage).round() as u64),
                "timestamp": now
            }
        ],
        "updatedAt": now
    })
}

#[derive(Debug, Clone, Copy)]
struct RuntimeTraffic {
    upload_bytes: u64,
    download_bytes: u64,
    today_bytes: u64,
    total_bytes: u64,
}

#[derive(Debug, Clone, Copy)]
struct RuntimeTls {
    session_count: u64,
    handshake_count: u64,
    error_count: u64,
    traffic_bytes: u64,
}

fn runtime_traffic(inner: &RuntimeInner) -> RuntimeTraffic {
    let upload_bytes = inner
        .tunnels
        .values()
        .map(|tunnel| tunnel.upload_bytes)
        .sum::<u64>();
    let download_bytes = inner
        .tunnels
        .values()
        .map(|tunnel| {
            tunnel.download_bytes.saturating_add(
                tunnel
                    .recent_requests
                    .iter()
                    .map(|request| request.traffic_bytes)
                    .sum::<u64>(),
            )
        })
        .sum::<u64>();
    let total_bytes = upload_bytes.saturating_add(download_bytes);

    RuntimeTraffic {
        upload_bytes,
        download_bytes,
        today_bytes: total_bytes,
        total_bytes,
    }
}

fn runtime_tls(inner: &RuntimeInner) -> RuntimeTls {
    RuntimeTls {
        session_count: inner
            .tunnels
            .values()
            .map(|tunnel| tunnel.tls_session_count)
            .sum::<u64>()
            .saturating_add(inner.counters.tls_session_total),
        handshake_count: inner
            .tunnels
            .values()
            .map(|tunnel| tunnel.tls_handshake_count)
            .sum::<u64>()
            .saturating_add(inner.counters.tls_handshake_total),
        error_count: inner
            .tunnels
            .values()
            .map(|tunnel| tunnel.tls_error_count)
            .sum(),
        traffic_bytes: inner
            .tunnels
            .values()
            .filter(|tunnel| tunnel.protocol == "https")
            .map(|tunnel| {
                tunnel
                    .upload_bytes
                    .saturating_add(tunnel.download_bytes)
                    .saturating_add(
                        tunnel
                            .recent_requests
                            .iter()
                            .map(|request| request.traffic_bytes)
                            .sum::<u64>(),
                    )
            })
            .sum(),
    }
}

fn system_snapshot() -> SystemSnapshot {
    let mut system = System::new_all();
    system.refresh_all();

    let total_memory = system.total_memory();
    let used_memory = system.used_memory();
    let memory_usage = if total_memory == 0 {
        0.0
    } else {
        (used_memory as f32 / total_memory as f32) * 100.0
    };

    let disks = Disks::new_with_refreshed_list();
    let total_disk = disks.iter().map(|disk| disk.total_space()).sum::<u64>();
    let available_disk = disks.iter().map(|disk| disk.available_space()).sum::<u64>();
    let disk_usage = if total_disk == 0 {
        0.0
    } else {
        ((total_disk.saturating_sub(available_disk)) as f32 / total_disk as f32) * 100.0
    };

    SystemSnapshot {
        cpu_usage: system.global_cpu_info().cpu_usage(),
        memory_usage,
        disk_usage,
        memory_bytes: used_memory,
    }
}

fn startup_diagnostics_json(path: &Path, inner: &RuntimeInner) -> Value {
    let data_dir = path
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from("."));
    let domain_database = domain_store_path();
    let certificate_dir = certificate_store_root();
    let runtime_store_ready = path.exists() || can_write_dir(&data_dir);
    let domain_database_ready = domain_database.exists() || can_write_dir(&data_dir);
    let mut checks = Vec::new();

    checks.push(startup_check(
        "database",
        runtime_store_ready,
        if path.exists() {
            format!("Runtime store is available: {}", path.display())
        } else {
            format!("Runtime store will be created at {}", path.display())
        },
        "Ensure the parent directory is writable before creating tunnels.",
    ));
    checks.push(startup_check(
        "domainDatabase",
        domain_database_ready,
        if domain_database.exists() {
            format!(
                "Domain SQLite store is available: {}",
                domain_database.display()
            )
        } else {
            format!(
                "Domain SQLite store will be created at {}",
                domain_database.display()
            )
        },
        "Grant write permission to the Gate data directory or set GATE_DOMAIN_DB.",
    ));
    checks.push(startup_check(
        "config",
        !inner.config.is_empty(),
        format!("{} runtime config entries loaded", inner.config.len()),
        "Open Settings and restore defaults if required keys are missing.",
    ));
    checks.push(startup_check(
        "server",
        !inner.servers.is_empty(),
        if inner.servers.is_empty() {
            "No server configuration has been created yet".to_string()
        } else {
            format!("{} server configurations restored", inner.servers.len())
        },
        "Add and connect a server before creating tunnels.",
    ));
    checks.push(startup_check(
        "tls",
        true,
        "TLS runtime fields are present in the store schema".to_string(),
        "Create an HTTPS tunnel with a domain to populate TLS counters.",
    ));
    checks.push(startup_check(
        "certificate",
        certificate_dir.exists(),
        if certificate_dir.exists() {
            format!("Certificate store found: {}", certificate_dir.display())
        } else {
            format!(
                "Certificate store has not been created: {}",
                certificate_dir.display()
            )
        },
        "Issue or import a certificate for HTTPS domains.",
    ));
    checks.push(startup_check(
        "port",
        true,
        format!("{} tunnel definitions restored", inner.tunnels.len()),
        "Run deployment diagnostics with a server address to verify remote ports.",
    ));
    checks.push(startup_check(
        "permission",
        can_write_dir(&data_dir),
        format!("Data directory: {}", data_dir.display()),
        "Grant write permission to the Gate data directory.",
    ));
    checks.push(startup_check(
        "directory",
        fs::create_dir_all(&data_dir).is_ok(),
        format!("Data directory checked: {}", data_dir.display()),
        "Create the data directory manually or choose a writable data path.",
    ));

    let ok = checks
        .iter()
        .all(|check| check.get("status").and_then(Value::as_str) != Some("error"));

    json!({
        "ok": ok,
        "checkedAt": Utc::now().timestamp_millis(),
        "checks": checks,
        "store": {
            "path": path,
            "domainDatabase": domain_database,
            "dataDir": data_dir,
            "certificateDir": certificate_dir
        }
    })
}

fn startup_check(id: &str, ok: bool, message: String, suggestion: &str) -> Value {
    json!({
        "id": id,
        "status": if ok { "ok" } else { "warning" },
        "message": message,
        "suggestion": suggestion
    })
}

fn can_write_dir(path: &Path) -> bool {
    if fs::create_dir_all(path).is_err() {
        return false;
    }
    let probe = path.join(".gate-write-check");
    fs::write(&probe, b"gate")
        .and_then(|_| fs::remove_file(&probe))
        .is_ok()
}

fn metric(
    name: &str,
    description: &str,
    kind: &str,
    scope: &str,
    unit: &str,
    value: f64,
    timestamp: i64,
) -> Value {
    json!({
        "name": name,
        "description": description,
        "kind": kind,
        "scope": scope,
        "unit": unit,
        "value": value,
        "labels": [],
        "timestamp": timestamp
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::{TcpListener, TcpStream};

    #[tokio::test]
    async fn local_http_tunnel_runtime_forwards_requests() -> anyhow::Result<()> {
        let target_listener = TcpListener::bind("127.0.0.1:0").await?;
        let target_addr = target_listener.local_addr()?;
        let target_task = tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = target_listener.accept().await else {
                    break;
                };
                tokio::spawn(async move {
                    let mut buffer = [0_u8; 1024];
                    let _ = stream.read(&mut buffer).await;
                    let response =
                        b"HTTP/1.1 200 OK\r\nContent-Length: 7\r\nConnection: close\r\n\r\ngate-ok";
                    let _ = stream.write_all(response).await;
                });
            }
        });

        let remote_port = unused_loopback_port().await?;
        let tunnel = test_tunnel_record("http", target_addr.port(), remote_port);
        let runtime = start_local_tunnel_runtime(&tunnel)
            .await
            .map_err(|error| anyhow::anyhow!(error))?;

        let mut client = TcpStream::connect(("127.0.0.1", remote_port)).await?;
        client
            .write_all(b"GET / HTTP/1.1\r\nHost: local.test\r\nConnection: close\r\n\r\n")
            .await?;
        let mut response = String::new();
        client.read_to_string(&mut response).await?;

        assert!(response.contains("200 OK"));
        assert!(response.contains("gate-ok"));

        runtime
            .shutdown()
            .await
            .map_err(|error| anyhow::anyhow!(error))?;
        target_task.abort();
        Ok(())
    }

    #[test]
    fn runtime_domain_repository_sync_persists_and_deletes() -> anyhow::Result<()> {
        let temp_dir = tempfile::tempdir()?;
        let db_path = temp_dir.path().join("domains.sqlite3");
        let repository = SqliteDomainRepository::open(&db_path)?;
        let mut tunnel = test_tunnel_record("https", 8443, 18443);
        tunnel.host = Some("example.com".to_string());

        let tunnel_id = tunnel.id.clone();
        let mut inner = RuntimeInner::from_stored(StoredRuntime::default());
        inner.tunnels.insert(tunnel_id.clone(), tunnel);
        inner.sync_domain_index_for_tunnel(&tunnel_id);

        sync_domain_repository(&repository, &inner.domains).map_err(anyhow::Error::msg)?;
        let reopened = SqliteDomainRepository::open(&db_path)?;
        let domains = reopened.list()?;
        assert_eq!(domains.len(), 1);
        assert_eq!(domains[0].host().as_str(), "example.com");
        assert_eq!(
            domains[0].tunnel_id().map(ManagedTunnelId::as_str),
            Some(tunnel_id.as_str())
        );

        inner.tunnels.remove(&tunnel_id);
        inner.sync_domain_index_for_tunnel(&tunnel_id);
        sync_domain_repository(&reopened, &inner.domains).map_err(anyhow::Error::msg)?;
        assert!(SqliteDomainRepository::open(&db_path)?.list()?.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn client_reconnect_restores_session_and_registers_running_tunnel() -> anyhow::Result<()>
    {
        let register_count = Arc::new(AtomicU64::new(0));
        let (server_addr, server_shutdown, server_task) =
            start_reconnect_test_server(Arc::clone(&register_count)).await?;
        let target_listener = TcpListener::bind("127.0.0.1:0").await?;
        let target_addr = target_listener.local_addr()?;
        let target_task = tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = target_listener.accept().await else {
                    break;
                };
                tokio::spawn(async move {
                    let mut buffer = [0_u8; 16];
                    let _ = stream.read(&mut buffer).await;
                });
            }
        });

        let temp_dir = tempfile::tempdir()?;
        let storage_path = temp_dir.path().join("runtime.json");
        let server_id = "server-reconnect".to_string();
        let previous_session_id = "session-old".to_string();
        let now = Utc::now().timestamp_millis();
        let mut inner = RuntimeInner::from_stored(StoredRuntime::default());
        inner.servers.insert(
            server_id.clone(),
            ServerRecord {
                id: server_id.clone(),
                name: "reconnect-test".to_string(),
                kind: "personal".to_string(),
                host: server_addr.ip().to_string(),
                port: server_addr.port(),
                token: "gate-alpha-token".to_string(),
                region: String::new(),
                remark: String::new(),
                tags: Vec::new(),
                heartbeat_interval: 30,
                reconnect_interval: 5,
                auto_connect: true,
                status: "connected".to_string(),
                last_error: None,
                last_rtt_ms: None,
                session_id: Some(previous_session_id.clone()),
                last_checked_at: Some(now),
                last_connected_at: Some(now),
                created_at: now,
                updated_at: now,
            },
        );
        inner.active_server_id = Some(server_id);
        inner.session_id = Some(previous_session_id.clone());
        inner.connected = false;

        let mut tunnel =
            test_tunnel_record("tcp", target_addr.port(), unused_loopback_port().await?);
        tunnel.status = "running".to_string();
        tunnel.started_at = Some(now);
        inner.tunnels.insert(tunnel.id.clone(), tunnel);

        let runtime = ClientRuntimeState {
            storage_path,
            domain_repository: None,
            inner: Mutex::new(inner),
            local_tunnels: Mutex::new(BTreeMap::new()),
        };

        let restored_session = runtime
            .restore_active_server_with_backoff(&[Duration::from_millis(1)])
            .await
            .map_err(anyhow::Error::msg)?;

        assert_eq!(restored_session, previous_session_id);
        assert_eq!(register_count.load(Ordering::Relaxed), 1);

        runtime
            .shutdown_all_local_tunnels()
            .await
            .map_err(anyhow::Error::msg)?;
        let _ = server_shutdown.send(true);
        server_task.abort();
        target_task.abort();
        Ok(())
    }

    async fn unused_loopback_port() -> anyhow::Result<u16> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let port = listener.local_addr()?.port();
        drop(listener);
        Ok(port)
    }

    fn test_tunnel_record(protocol: &str, local_port: u16, remote_port: u16) -> TunnelRecord {
        let now = Utc::now().timestamp_millis();
        TunnelRecord {
            id: Uuid::new_v4().to_string(),
            name: "local-test".to_string(),
            protocol: protocol.to_string(),
            status: "stopped".to_string(),
            local_host: "127.0.0.1".to_string(),
            local_port,
            remote_port,
            host: None,
            path: Some("/".to_string()),
            upload_speed_bps: 0.0,
            download_speed_bps: 0.0,
            upload_bytes: 0,
            download_bytes: 0,
            connections: 0,
            uptime_seconds: 0,
            request_count: 0,
            success_count: 0,
            total_latency_ms: 0,
            started_at: None,
            created_at: now,
            updated_at: now,
            last_sample_at: now,
            tls_session_count: 0,
            tls_handshake_count: 0,
            tls_error_count: 0,
            tls_version: None,
            certificate_status: None,
            certificate_expire_days: None,
            certificate_issuer: None,
            recent_requests: Vec::new(),
        }
    }

    async fn start_reconnect_test_server(
        register_count: Arc<AtomicU64>,
    ) -> anyhow::Result<(SocketAddr, watch::Sender<bool>, JoinHandle<()>)> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let addr = listener.local_addr()?;
        let (shutdown, mut shutdown_rx) = watch::channel(false);
        let task = tokio::spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown_rx.changed() => break,
                    accepted = listener.accept() => {
                        let Ok((stream, _)) = accepted else {
                            continue;
                        };
                        let register_count = Arc::clone(&register_count);
                        tokio::spawn(async move {
                            let _ = handle_reconnect_test_connection(stream, register_count).await;
                        });
                    }
                }
            }
        });

        Ok((addr, shutdown, task))
    }

    async fn handle_reconnect_test_connection(
        mut stream: TcpStream,
        register_count: Arc<AtomicU64>,
    ) -> anyhow::Result<()> {
        let protocol = ProtocolBuilder::new().build();
        loop {
            let Some(message) = read_protocol_message(&mut stream, &protocol)
                .await
                .map_err(anyhow::Error::msg)?
            else {
                break;
            };

            let body = match &message.body {
                Body::Json(value) => value.clone(),
                _ => Value::Null,
            };
            let response_body = match message.header.command {
                Command::AuthLogin => {
                    let session_id = body
                        .get("sessionId")
                        .and_then(Value::as_str)
                        .unwrap_or("session-new");
                    json!({ "ok": true, "data": { "sessionId": session_id } })
                }
                Command::TunnelRegister | Command::TunnelStart => {
                    register_count.fetch_add(1, Ordering::Relaxed);
                    json!({ "ok": true, "data": { "registered": true } })
                }
                Command::TunnelRelayConnect => {
                    json!({ "ok": true, "data": { "waiting": true } })
                }
                _ => json!({ "ok": true, "data": {} }),
            };
            let mut response = Message::new(
                gate_protocol::MessageType::Response,
                message.header.command.clone(),
                Body::Json(response_body),
                Metadata::default(),
            );
            response.header.request_id = message.header.request_id;
            write_protocol_message(&mut stream, &protocol, &response)
                .await
                .map_err(anyhow::Error::msg)?;

            if message.header.command == Command::TunnelRelayConnect {
                break;
            }
        }

        Ok(())
    }
}
