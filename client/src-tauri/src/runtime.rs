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
    sync::{watch, Mutex, Notify, RwLock},
    task::JoinHandle,
    time::sleep,
};
use uuid::Uuid;

use crate::discovery::{
    diagnose_start_failure, local_port_discovery, local_service_discovery, probe_local_service,
};
use crate::tunnel_performance::{
    self, recommend_tunnel_performance, resolve_tunnel_performance,
    TunnelPerformanceRecommendation, CONFIG_KEY_MODE,
};
use crate::utils::app_data_dir;

const STORE_VERSION: u32 = 1;
const MAX_LOGS: usize = 500;
const RECONNECT_DELAYS_SECS: [u64; 6] = [1, 2, 5, 10, 30, 60];
const HEARTBEAT_RECONNECT_DELAYS_SECS: [u64; 4] = [0, 1, 2, 5];
const WORKER_SUPERVISOR_INTERVAL: Duration = Duration::from_secs(1);
const DEFAULT_MINIMUM_RELAY_WORKERS: usize = 32;
const RELAY_ATTACH_CONNECT_TIMEOUT: Duration = Duration::from_secs(10);
const RELAY_ATTACH_HANDSHAKE_TIMEOUT: Duration = Duration::from_secs(10);
const RELAY_COPY_BUFFER_SIZE: usize = 64 * 1024;
const SERVER_LOG_SNAPSHOT_TIMEOUT: Duration = Duration::from_secs(3);

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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTunnelRequest {
    pub name: Option<String>,
    pub server_id: Option<String>,
    pub protocol: Option<String>,
    pub local_host: Option<String>,
    pub local_port: Option<u16>,
    pub remote_port: Option<u16>,
    pub host: Option<String>,
    pub path: Option<String>,
    pub performance_mode: Option<String>,
    pub relay_workers: Option<u32>,
    pub max_connections: Option<u64>,
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
    #[serde(default)]
    server_id: Option<String>,
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
    #[serde(default)]
    performance_mode: Option<String>,
    #[serde(default)]
    relay_workers: Option<u32>,
    #[serde(default)]
    max_connections: Option<u64>,
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
    #[serde(default)]
    discovery: Value,
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
    #[serde(default)]
    today_traffic_bytes: u64,
    #[serde(default)]
    today_traffic_date: String,
    #[serde(default)]
    total_traffic_bytes: u64,
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
    session: Arc<RwLock<RelaySessionContext>>,
}

#[derive(Clone)]
struct ServerConnectionRuntime {
    transport: Arc<TcpTransport>,
    session_id: String,
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
struct RelaySessionContext {
    session_id: String,
}

#[derive(Debug, Clone)]
struct RelayWorkerConfig {
    worker_id: usize,
    server_host: String,
    server_port: u16,
    token: String,
    session: Arc<RwLock<RelaySessionContext>>,
    tunnel_id: String,
    protocol: String,
    target_addr: SocketAddr,
}

#[derive(Debug, Clone, Copy)]
struct RelayWorkerPoolConfig {
    minimum: usize,
    maximum: usize,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct RelayWorkerSnapshot {
    worker_id: String,
    tunnel_id: String,
    state: String,
    alive: bool,
    ready: bool,
    healthy: bool,
    session_id: String,
    last_heartbeat: i64,
    last_attach: Option<i64>,
    last_serve: Option<i64>,
    last_activity: i64,
    bytes_in: u64,
    bytes_out: u64,
    failure_count: u64,
    success_count: u64,
    reconnect_count: u64,
    latency_ms: Option<u64>,
    attach_failures: u64,
    attach_failure_count: u64,
    last_attach_success: Option<i64>,
    last_successful_attach: Option<i64>,
    attach_latency_ms: Option<u64>,
    last_attach_error: Option<String>,
}

#[derive(Debug, Default)]
struct RelayWorkerRegistry {
    workers: Mutex<BTreeMap<String, RelayWorkerSnapshot>>,
}

impl RelayWorkerRegistry {
    async fn register(&self, worker_id: String, tunnel_id: String, session_id: String) {
        let now = Utc::now().timestamp_millis();
        self.workers.lock().await.insert(
            worker_id.clone(),
            RelayWorkerSnapshot {
                worker_id,
                tunnel_id,
                state: "initializing".to_string(),
                alive: true,
                ready: false,
                healthy: false,
                session_id,
                last_heartbeat: now,
                last_attach: None,
                last_serve: None,
                last_activity: now,
                bytes_in: 0,
                bytes_out: 0,
                failure_count: 0,
                success_count: 0,
                reconnect_count: 0,
                latency_ms: None,
                attach_failures: 0,
                attach_failure_count: 0,
                last_attach_success: None,
                last_successful_attach: None,
                attach_latency_ms: None,
                last_attach_error: None,
            },
        );
    }

    async fn update(&self, worker_id: &str, update: impl FnOnce(&mut RelayWorkerSnapshot)) {
        if let Some(worker) = self.workers.lock().await.get_mut(worker_id) {
            update(worker);
            worker.last_activity = Utc::now().timestamp_millis();
        }
    }

    async fn update_tunnel_session(&self, tunnel_id: &str, session_id: &str) {
        let now = Utc::now().timestamp_millis();
        for worker in self.workers.lock().await.values_mut() {
            if worker.tunnel_id == tunnel_id {
                worker.session_id = session_id.to_string();
                worker.state = "recovering".to_string();
                worker.ready = false;
                worker.healthy = false;
                worker.last_heartbeat = now;
                worker.last_activity = now;
            }
        }
    }

    async fn remove_tunnel(&self, tunnel_id: &str) {
        self.workers
            .lock()
            .await
            .retain(|_, worker| worker.tunnel_id != tunnel_id);
    }

    async fn remove(&self, worker_id: &str) {
        self.workers.lock().await.remove(worker_id);
    }

    async fn snapshot(&self) -> Value {
        let workers = self
            .workers
            .lock()
            .await
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let total = workers.len() as u64;
        let healthy = workers.iter().filter(|worker| worker.healthy).count() as u64;
        let serving = workers
            .iter()
            .filter(|worker| worker.state == "serving")
            .count() as u64;
        let recovering = workers
            .iter()
            .filter(|worker| worker.state == "recovering")
            .count() as u64;
        let dead = workers
            .iter()
            .filter(|worker| worker.state == "dead")
            .count() as u64;
        let attach_attempts = workers
            .iter()
            .map(|worker| worker.success_count + worker.attach_failure_count)
            .sum::<u64>();
        let attach_success = workers
            .iter()
            .map(|worker| worker.success_count)
            .sum::<u64>();
        let average_attach_time_ms = {
            let samples = workers
                .iter()
                .filter_map(|worker| worker.latency_ms)
                .collect::<Vec<_>>();
            if samples.is_empty() {
                0.0
            } else {
                samples.iter().sum::<u64>() as f64 / samples.len() as f64
            }
        };
        json!({
            "items": workers,
            "healthyWorkers": healthy,
            "recoveringWorkers": recovering,
            "deadWorkers": dead,
            "currentPool": total,
            "poolUsage": if total == 0 { 0.0 } else { (serving as f64 / total as f64) * 100.0 },
            "attachSuccessRate": if attach_attempts == 0 { 0.0 } else { attach_success as f64 / attach_attempts as f64 },
            "averageAttachTimeMs": average_attach_time_ms,
            "reconnectCount": workers.iter().map(|worker| worker.reconnect_count).sum::<u64>(),
            // 此处表示本地数据面正在等待 attach 完成的 Worker，不再使用硬编码值。
            "currentQueue": workers.iter().filter(|worker| matches!(worker.state.as_str(), "initializing" | "connecting" | "recovering")).count()
        })
    }

    async fn tunnel_statuses(&self) -> BTreeMap<String, &'static str> {
        let workers = self.workers.lock().await;
        let mut statuses = BTreeMap::new();
        for worker in workers.values() {
            let candidate = if worker.healthy {
                "running"
            } else if worker.alive && worker.state == "recovering" {
                "recovering"
            } else {
                "warning"
            };
            statuses
                .entry(worker.tunnel_id.clone())
                .and_modify(|current| {
                    if *current != "running" && candidate == "running" {
                        *current = candidate;
                    } else if *current == "warning" && candidate == "recovering" {
                        *current = candidate;
                    }
                })
                .or_insert(candidate);
        }
        statuses
    }
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
    async fn update_session(&self, session_id: String) {
        self.session.write().await.session_id = session_id;
    }

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
        let mut servers = stored.servers;
        deactivate_persisted_servers(&mut servers, "CLIENT_RESTART_MANUAL_START");
        let mut tunnels = stored.tunnels;
        deactivate_persisted_tunnels(&mut tunnels);

        Self {
            transport: None,
            server_addr: None,
            session_id: None,
            connected: false,
            started_at: Instant::now(),
            counters: stored.counters,
            config: stored.config,
            tunnels,
            domains: stored.domains,
            certificate_references: stored.certificate_references,
            servers,
            active_server_id: None,
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
    tunnel_start_lock: Mutex<()>,
    worker_registry: Arc<RelayWorkerRegistry>,
    server_connections: Mutex<BTreeMap<String, ServerConnectionRuntime>>,
    control_io_lock: Mutex<()>,
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
            tunnel_start_lock: Mutex::new(()),
            worker_registry: Arc::new(RelayWorkerRegistry::default()),
            server_connections: Mutex::new(BTreeMap::new()),
            control_io_lock: Mutex::new(()),
        }
    }
}

impl ClientRuntimeState {
    pub async fn backup_snapshot(&self) -> Result<Value, String> {
        self.sync_local_tunnel_metrics().await;
        let mut inner = self.inner.lock().await;
        inner.sync_tunnel_state();
        serde_json::to_value(inner.snapshot()).map_err(|error| error.to_string())
    }

    pub async fn shutdown_on_exit(&self) -> Result<(), String> {
        use std::sync::atomic::{AtomicBool, Ordering};

        static SHUTDOWN_DONE: AtomicBool = AtomicBool::new(false);
        if SHUTDOWN_DONE.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        self.shutdown_all_local_tunnels().await?;
        let transports: Vec<_> = {
            let mut connections = self.server_connections.lock().await;
            std::mem::take(&mut *connections)
                .into_values()
                .map(|connection| connection.transport)
                .collect()
        };

        let mut inner = self.inner.lock().await;
        deactivate_persisted_tunnels(&mut inner.tunnels);
        deactivate_persisted_servers(&mut inner.servers, "CLIENT_SHUTDOWN");
        inner.transport = None;
        inner.connected = false;
        inner.server_addr = None;
        inner.session_id = None;
        inner.active_server_id = None;
        inner.counters.disconnect_total = inner
            .counters
            .disconnect_total
            .saturating_add(transports.len() as u64);
        inner.log("info", "shutdown", "CLIENT_SHUTDOWN");
        persist_runtime(&self.storage_path, &inner)?;
        drop(inner);

        for transport in transports {
            let _ = transport.disconnect().await;
        }

        Ok(())
    }

    pub async fn stop_for_restore(&self) -> Result<(), String> {
        self.shutdown_all_local_tunnels().await?;
        let transport = {
            let mut inner = self.inner.lock().await;
            stop_running_tunnels(&mut inner);
            inner.connected = false;
            inner.session_id = None;
            inner.server_addr = None;
            let active_server_id = inner.active_server_id.take();
            if let Some(server_id) = active_server_id.as_deref() {
                if let Some(server) = inner.servers.get_mut(server_id) {
                    server.status = "disconnected".to_string();
                    server.session_id = None;
                    server.last_error = Some("BACKUP_RUNTIME_STOPPED".to_string());
                    server.updated_at = Utc::now().timestamp_millis();
                }
            }
            inner.counters.disconnect_total = inner.counters.disconnect_total.saturating_add(1);
            inner.log("info", "backup", "BACKUP_RUNTIME_STOPPED");
            let transport = inner.transport.take();
            persist_runtime(&self.storage_path, &inner)?;
            transport
        };

        if let Some(transport) = transport {
            let _ = transport.disconnect().await;
        }

        Ok(())
    }

    pub async fn restore_runtime_snapshot(&self, snapshot: Value) -> Result<(), String> {
        self.shutdown_all_local_tunnels().await?;
        let stored: StoredRuntime =
            serde_json::from_value(snapshot).map_err(|error| error.to_string())?;
        if stored.version > STORE_VERSION {
            return Err(format!(
                "RUNTIME_BACKUP_VERSION_UNSUPPORTED:{}>{}",
                stored.version, STORE_VERSION
            ));
        }

        let mut restored = RuntimeInner::from_stored(stored);
        mark_restored_runtime_inactive(&mut restored);
        restored.log("info", "backup", "BACKUP_RESTORED_MANUAL_RECONNECT");
        persist_runtime(&self.storage_path, &restored)?;

        let mut inner = self.inner.lock().await;
        *inner = restored;
        Ok(())
    }

    pub async fn connect(&self, server_addr: String, token: String) -> Result<String, String> {
        let (transport, session_id, discovery) = establish_connection(&server_addr, &token).await?;

        let mut inner = self.inner.lock().await;
        inner.transport = Some(transport);
        inner.server_addr = Some(server_addr);
        inner.session_id = Some(session_id.clone());
        inner.connected = true;
        inner.active_server_id = None;
        inner.counters.connection_total += 1;
        inner.counters.auth_success += 1;
        inner
            .config
            .insert("server.discovery".to_string(), discovery.to_string());
        inner.log("info", "connection", "client connected");
        inner.log("info", "authentication", "session established");
        persist_runtime(&self.storage_path, &inner)?;
        Ok(session_id)
    }

    pub async fn list_servers(&self) -> Value {
        let connected_ids: Vec<String> = self
            .server_connections
            .lock()
            .await
            .keys()
            .cloned()
            .collect();
        let connected_set: BTreeSet<String> = connected_ids.iter().cloned().collect();
        let inner = self.inner.lock().await;
        let recovery_in_progress = inner
            .servers
            .values()
            .any(|server| server.status == "recovering");
        let items = inner
            .servers
            .values()
            .map(|server| {
                let mut item = server_json(server);
                if connected_set.contains(&server.id) {
                    item["status"] = json!("connected");
                }
                item
            })
            .collect::<Vec<_>>();
        json!({
            "items": items,
            "activeServerId": inner.active_server_id,
            "connectedServerIds": connected_ids,
            "connected": !connected_set.is_empty(),
            "recoveryInProgress": recovery_in_progress
        })
    }

    pub async fn has_active_server_connections(&self) -> bool {
        !self.server_connections.lock().await.is_empty()
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
            discovery: Value::Null,
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
        self.shutdown_local_tunnels_for_server(&server_id).await?;
        let transport = {
            let transport = self
                .server_connections
                .lock()
                .await
                .remove(&server_id)
                .map(|connection| connection.transport);
            let mut inner = self.inner.lock().await;
            if !inner.servers.contains_key(&server_id) {
                return Err("server not found".to_string());
            }
            stop_running_tunnels_for_server(&mut inner, &server_id);
            inner.servers.remove(&server_id);
            if transport.is_some() {
                inner.counters.disconnect_total += 1;
                inner.log("info", "server", "server disconnected before deletion");
            }
            inner.log("info", "server", "server configuration deleted");
            self.sync_primary_from_connections_locked(&mut inner).await;
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
        if self
            .server_connections
            .lock()
            .await
            .contains_key(&server_id)
        {
            if self.verify_server_connection(&server_id).await {
                return Ok(self
                    .server_connections
                    .lock()
                    .await
                    .get(&server_id)
                    .map(|connection| connection.session_id.clone())
                    .unwrap_or_default());
            }
            self.record_control_disconnect_for(
                &server_id,
                "stale connection detected; reconnecting",
            )
            .await;
        }

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
            Ok((transport, session_id, discovery)) => {
                let now = Utc::now().timestamp_millis();
                let mut inner = self.inner.lock().await;

                if let Some(server) = inner.servers.get_mut(&server_id) {
                    server.status = "connected".to_string();
                    server.session_id = Some(session_id.clone());
                    server.last_connected_at = Some(now);
                    server.last_checked_at = Some(now);
                    server.last_error = None;
                    server.discovery = discovery.clone();
                    server.updated_at = now;
                }

                inner.transport = Some(transport.clone());
                inner.server_addr = Some(server_addr.clone());
                inner.session_id = Some(session_id.clone());
                inner.connected = true;
                inner.active_server_id = Some(server_id.clone());
                inner.counters.connection_total += 1;
                inner.counters.auth_success += 1;
                inner.config.insert(
                    format!("server.discovery.{server_id}"),
                    discovery.to_string(),
                );
                inner.log("info", "server", "server connected");
                inner.log("info", "authentication", "session established");
                persist_runtime(&self.storage_path, &inner)?;
                self.server_connections.lock().await.insert(
                    server_id,
                    ServerConnectionRuntime {
                        transport,
                        session_id: session_id.clone(),
                    },
                );
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
        self.shutdown_local_tunnels_for_server(&server_id).await?;
        let transport = self
            .server_connections
            .lock()
            .await
            .remove(&server_id)
            .map(|connection| connection.transport);
        let transport = {
            let mut inner = self.inner.lock().await;
            if !inner.servers.contains_key(&server_id) {
                return Err("server not found".to_string());
            }

            if let Some(server) = inner.servers.get_mut(&server_id) {
                server.status = "disconnected".to_string();
                server.session_id = None;
                server.last_error = None;
                server.updated_at = Utc::now().timestamp_millis();
            }

            stop_running_tunnels_for_server(&mut inner, &server_id);
            self.sync_primary_from_connections_locked(&mut inner).await;
            inner.counters.disconnect_total += 1;
            inner.log("info", "server", "server disconnected");
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
            Ok((transport, session_id, discovery)) => {
                let rtt_ms = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;
                let _ = transport.disconnect().await;
                let now = Utc::now().timestamp_millis();
                let mut inner = self.inner.lock().await;
                if let Some(server) = inner.servers.get_mut(&server_id) {
                    server.last_rtt_ms = Some(rtt_ms);
                    server.last_checked_at = Some(now);
                    server.last_error = None;
                    server.discovery = discovery;
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
                let was_connected = self
                    .server_connections
                    .lock()
                    .await
                    .contains_key(&server_id);
                if was_connected {
                    self.record_control_disconnect_for(
                        &server_id,
                        &format!("server connection test failed: {error}"),
                    )
                    .await;
                    if let Err(recovery_error) = self
                        .restore_server_with_backoff(&server_id, &default_reconnect_delays())
                        .await
                    {
                        tracing::warn!(
                            "control recovery after connection test failed: {recovery_error}"
                        );
                    }
                } else {
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
                }
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
        let server_ids: Vec<String> = self
            .server_connections
            .lock()
            .await
            .keys()
            .cloned()
            .collect();
        if server_ids.is_empty() {
            return Err("runtime backend is not connected".to_string());
        }

        let started = Instant::now();
        let mut last_rtt = 0_u64;
        for server_id in server_ids {
            last_rtt = match self
                .request_for(
                    &server_id,
                    Command::HeartbeatPing,
                    self.heartbeat_payload_for(&server_id).await,
                )
                .await
            {
                Ok(response) => {
                    ensure_ok(&response)?;
                    started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
                }
                Err(error) => {
                    self.record_control_disconnect_for(
                        &server_id,
                        &format!("heartbeat failed; starting control recovery: {error}"),
                    )
                    .await;
                    self.restore_server_with_backoff(&server_id, &heartbeat_reconnect_delays())
                        .await?;
                    let response = self
                        .request_for(
                            &server_id,
                            Command::HeartbeatPing,
                            self.heartbeat_payload_for(&server_id).await,
                        )
                        .await?;
                    ensure_ok(&response)?;
                    started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64
                }
            };
        }

        let mut inner = self.inner.lock().await;
        inner.counters.heartbeat_ping += 1;
        inner.counters.heartbeat_pong += 1;
        inner.counters.last_rtt_ms = Some(last_rtt);
        inner.counters.average_rtt_ms = if inner.counters.heartbeat_pong == 1 {
            last_rtt as f64
        } else {
            ((inner.counters.average_rtt_ms * (inner.counters.heartbeat_pong - 1) as f64)
                + last_rtt as f64)
                / inner.counters.heartbeat_pong as f64
        };
        inner.log("info", "heartbeat", "heartbeat completed");
        persist_runtime(&self.storage_path, &inner)?;
        Ok(last_rtt)
    }

    async fn heartbeat_payload_for(&self, server_id: &str) -> Value {
        let inner = self.inner.lock().await;
        let registered_tunnels = inner
            .tunnels
            .values()
            .filter(|tunnel| tunnel.server_id.as_deref() == Some(server_id))
            .map(|tunnel| tunnel.id.clone())
            .collect::<Vec<_>>();
        let running_tunnels = inner
            .tunnels
            .values()
            .filter(|tunnel| {
                tunnel.server_id.as_deref() == Some(server_id)
                    && is_running_tunnel_status(&tunnel.status)
            })
            .count();
        let active_connections = inner
            .tunnels
            .values()
            .filter(|tunnel| tunnel.server_id.as_deref() == Some(server_id))
            .map(|tunnel| tunnel.connections)
            .sum::<u64>();

        json!({
            "sentAt": Utc::now().timestamp_millis(),
            "clientId": runtime_client_id(Some(server_id)),
            "clientStatus": "online",
            "registeredTunnels": registered_tunnels,
            "runtimeStatus": {
                "sessionId": inner
                    .servers
                    .get(server_id)
                    .and_then(|server| server.session_id.clone()),
                "runningTunnels": running_tunnels,
                "activeConnections": active_connections,
                "requestTotal": inner.counters.request_total,
                "responseTotal": inner.counters.response_total,
                "reconnectTotal": inner.counters.reconnect_total
            }
        })
    }

    // 控制连接恢复只复用现有 AuthLogin/TunnelRegister，不引入新的协议命令。
    async fn record_control_disconnect_for(&self, server_id: &str, reason: &str) {
        let transport = {
            let mut inner = self.inner.lock().await;
            inner.counters.heartbeat_timeout = inner.counters.heartbeat_timeout.saturating_add(1);
            inner.counters.disconnect_total = inner.counters.disconnect_total.saturating_add(1);
            if let Some(server) = inner.servers.get_mut(server_id) {
                server.status = "recovering".to_string();
                server.last_error = Some(reason.to_string());
                server.last_checked_at = Some(Utc::now().timestamp_millis());
                server.updated_at = Utc::now().timestamp_millis();
            }
            inner.log("warn", "reconnect", reason);
            let _ = persist_runtime(&self.storage_path, &inner);
            self.server_connections
                .lock()
                .await
                .remove(server_id)
                .map(|connection| connection.transport)
        };

        {
            let mut inner = self.inner.lock().await;
            self.sync_primary_from_connections_locked(&mut inner).await;
            let _ = persist_runtime(&self.storage_path, &inner);
        }

        if let Some(transport) = transport {
            let _ = transport.disconnect().await;
        }
    }

    async fn record_control_disconnect(&self, reason: &str) {
        let active_server_id = {
            let inner = self.inner.lock().await;
            inner.active_server_id.clone()
        };
        if let Some(server_id) = active_server_id {
            self.record_control_disconnect_for(&server_id, reason).await;
        }
    }

    async fn restore_active_server_with_backoff(
        &self,
        delays: &[Duration],
    ) -> Result<String, String> {
        let server_id = {
            let inner = self.inner.lock().await;
            inner
                .active_server_id
                .clone()
                .ok_or_else(|| "NO_RECOVERABLE_ACTIVE_SERVER".to_string())?
        };
        self.restore_server_with_backoff(&server_id, delays).await
    }

    async fn restore_server_with_backoff(
        &self,
        server_id: &str,
        delays: &[Duration],
    ) -> Result<String, String> {
        let (server_id, server_addr, token, previous_session_id, running_tunnels) = {
            let mut inner = self.inner.lock().await;
            let server = inner
                .servers
                .get(server_id)
                .ok_or_else(|| "ACTIVE_SERVER_CONFIG_MISSING".to_string())?;
            let server_addr = format!("{}:{}", server.host, server.port);
            let token = server.token.clone();
            let previous_session_id = server.session_id.clone();
            let running_tunnels = inner
                .tunnels
                .values()
                .filter(|tunnel| {
                    tunnel.server_id.as_deref() == Some(server_id)
                        && matches!(tunnel.status.as_str(), "running" | "recovering")
                })
                .cloned()
                .collect::<Vec<_>>();
            if let Some(server) = inner.servers.get_mut(server_id) {
                server.status = "recovering".to_string();
                server.updated_at = Utc::now().timestamp_millis();
            }
            inner.log("info", "reconnect", "RECONNECT_CONTROL_STARTED");
            persist_runtime(&self.storage_path, &inner)?;
            (
                server_id.to_string(),
                server_addr,
                token,
                previous_session_id,
                running_tunnels,
            )
        };

        let recovery_started = Instant::now();
        let mut last_error = None;
        for (attempt, delay) in delays.iter().enumerate() {
            if attempt > 0 {
                tokio::select! {
                    _ = sleep(*delay) => {}
                }
            }

            match establish_connection_with_session(
                &server_addr,
                &token,
                previous_session_id.clone(),
                Some(runtime_client_id(Some(&server_id))),
            )
            .await
            {
                Ok((transport, session_id, discovery)) => {
                    self.finish_control_recovery(
                        server_id,
                        server_addr,
                        token,
                        transport,
                        session_id.clone(),
                        discovery,
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
                        &format!("control recovery failed; waiting for next backoff: {error}"),
                    );
                    let _ = persist_runtime(&self.storage_path, &inner);
                }
            }
        }

        let failure_message = format!(
            "control recovery failed: {}",
            last_error.unwrap_or_else(|| "no retry opportunity available".to_string())
        );
        {
            let now = Utc::now().timestamp_millis();
            let mut inner = self.inner.lock().await;
            if let Some(server) = inner.servers.get_mut(&server_id) {
                server.status = "error".to_string();
                server.last_error = Some(failure_message.clone());
                server.session_id = None;
                server.last_checked_at = Some(now);
                server.updated_at = now;
            }
            inner.counters.recovery_failure = inner.counters.recovery_failure.saturating_add(1);
            inner.log("error", "reconnect", "RECONNECT_CONTROL_FAILED");
            let _ = persist_runtime(&self.storage_path, &inner);
        }

        Err(failure_message)
    }

    async fn finish_control_recovery(
        &self,
        server_id: String,
        server_addr: String,
        _token: String,
        transport: Arc<TcpTransport>,
        session_id: String,
        discovery: Value,
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
                server.discovery = discovery.clone();
                server.updated_at = now;
            }
            inner.transport = Some(transport.clone());
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
            inner.log("info", "reconnect", "RECONNECT_CONTROL_SUCCEEDED");
            inner.config.insert(
                format!("server.discovery.{server_id}"),
                discovery.to_string(),
            );
            persist_runtime(&self.storage_path, &inner)?;
        }
        let global_config = self.inner.lock().await.config.clone();
        self.server_connections.lock().await.insert(
            server_id.clone(),
            ServerConnectionRuntime {
                transport: transport.clone(),
                session_id: session_id.clone(),
            },
        );

        for tunnel in running_tunnels {
            self.request_required_for(
                &server_id,
                Command::TunnelRegister,
                tunnel_control_payload(&tunnel, &global_config),
            )
            .await?;
            self.update_worker_session(&tunnel.id, &session_id).await;
            let mut inner = self.inner.lock().await;
            let now = Utc::now().timestamp_millis();
            if let Some(record) = inner.tunnels.get_mut(&tunnel.id) {
                record.status = "recovering".to_string();
                record.started_at = record.started_at.or(Some(now));
                record.updated_at = now;
            }
            inner.log_tunnel(
                "info",
                &tunnel.id,
                "worker session context updated after control recovery",
            );
            persist_runtime(&self.storage_path, &inner)?;
        }

        Ok(())
    }

    async fn update_worker_session(&self, tunnel_id: &str, session_id: &str) {
        if let Some(LocalTunnelRuntime::Remote(runtime)) =
            self.local_tunnels.lock().await.get(tunnel_id)
        {
            runtime.update_session(session_id.to_string()).await;
        }
        self.worker_registry
            .update_tunnel_session(tunnel_id, session_id)
            .await;
    }

    pub async fn create_tunnel(
        &self,
        local_port: u16,
        remote_port: u16,
        protocol: String,
        server_id: Option<String>,
        local_host: Option<String>,
        host: Option<String>,
        path: Option<String>,
    ) -> Result<String, String> {
        let tunnel_id = Uuid::new_v4().to_string();
        let protocol = normalize_protocol(&protocol)?;
        let local_host = local_host.unwrap_or_else(|| "127.0.0.1".to_string());
        if local_port == 0 {
            return Err("LOCAL_PORT_REQUIRED".to_string());
        }
        let path = path.map(|value| normalize_http_path(&value)).or_else(|| {
            if protocol == "http" || protocol == "https" {
                Some("/".to_string())
            } else {
                None
            }
        });

        let server_id = self.resolve_tunnel_server_id(server_id.as_deref()).await?;
        self.ensure_connected_server_for(&server_id).await?;
        let occupied_ports = self.remote_occupied_ports(Some(&server_id)).await;
        let remote_port = if matches!(protocol.as_str(), "http" | "https") {
            normalize_http_tunnel_port(&protocol, remote_port)
        } else {
            remote_port
        };
        if remote_port == 0 {
            return Err("REMOTE_PORT_REQUIRED".to_string());
        }
        if occupied_ports.contains(&remote_port)
            && !allows_shared_public_port(&protocol, remote_port)
        {
            return Err(format!("REMOTE_PORT_OCCUPIED:{remote_port}"));
        }

        if let Some(host_name) = host.as_deref().filter(|value| !value.trim().is_empty()) {
            self.release_server_domain_binding(&server_id, host_name)
                .await;
        }

        self.request_if_connected_for(
            &server_id,
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
                server_id: Some(server_id),
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
                performance_mode: None,
                relay_workers: None,
                max_connections: None,
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
        // 串行化启动流程，避免快速重复操作创建多套失去管理的 Worker 池。
        let _start_guard = self.tunnel_start_lock.lock().await;
        self.ensure_tunnel_exists(&tunnel_id).await?;
        let (tunnel, server_id, server_addr, token, session_id) =
            self.active_relay_context(&tunnel_id).await?;
        let global_config = self.inner.lock().await.config.clone();
        if let Err(error) = self.ensure_connected_server_for(&server_id).await {
            self.record_tunnel_start_failure(&tunnel_id, &error).await;
            return Err(error);
        }

        if self
            .remote_occupied_ports(Some(&server_id))
            .await
            .contains(&tunnel.remote_port)
            && !self
                .remote_port_belongs_to_tunnel(&tunnel_id, tunnel.remote_port)
                .await
        {
            let message = format!("REMOTE_PORT_OCCUPIED:{}", tunnel.remote_port);
            self.record_tunnel_start_failure(&tunnel_id, &message).await;
            return Err(message);
        }

        if let Err(error) = self
            .request_required_for(
                &server_id,
                Command::TunnelStart,
                tunnel_control_payload(&tunnel, &global_config),
            )
            .await
        {
            let message = explain_server_control_error(&error);
            self.record_server_control_failure_for(&server_id, &message)
                .await;
            self.record_tunnel_start_failure(&tunnel_id, &message).await;
            return Err(message);
        }

        if !self.local_tunnels.lock().await.contains_key(&tunnel_id) {
            match start_remote_tunnel_runtime(
                &tunnel,
                &global_config,
                &server_addr,
                &token,
                &session_id,
                Arc::clone(&self.worker_registry),
            )
            .await
            {
                Ok(runtime) => {
                    self.local_tunnels
                        .lock()
                        .await
                        .insert(tunnel_id.clone(), runtime);
                }
                Err(error) => {
                    let message = explain_local_runtime_error(&tunnel, &error);
                    let _ = self
                        .request_if_connected_for(
                            &server_id,
                            Command::TunnelStop,
                            json!({ "id": tunnel_id }),
                        )
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
        let server_id = self.tunnel_server_id(&tunnel_id).await?;
        self.shutdown_local_tunnel(&tunnel_id).await?;
        if self
            .server_connections
            .lock()
            .await
            .contains_key(&server_id)
        {
            if let Err(error) = self
                .request_required_for(&server_id, Command::TunnelStop, json!({ "id": tunnel_id }))
                .await
            {
                tracing::warn!(
                    "server tunnel stop skipped because control connection is unavailable: {error}"
                );
            }
        }

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

        let (server_id, control_payload) = {
            let mut inner = self.inner.lock().await;
            let fallback_server_id = inner.active_server_id.clone();
            let tunnel = inner
                .tunnels
                .get_mut(&tunnel_id)
                .ok_or_else(|| "tunnel not found".to_string())?;
            if let Some(name) = patch.name {
                tunnel.name = name;
            }
            if let Some(server_id) = patch.server_id {
                tunnel.server_id = Some(server_id);
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
            if let Some(performance_mode) = patch.performance_mode {
                tunnel.performance_mode = Some(performance_mode);
            }
            if let Some(relay_workers) = patch.relay_workers {
                tunnel.relay_workers = Some(relay_workers.max(1));
            }
            if let Some(max_connections) = patch.max_connections {
                tunnel.max_connections = Some(max_connections.max(1));
            }
            if matches!(tunnel.protocol.as_str(), "http" | "https") {
                tunnel.remote_port =
                    normalize_http_tunnel_port(&tunnel.protocol, tunnel.remote_port);
            }
            tunnel.tls_version = if tunnel.protocol == "https" {
                Some("TLS".to_string())
            } else {
                None
            };
            tunnel.updated_at = Utc::now().timestamp_millis();
            let server_id = tunnel
                .server_id
                .clone()
                .or(fallback_server_id)
                .ok_or_else(|| "SERVER_REQUIRED_FOR_TUNNEL".to_string())?;
            let updated_tunnel = tunnel.clone();
            let global_config = inner.config.clone();
            inner.sync_domain_index_for_tunnel(&tunnel_id);
            sync_runtime_domains(&self.domain_repository, &inner.domains)?;
            inner.log("info", "tunnel", "tunnel configuration updated");
            persist_runtime(&self.storage_path, &inner)?;
            let control_payload = tunnel_control_payload(&updated_tunnel, &global_config);
            (server_id, control_payload)
        };

        self.request_if_connected_for(&server_id, Command::TunnelRegister, control_payload)
            .await?;
        Ok(())
    }

    pub async fn delete_tunnel(&self, tunnel_id: String) -> Result<(), String> {
        self.ensure_tunnel_exists(&tunnel_id).await?;
        let server_id = self.tunnel_server_id(&tunnel_id).await?;
        self.shutdown_local_tunnel(&tunnel_id).await?;
        self.request_if_connected_for(&server_id, Command::TunnelStop, json!({ "id": tunnel_id }))
            .await?;

        let mut inner = self.inner.lock().await;
        inner.tunnels.remove(&tunnel_id);
        inner.sync_domain_index_for_tunnel(&tunnel_id);
        sync_runtime_domains(&self.domain_repository, &inner.domains)?;
        inner.log_tunnel("info", &tunnel_id, "tunnel configuration deleted");
        persist_runtime(&self.storage_path, &inner)?;
        Ok(())
    }

    pub async fn local_services(&self) -> Value {
        local_service_discovery()
    }

    pub async fn probe_local_service(&self, host: String, port: u16) -> Value {
        probe_local_service(&host, port)
    }

    pub async fn remote_port_discovery(&self, server_id: Option<String>) -> Value {
        let inner = self.inner.lock().await;
        let discovery = selected_server_discovery(&inner, server_id.as_deref());
        discovery
            .pointer("/portDiscovery")
            .cloned()
            .unwrap_or_else(|| local_port_discovery(gate_reserved_ports(&inner)))
    }

    pub async fn check_remote_port(&self, server_id: Option<String>, port: u16) -> Value {
        let occupied = self.remote_occupied_ports(server_id.as_deref()).await;
        let available = public_port_available_for_tunnel(port, &occupied);
        json!({
            "port": port,
            "available": available,
            "status": if available { "available" } else { "occupied" },
            "reason": if available { "" } else { "REMOTE_PORT_OCCUPIED" },
            "checkedAt": Utc::now().timestamp_millis()
        })
    }

    pub async fn diagnose_tunnel(
        &self,
        local_host: String,
        local_port: u16,
        remote_port: u16,
        server_id: Option<String>,
    ) -> Value {
        let (connected, last_error) = {
            let inner = self.inner.lock().await;
            let server = server_id
                .as_deref()
                .and_then(|id| inner.servers.get(id))
                .or_else(|| {
                    inner
                        .active_server_id
                        .as_deref()
                        .and_then(|id| inner.servers.get(id))
                });
            (
                inner.connected && server.is_some_and(|item| item.status == "connected"),
                server
                    .and_then(|item| item.last_error.as_deref())
                    .map(str::to_string),
            )
        };
        let occupied = self.remote_occupied_ports(server_id.as_deref()).await;
        diagnose_start_failure(
            &local_host,
            local_port,
            remote_port,
            &occupied,
            connected,
            last_error.as_deref(),
        )
    }

    pub async fn config(&self) -> Value {
        let inner = self.inner.lock().await;
        json!(&inner.config)
    }

    pub async fn recommend_tunnel_performance(
        &self,
        mode: Option<String>,
    ) -> TunnelPerformanceRecommendation {
        let inner = self.inner.lock().await;
        let selected_mode = mode.or_else(|| inner.config.get(CONFIG_KEY_MODE).cloned());
        recommend_tunnel_performance(tunnel_performance::parse_performance_mode(
            selected_mode.as_deref(),
        ))
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

    pub async fn active_server_id(&self) -> Option<String> {
        self.inner.lock().await.active_server_id.clone()
    }

    pub async fn dashboard(&self) -> Value {
        self.sync_local_tunnel_metrics().await;
        let mut inner = self.inner.lock().await;
        inner.sync_tunnel_state();
        let mut dashboard = dashboard_json(&inner);
        drop(inner);
        let worker_statuses = self.worker_registry.tunnel_statuses().await;
        apply_worker_health_to_dashboard(&mut dashboard, &worker_statuses);
        dashboard["workers"] = self.worker_registry.snapshot().await;
        dashboard
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
        let mut entries = {
            let mut inner = self.inner.lock().await;
            inner.sync_tunnel_state();
            inner.logs.iter().map(|log| json!(log)).collect::<Vec<_>>()
        };

        entries.extend(self.server_access_log_entries().await);
        entries.sort_by_key(|entry| entry.get("timestamp").and_then(Value::as_i64).unwrap_or(0));
        Value::Array(entries)
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
        let server_ids = {
            let inner = self.inner.lock().await;
            inner
                .servers
                .iter()
                .filter(|(_, server)| server.auto_connect)
                .map(|(id, _)| id.clone())
                .collect::<Vec<_>>()
        };

        if server_ids.is_empty() {
            return Ok(None);
        }

        let mut last_session = None;
        for server_id in server_ids {
            match self
                .restore_server_with_backoff(&server_id, &default_reconnect_delays())
                .await
            {
                Ok(session_id) => last_session = Some(session_id),
                Err(error) => {
                    tracing::warn!("startup recovery failed for server `{server_id}`: {error}");
                }
            }
        }

        Ok(last_session)
    }

    async fn shutdown_local_tunnel(&self, tunnel_id: &str) -> Result<(), String> {
        let runtime = self.local_tunnels.lock().await.remove(tunnel_id);
        if let Some(runtime) = runtime {
            runtime.shutdown().await?;
        }
        self.worker_registry.remove_tunnel(tunnel_id).await;
        Ok(())
    }

    async fn shutdown_all_local_tunnels(&self) -> Result<(), String> {
        let runtimes = {
            let mut local_tunnels = self.local_tunnels.lock().await;
            std::mem::take(&mut *local_tunnels)
        };

        for (tunnel_id, runtime) in runtimes {
            runtime.shutdown().await?;
            self.worker_registry.remove_tunnel(&tunnel_id).await;
        }

        Ok(())
    }

    async fn shutdown_local_tunnels_for_server(&self, server_id: &str) -> Result<(), String> {
        let tunnel_ids = {
            let inner = self.inner.lock().await;
            inner
                .tunnels
                .values()
                .filter(|tunnel| tunnel.server_id.as_deref() == Some(server_id))
                .map(|tunnel| tunnel.id.clone())
                .collect::<Vec<_>>()
        };

        for tunnel_id in tunnel_ids {
            self.shutdown_local_tunnel(&tunnel_id).await?;
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

    async fn record_server_control_failure_for(&self, server_id: &str, reason: &str) {
        let transport = {
            let transport = self
                .server_connections
                .lock()
                .await
                .remove(server_id)
                .map(|connection| connection.transport);
            let mut inner = self.inner.lock().await;
            if let Some(server) = inner.servers.get_mut(server_id) {
                server.status = "error".to_string();
                server.last_error = Some(reason.to_string());
                server.session_id = None;
                server.updated_at = Utc::now().timestamp_millis();
            }
            inner.counters.disconnect_total = inner.counters.disconnect_total.saturating_add(1);
            inner.log("error", "server", reason);
            self.sync_primary_from_connections_locked(&mut inner).await;
            let _ = persist_runtime(&self.storage_path, &inner);
            transport
        };

        if let Some(transport) = transport {
            let _ = transport.disconnect().await;
        }
    }

    async fn record_server_control_failure(&self, reason: &str) {
        let active_server_id = {
            let inner = self.inner.lock().await;
            inner.active_server_id.clone()
        };
        if let Some(server_id) = active_server_id {
            self.record_server_control_failure_for(&server_id, reason)
                .await;
        }
    }

    async fn server_access_log_entries(&self) -> Vec<Value> {
        let server_ids = self
            .server_connections
            .lock()
            .await
            .keys()
            .cloned()
            .collect::<Vec<_>>();
        let mut entries = Vec::new();

        for server_id in server_ids {
            let statistics = match tokio::time::timeout(
                SERVER_LOG_SNAPSHOT_TIMEOUT,
                self.request_for(&server_id, Command::StatisticsQuery, json!({})),
            )
            .await
            {
                Ok(Ok(statistics)) => statistics,
                _ => continue,
            };

            let Some(access_logs) = statistics_access_logs(&statistics) else {
                continue;
            };

            for access in access_logs {
                entries.push(server_access_log_entry(&server_id, access));
            }
        }

        entries
    }

    async fn sync_local_tunnel_metrics(&self) {
        let snapshots = {
            let local_tunnels = self.local_tunnels.lock().await;
            local_tunnels
                .iter()
                .map(|(id, runtime)| (id.clone(), runtime.snapshot()))
                .collect::<Vec<_>>()
        };

        let mut inner = self.inner.lock().await;
        let day_before = inner.counters.today_traffic_date.clone();
        bootstrap_traffic_counters(&mut inner);
        ensure_traffic_day(&mut inner.counters);

        if snapshots.is_empty() {
            if day_before != inner.counters.today_traffic_date {
                let _ = persist_runtime(&self.storage_path, &inner);
            }
            return;
        }

        let now = Utc::now().timestamp_millis();
        let mut traffic_delta = 0_u64;
        for (id, snapshot) in snapshots {
            let previous_bytes = inner
                .tunnels
                .get(&id)
                .map(tunnel_traffic_bytes)
                .unwrap_or(0);
            let next_bytes = snapshot_traffic_bytes(&snapshot);
            traffic_delta =
                traffic_delta.saturating_add(traffic_bytes_delta(previous_bytes, next_bytes));

            let Some(tunnel) = inner.tunnels.get_mut(&id) else {
                continue;
            };

            let elapsed_seconds = now.saturating_sub(tunnel.last_sample_at).max(0) as f64 / 1000.0;
            let mut upload_speed_bps =
                bytes_per_second(snapshot.upload_bytes, tunnel.upload_bytes, elapsed_seconds);
            let download_speed_bps = bytes_per_second(
                snapshot.download_bytes,
                tunnel.download_bytes,
                elapsed_seconds,
            );
            if upload_speed_bps == 0.0
                && download_speed_bps == 0.0
                && snapshot.current_speed_bps > 0
            {
                upload_speed_bps = snapshot.current_speed_bps as f64;
            }

            tunnel.connections = snapshot.active_connections;
            tunnel.upload_bytes = snapshot.upload_bytes;
            tunnel.download_bytes = snapshot.download_bytes;
            tunnel.upload_speed_bps = upload_speed_bps;
            tunnel.download_speed_bps = download_speed_bps;
            tunnel.uptime_seconds = snapshot.runtime_seconds;
            tunnel.request_count = snapshot.request_count;
            tunnel.success_count = snapshot.success_count;
            tunnel.total_latency_ms = snapshot.total_latency_ms;
            tunnel.recent_requests = snapshot.recent_requests;
            tunnel.last_sample_at = now;
        }

        inner.counters.today_traffic_bytes = inner
            .counters
            .today_traffic_bytes
            .saturating_add(traffic_delta);
        inner.counters.total_traffic_bytes = inner
            .counters
            .total_traffic_bytes
            .saturating_add(traffic_delta);

        let _ = persist_runtime(&self.storage_path, &inner);
    }

    async fn active_relay_context(
        &self,
        tunnel_id: &str,
    ) -> Result<(TunnelRecord, String, String, String, String), String> {
        let (tunnel, server_id, server_addr, token, stored_session_id) = {
            let inner = self.inner.lock().await;
            let tunnel = inner
                .tunnels
                .get(tunnel_id)
                .cloned()
                .ok_or_else(|| "tunnel not found".to_string())?;
            let server_id = tunnel
                .server_id
                .clone()
                .or_else(|| inner.active_server_id.clone())
                .ok_or_else(|| tunnel_server_not_ready_message(&inner))?;
            let server = inner
                .servers
                .get(&server_id)
                .ok_or_else(|| tunnel_server_not_ready_message(&inner))?;
            (
                tunnel,
                server_id,
                format!("{}:{}", server.host, server.port),
                server.token.clone(),
                server.session_id.clone(),
            )
        };
        let session_id = self
            .server_connections
            .lock()
            .await
            .get(&server_id)
            .map(|connection| connection.session_id.clone())
            .or(stored_session_id)
            .ok_or_else(|| "SERVER_SESSION_MISSING".to_string())?;
        Ok((tunnel, server_id, server_addr, token, session_id))
    }

    async fn ensure_tunnel_exists(&self, tunnel_id: &str) -> Result<(), String> {
        let inner = self.inner.lock().await;
        if inner.tunnels.contains_key(tunnel_id) {
            Ok(())
        } else {
            Err("tunnel not found".to_string())
        }
    }

    async fn tunnel_server_id(&self, tunnel_id: &str) -> Result<String, String> {
        let inner = self.inner.lock().await;
        let tunnel = inner
            .tunnels
            .get(tunnel_id)
            .ok_or_else(|| "tunnel not found".to_string())?;
        tunnel
            .server_id
            .clone()
            .or_else(|| inner.active_server_id.clone())
            .ok_or_else(|| "SERVER_REQUIRED_FOR_TUNNEL".to_string())
    }

    async fn resolve_tunnel_server_id(&self, server_id: Option<&str>) -> Result<String, String> {
        let inner = self.inner.lock().await;
        if let Some(server_id) = server_id.filter(|value| !value.trim().is_empty()) {
            if inner.servers.contains_key(server_id) {
                return Ok(server_id.to_string());
            }
            return Err("SERVER_NOT_FOUND_FOR_TUNNEL".to_string());
        }
        inner
            .active_server_id
            .clone()
            .or_else(|| {
                inner
                    .servers
                    .values()
                    .find(|server| server.status == "connected")
                    .map(|server| server.id.clone())
            })
            .ok_or_else(|| tunnel_server_not_ready_message(&inner))
    }

    async fn ensure_connected_server_for(&self, server_id: &str) -> Result<(), String> {
        if self.server_connections.lock().await.contains_key(server_id) {
            return Ok(());
        }

        let inner = self.inner.lock().await;
        Err(tunnel_server_not_ready_message(&inner))
    }

    async fn remote_occupied_ports(&self, server_id: Option<&str>) -> BTreeSet<u16> {
        let inner = self.inner.lock().await;
        let discovery = selected_server_discovery(&inner, server_id);
        ports_from_discovery(&discovery)
            .into_iter()
            .chain(gate_reserved_ports(&inner))
            .collect()
    }

    async fn remote_port_belongs_to_tunnel(&self, tunnel_id: &str, remote_port: u16) -> bool {
        let inner = self.inner.lock().await;
        inner
            .tunnels
            .get(tunnel_id)
            .map(|tunnel| tunnel.remote_port == remote_port)
            .unwrap_or(false)
    }

    async fn request_if_connected_for(
        &self,
        server_id: &str,
        command: Command,
        body: Value,
    ) -> Result<(), String> {
        if !self.server_connections.lock().await.contains_key(server_id) {
            return Ok(());
        }
        let response = self.request_for(server_id, command, body).await?;
        ensure_ok(&response)
    }

    /// 清理服务器端残留的域名绑定（客户端隧道列表为空时，服务器可能仍记着旧隧道 ID）。
    pub async fn release_server_domain_binding(&self, server_id: &str, host: &str) {
        let host = host.trim();
        if host.is_empty() {
            return;
        }
        let body = json!({ "host": host });
        let _ = self
            .request_if_connected_for(server_id, Command::DomainUnbind, body.clone())
            .await;
        let _ = self
            .request_if_connected_for(server_id, Command::DomainDelete, body)
            .await;
    }

    async fn verify_server_connection(&self, server_id: &str) -> bool {
        let _io_guard = self.control_io_lock.lock().await;
        let transport = self
            .server_connections
            .lock()
            .await
            .get(server_id)
            .map(|connection| connection.transport.clone());
        let Some(transport) = transport else {
            return false;
        };

        let payload = self.heartbeat_payload_for(server_id).await;
        match send_request(&transport, Command::HeartbeatPing, payload).await {
            Ok(response) => ensure_ok(&response).is_ok(),
            Err(_) => false,
        }
    }

    async fn verify_server_connections(&self) {
        let server_ids: Vec<String> = self
            .server_connections
            .lock()
            .await
            .keys()
            .cloned()
            .collect();

        for server_id in server_ids {
            let already_recovering = {
                let inner = self.inner.lock().await;
                inner
                    .servers
                    .get(&server_id)
                    .map(|server| server.status == "recovering")
                    .unwrap_or(false)
            };
            if already_recovering {
                continue;
            }

            if self.verify_server_connection(&server_id).await {
                continue;
            }

            self.record_control_disconnect_for(
                &server_id,
                "connection health check failed; starting control recovery",
            )
            .await;
            if let Err(error) = self
                .restore_server_with_backoff(&server_id, &default_reconnect_delays())
                .await
            {
                tracing::warn!("control recovery after health check failed: {error}");
            }
        }
    }

    async fn sync_primary_from_connections_locked(&self, inner: &mut RuntimeInner) {
        let connections = self.server_connections.lock().await;
        inner.connected = !connections.is_empty();
        if connections.is_empty() {
            inner.transport = None;
            inner.session_id = None;
            inner.server_addr = None;
            inner.active_server_id = None;
            return;
        }

        let primary_id = inner
            .active_server_id
            .clone()
            .filter(|id| connections.contains_key(id))
            .or_else(|| connections.keys().next().cloned());

        if let Some(primary_id) = primary_id {
            if let Some(connection) = connections.get(&primary_id) {
                inner.transport = Some(connection.transport.clone());
                inner.session_id = Some(connection.session_id.clone());
            }
            inner.active_server_id = Some(primary_id.clone());
            if let Some(server) = inner.servers.get(&primary_id) {
                inner.server_addr = Some(format!("{}:{}", server.host, server.port));
            }
        }
    }

    async fn request_required_for(
        &self,
        server_id: &str,
        command: Command,
        body: Value,
    ) -> Result<(), String> {
        let response = self.request_for(server_id, command, body).await?;
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
        record_server_discovery_from_response(&mut inner, &response);
        persist_runtime(&self.storage_path, &inner)?;
        Ok(response)
    }

    async fn request_for(
        &self,
        server_id: &str,
        command: Command,
        body: Value,
    ) -> Result<Value, String> {
        let _io_guard = self.control_io_lock.lock().await;
        let transport = {
            {
                let mut inner = self.inner.lock().await;
                inner.counters.request_total += 1;
            }
            self.server_connections
                .lock()
                .await
                .get(server_id)
                .map(|connection| connection.transport.clone())
        }
        .ok_or_else(|| "runtime backend is not connected".to_string())?;

        let response = send_request(&transport, command, body).await?;
        let mut inner = self.inner.lock().await;
        inner.counters.response_total += 1;
        record_server_discovery_from_response_for(&mut inner, server_id, &response);
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
                    "failed to start local TCP listener {} -> {}: {}",
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
                    "failed to start local HTTP listener {} -> {}: {}",
                    listen_addr, target_addr, error
                )
            })?;
            Ok(LocalTunnelRuntime::Http(runtime))
        }
        "https" => Err("HTTPS_LOCAL_CERTIFICATE_REQUIRED".to_string()),
        protocol => Err(format!("UNSUPPORTED_TUNNEL_PROTOCOL:{protocol}")),
    }
}

async fn start_remote_tunnel_runtime(
    tunnel: &TunnelRecord,
    global_config: &BTreeMap<String, String>,
    server_addr: &str,
    token: &str,
    session_id: &str,
    worker_registry: Arc<RelayWorkerRegistry>,
) -> Result<LocalTunnelRuntime, String> {
    let (server_host, server_port) = parse_server_addr(server_addr)?;
    let target_addr = resolve_target_addr(&tunnel.local_host, tunnel.local_port).await?;
    verify_target_reachable(target_addr).await?;

    let (shutdown, shutdown_rx) = watch::channel(false);
    let stats = Arc::new(RelayTunnelStats::default());
    let session = Arc::new(RwLock::new(RelaySessionContext {
        session_id: session_id.to_string(),
    }));
    stats.started_at_millis.store(
        Utc::now().timestamp_millis().max(0) as u64,
        Ordering::Relaxed,
    );

    let maximum_workers = resolve_tunnel_performance(
        tunnel.performance_mode.as_deref(),
        tunnel.relay_workers,
        tunnel.max_connections,
        global_config,
    )
    .relay_workers;
    let pool = RelayWorkerPoolConfig {
        maximum: maximum_workers.max(1),
        minimum: DEFAULT_MINIMUM_RELAY_WORKERS.min(maximum_workers.max(1)),
    };
    let template = RelayWorkerConfig {
        worker_id: 0,
        server_host,
        server_port,
        token: token.to_string(),
        session: Arc::clone(&session),
        tunnel_id: tunnel.id.clone(),
        protocol: tunnel.protocol.clone(),
        target_addr,
    };
    let supervisor_stats = Arc::clone(&stats);
    let supervisor_registry = Arc::clone(&worker_registry);
    let tasks = vec![tokio::spawn(async move {
        relay_worker_supervisor_loop(
            template,
            pool,
            supervisor_stats,
            supervisor_registry,
            shutdown_rx,
        )
        .await;
    })];

    Ok(LocalTunnelRuntime::Remote(RelayTunnelRuntime {
        shutdown,
        tasks,
        stats,
        session,
    }))
}

async fn relay_worker_supervisor_loop(
    template: RelayWorkerConfig,
    pool: RelayWorkerPoolConfig,
    stats: Arc<RelayTunnelStats>,
    registry: Arc<RelayWorkerRegistry>,
    mut shutdown: watch::Receiver<bool>,
) {
    let mut workers = BTreeMap::<usize, (watch::Sender<bool>, JoinHandle<()>)>::new();
    let mut next_worker_id = 0_usize;
    let refill_notify = Arc::new(Notify::new());

    let spawn_worker =
        |worker_id: usize, workers: &mut BTreeMap<usize, (watch::Sender<bool>, JoinHandle<()>)>| {
            let (worker_shutdown, worker_shutdown_rx) = watch::channel(false);
            let mut config = template.clone();
            config.worker_id = worker_id;
            let worker_stats = Arc::clone(&stats);
            let worker_registry = Arc::clone(&registry);
            let worker_refill_notify = Arc::clone(&refill_notify);
            let task = tokio::spawn(async move {
                relay_worker_loop(
                    config,
                    worker_stats,
                    worker_registry,
                    worker_refill_notify,
                    worker_shutdown_rx,
                )
                .await;
            });
            workers.insert(worker_id, (worker_shutdown, task));
        };

    while workers.len() < pool.minimum {
        spawn_worker(next_worker_id, &mut workers);
        next_worker_id = next_worker_id.saturating_add(1);
    }

    loop {
        tokio::select! {
            _ = shutdown.changed() => {
                if *shutdown.borrow() {
                    break;
                }
            }
            _ = refill_notify.notified() => {}
            _ = sleep(WORKER_SUPERVISOR_INTERVAL) => {}
        }

        for (worker_id, (_, task)) in &workers {
            if task.is_finished() {
                registry
                    .remove(&format!("{}:{worker_id}", template.tunnel_id))
                    .await;
            }
        }
        workers.retain(|_, (_, task)| !task.is_finished());

        // 已有 Worker 完成转发后会自行重新挂载；保持固定热池可避免重复补位造成连接风暴。
        let desired = relay_pool_target(pool);

        while workers.len() < desired {
            spawn_worker(next_worker_id, &mut workers);
            next_worker_id = next_worker_id.saturating_add(1);
        }

        // 固定热池不会随瞬时并发永久扩张，因此无需关闭可能正在转发的连接。
    }

    for (_, (worker_shutdown, task)) in workers {
        let _ = worker_shutdown.send(true);
        let _ = task.await;
    }
}

fn relay_pool_target(pool: RelayWorkerPoolConfig) -> usize {
    // maximum 仅作为配置防线；运行时固定维持 minimum，避免一次性 Worker 被重复补位后永久膨胀。
    pool.minimum.min(pool.maximum)
}

async fn relay_worker_loop(
    config: RelayWorkerConfig,
    stats: Arc<RelayTunnelStats>,
    registry: Arc<RelayWorkerRegistry>,
    refill_notify: Arc<Notify>,
    mut shutdown: watch::Receiver<bool>,
) {
    // relay worker 失败时按固定退避序列重连，成功转发后立即补回空闲 worker。
    let worker_key = format!("{}:{}", config.tunnel_id, config.worker_id);
    registry
        .register(
            worker_key.clone(),
            config.tunnel_id.clone(),
            config.session.read().await.session_id.clone(),
        )
        .await;
    let mut retry_attempt = 0_usize;
    loop {
        if *shutdown.borrow() {
            break;
        }

        let delay = match relay_worker_once(
            config.clone(),
            Arc::clone(&stats),
            Arc::clone(&registry),
            &worker_key,
            Arc::clone(&refill_notify),
            shutdown.clone(),
        )
        .await
        {
            Ok(()) => {
                retry_attempt = 0;
                Duration::ZERO
            }
            Err(error) => {
                stats.failed_connections.fetch_add(1, Ordering::Relaxed);
                let attach_failure = error.starts_with("attach:");
                registry
                    .update(&worker_key, |worker| {
                        worker.failure_count = worker.failure_count.saturating_add(1);
                        if attach_failure {
                            worker.attach_failures = worker.attach_failures.saturating_add(1);
                            worker.attach_failure_count =
                                worker.attach_failure_count.saturating_add(1);
                        }
                        worker.reconnect_count = worker.reconnect_count.saturating_add(1);
                        worker.state = if attach_failure && worker.attach_failures >= 5 {
                            "recovering"
                        } else {
                            "connecting"
                        }
                        .to_string();
                        worker.ready = false;
                        worker.healthy = false;
                        worker.last_attach_error = Some(error);
                    })
                    .await;
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
    registry
        .update(&worker_key, |worker| {
            worker.state = "dead".to_string();
            worker.alive = false;
            worker.ready = false;
            worker.healthy = false;
        })
        .await;
}

async fn relay_worker_once(
    config: RelayWorkerConfig,
    stats: Arc<RelayTunnelStats>,
    registry: Arc<RelayWorkerRegistry>,
    worker_key: &str,
    refill_notify: Arc<Notify>,
    mut shutdown: watch::Receiver<bool>,
) -> Result<(), String> {
    registry
        .update(worker_key, |worker| {
            worker.state = "connecting".to_string();
            worker.alive = true;
            worker.ready = false;
            worker.healthy = false;
        })
        .await;
    let attach_started = Instant::now();
    let mut stream = tokio::time::timeout(
        RELAY_ATTACH_CONNECT_TIMEOUT,
        TcpStream::connect((config.server_host.as_str(), config.server_port)),
    )
    .await
    .map_err(|_| "attach: connecting server relay entry timed out".to_string())?
    .map_err(|error| format!("attach: failed to connect server relay entry: {error}"))?;
    stream
        .set_nodelay(true)
        .map_err(|error| format!("attach: failed to set relay TCP_NODELAY: {error}"))?;
    let keepalive = socket2::SockRef::from(&stream);
    let _ = keepalive.set_keepalive(true);

    let protocol = ProtocolBuilder::new().build();
    let connect = Message::request(
        Command::TunnelRelayConnect,
        Body::Json(json!({
            "token": &config.token,
            "sessionId": &config.session.read().await.session_id,
            "tunnelId": &config.tunnel_id,
            "protocol": &config.protocol,
            // workerId 使用字符串，兼容服务端 HashMap 注册表并避免数字类型解析差异。
            "workerId": config.worker_id.to_string()
        })),
        Metadata::default(),
    );
    tokio::time::timeout(
        RELAY_ATTACH_HANDSHAKE_TIMEOUT,
        write_protocol_message(&mut stream, &protocol, &connect),
    )
    .await
    .map_err(|_| "attach: writing relay handshake timed out".to_string())?
    .map_err(|error| format!("attach: {error}"))?;

    let response = tokio::time::timeout(
        RELAY_ATTACH_HANDSHAKE_TIMEOUT,
        read_protocol_message(&mut stream, &protocol),
    )
    .await
    .map_err(|_| "attach: reading relay handshake timed out".to_string())?
    .map_err(|error| format!("attach: {error}"))?
    .ok_or_else(|| "attach: server closed relay handshake connection".to_string())?;
    match response.body {
        Body::Json(value) => ensure_ok(&value).map_err(|error| format!("attach: {error}"))?,
        _ => return Err("attach: server relay handshake response was not JSON".to_string()),
    }
    let attach_latency = attach_started
        .elapsed()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64;
    registry
        .update(worker_key, |worker| {
            worker.state = "idle".to_string();
            worker.success_count = worker.success_count.saturating_add(1);
            // attach_failures 表示连续失败次数；成功后清零，但 failure_count 保留累计值。
            worker.attach_failures = 0;
            let now = Utc::now().timestamp_millis();
            worker.last_attach = Some(now);
            worker.last_attach_success = Some(now);
            worker.last_successful_attach = Some(now);
            worker.latency_ms = Some(attach_latency);
            worker.attach_latency_ms = Some(attach_latency);
            worker.ready = true;
            worker.healthy = true;
            worker.last_attach_error = None;
        })
        .await;

    let start = tokio::select! {
        result = read_protocol_message(&mut stream, &protocol) => {
            result?.ok_or_else(|| "server closed idle relay worker".to_string())?
        }
        _ = shutdown.changed() => return Ok(()),
    };

    if start.header.command != Command::TunnelRelayStart {
        return Err(format!(
            "unsupported relay command received: {}",
            start.header.command
        ));
    }

    registry
        .update(worker_key, |worker| {
            worker.state = "serving".to_string();
            worker.ready = false;
            worker.last_serve = Some(Utc::now().timestamp_millis());
        })
        .await;
    // 一旦 worker 被公网请求占用，立即唤醒 supervisor 补位，避免等下一轮定时巡检。
    refill_notify.notify_one();
    stats.total_connections.fetch_add(1, Ordering::Relaxed);
    let mut target = TcpStream::connect(config.target_addr)
        .await
        .map_err(|error| {
            format!(
                "failed to connect local service {}: {error}",
                config.target_addr
            )
        })?;
    target
        .set_nodelay(true)
        .map_err(|error| format!("failed to set local service TCP_NODELAY: {error}"))?;

    stats.active_connections.fetch_add(1, Ordering::Relaxed);
    let copy = tokio::io::copy_bidirectional_with_sizes(
        &mut stream,
        &mut target,
        RELAY_COPY_BUFFER_SIZE,
        RELAY_COPY_BUFFER_SIZE,
    );
    tokio::pin!(copy);
    let result = tokio::select! {
        result = &mut copy => result.map_err(|error| format!("relay bidirectional copy failed: {error}")),
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
    registry
        .update(worker_key, |worker| {
            worker.bytes_in = worker.bytes_in.saturating_add(upload_bytes);
            worker.bytes_out = worker.bytes_out.saturating_add(download_bytes);
            worker.last_heartbeat = Utc::now().timestamp_millis();
        })
        .await;

    Ok(())
}

#[cfg(test)]
fn local_listen_addr(remote_port: u16) -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), remote_port)
}

async fn resolve_target_addr(host: &str, port: u16) -> Result<SocketAddr, String> {
    let host = host.trim();
    if host.is_empty() {
        return Err("LOCAL_SERVICE_HOST_EMPTY".to_string());
    }

    let mut addrs = tokio::net::lookup_host((host, port))
        .await
        .map_err(|error| {
            format!("failed to resolve local service address {host}:{port}: {error}")
        })?;
    addrs
        .next()
        .ok_or_else(|| format!("failed to resolve local service address {host}:{port}"))
}

async fn verify_target_reachable(target_addr: SocketAddr) -> Result<(), String> {
    match tokio::time::timeout(
        Duration::from_secs(2),
        tokio::net::TcpStream::connect(target_addr),
    )
    .await
    {
        Ok(Ok(_stream)) => Ok(()),
        Ok(Err(error)) => Err(format!(
            "local service is unreachable {target_addr}: {error}"
        )),
        Err(_) => Err(format!("local service connection timed out {target_addr}")),
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
        return "NO_AVAILABLE_SERVER_CONFIG".to_string();
    }

    if let Some(server_id) = inner.active_server_id.as_deref() {
        if let Some(server) = inner.servers.get(server_id) {
            return format!(
                "server `{}` has no active connection; reconnect {}:{} after confirming the server process is running",
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
            "server is disconnected; tunnel cannot start until `{}` ({}:{}) reconnects",
            server.name, server.host, server.port
        );
    }

    "SERVER_DISCONNECTED_TUNNEL_START_BLOCKED".to_string()
}

fn explain_server_control_error(error: &str) -> String {
    format!(
        "server unavailable or control connection disconnected: {error}; check server process and token"
    )
}

fn explain_local_runtime_error(tunnel: &TunnelRecord, error: &str) -> String {
    let lower = error.to_ascii_lowercase();
    if lower.contains("10048")
        || lower.contains("address already in use")
        || lower.contains("only one usage")
    {
        return format!(
            "port {} is already in use and cannot be used as the local test entry",
            tunnel.remote_port
        );
    }

    if error.contains("local service is unreachable")
        || error.contains("local service connection timed out")
    {
        return format!(
            "local service {}:{} is unreachable; tunnel stopped; detail: {}",
            tunnel.local_host, tunnel.local_port, error
        );
    }

    if error.contains("failed to resolve local service address") {
        return format!(
            "local service address is invalid: {}:{}; detail: {}",
            tunnel.local_host, tunnel.local_port, error
        );
    }

    format!(
        "local runtime failed to start; check local service {}:{}, public port {}, protocol {}; detail: {}",
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

fn stop_running_tunnels_for_server(inner: &mut RuntimeInner, server_id: &str) {
    let now = Utc::now().timestamp_millis();
    for tunnel in inner.tunnels.values_mut() {
        if tunnel.status == "running" && tunnel.server_id.as_deref() == Some(server_id) {
            tunnel.status = "stopped".to_string();
            tunnel.upload_speed_bps = 0.0;
            tunnel.download_speed_bps = 0.0;
            tunnel.connections = 0;
            tunnel.started_at = None;
            tunnel.updated_at = now;
        }
    }
}

fn connected_server_count(inner: &RuntimeInner) -> usize {
    inner
        .servers
        .values()
        .filter(|server| server.status == "connected")
        .count()
}

fn default_config() -> BTreeMap<String, String> {
    let mut config = BTreeMap::new();
    config.insert("runtime.mode".to_string(), "production".to_string());
    config.insert("authentication.required".to_string(), "true".to_string());
    config.insert("heartbeat.interval_ms".to_string(), "15000".to_string());
    config.insert("network.transport".to_string(), "tcp".to_string());
    config.insert(CONFIG_KEY_MODE.to_string(), "auto".to_string());
    config
}

fn heartbeat_reconnect_delays() -> Vec<Duration> {
    HEARTBEAT_RECONNECT_DELAYS_SECS
        .iter()
        .copied()
        .map(Duration::from_secs)
        .collect()
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

fn reconnect_delay(attempt: usize) -> Duration {
    let index = attempt.min(RECONNECT_DELAYS_SECS.len().saturating_sub(1));
    Duration::from_secs(RECONNECT_DELAYS_SECS[index])
}

fn runtime_client_id(active_server_id: Option<&str>) -> String {
    active_server_id
        .map(|server_id| format!("gate-client:{server_id}"))
        .unwrap_or_else(|| "gate-client:standalone".to_string())
}

pub(crate) fn runtime_store_path() -> PathBuf {
    // 平台目录由单一入口解析，macOS 使用 Application Support。
    app_data_dir()
        .unwrap_or_else(|| PathBuf::from(".gate"))
        .join("client-runtime.json")
}

pub(crate) fn domain_store_path() -> PathBuf {
    if let Some(value) = env::var_os("GATE_DOMAIN_DB") {
        return PathBuf::from(value);
    }

    runtime_data_dir().join("domains.sqlite3")
}

pub(crate) fn runtime_data_dir() -> PathBuf {
    runtime_store_path()
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| PathBuf::from(".gate"))
}

pub(crate) fn certificate_store_root() -> PathBuf {
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

fn mark_restored_runtime_inactive(inner: &mut RuntimeInner) {
    inner.transport = None;
    inner.connected = false;
    inner.server_addr = None;
    inner.session_id = None;
    inner.active_server_id = None;
    deactivate_persisted_servers(&mut inner.servers, "BACKUP_RESTORED_MANUAL_RECONNECT");
    deactivate_persisted_tunnels(&mut inner.tunnels);
}

fn deactivate_persisted_servers(servers: &mut BTreeMap<String, ServerRecord>, reason: &str) {
    let now = Utc::now().timestamp_millis();
    for server in servers.values_mut() {
        if matches!(
            server.status.as_str(),
            "connected" | "connecting" | "recovering"
        ) {
            server.status = "disconnected".to_string();
            server.session_id = None;
            server.last_error = Some(reason.to_string());
            server.updated_at = now;
        }
    }
}

fn deactivate_persisted_tunnels(tunnels: &mut BTreeMap<String, TunnelRecord>) {
    let now = Utc::now().timestamp_millis();
    for tunnel in tunnels.values_mut() {
        if matches!(
            tunnel.status.as_str(),
            "running" | "starting" | "restarting" | "stopping" | "recovering"
        ) {
            tunnel.status = "stopped".to_string();
            tunnel.upload_speed_bps = 0.0;
            tunnel.download_speed_bps = 0.0;
            tunnel.connections = 0;
            tunnel.started_at = None;
            tunnel.updated_at = now;
        }
    }
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

const DEFAULT_HTTP_PUBLIC_PORT: u16 = 8880;
const DEFAULT_HTTPS_PUBLIC_PORT: u16 = 8443;

fn default_public_port(protocol: &str) -> u16 {
    match protocol {
        "https" => DEFAULT_HTTPS_PUBLIC_PORT,
        "http" => DEFAULT_HTTP_PUBLIC_PORT,
        _ => 0,
    }
}

fn should_omit_public_port(protocol: &str, port: u16) -> bool {
    matches!(protocol, "http" | "https") && matches!(port, 80 | 443)
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
        "discovery": server.discovery,
        "createdAt": server.created_at,
        "updatedAt": server.updated_at
    })
}

fn record_server_discovery_from_response(inner: &mut RuntimeInner, response: &Value) {
    let Some(discovery) = response.pointer("/data/server").cloned() else {
        return;
    };
    let Some(server_id) = inner.active_server_id.clone() else {
        return;
    };
    if let Some(server) = inner.servers.get_mut(&server_id) {
        server.discovery = discovery.clone();
        server.last_checked_at = Some(Utc::now().timestamp_millis());
    }
    inner
        .config
        .insert("server.discovery".to_string(), discovery.to_string());
}

fn record_server_discovery_from_response_for(
    inner: &mut RuntimeInner,
    server_id: &str,
    response: &Value,
) {
    let Some(discovery) = response.pointer("/data/server").cloned() else {
        return;
    };
    if let Some(server) = inner.servers.get_mut(server_id) {
        server.discovery = discovery.clone();
        server.last_checked_at = Some(Utc::now().timestamp_millis());
    }
    inner.config.insert(
        format!("server.discovery.{server_id}"),
        discovery.to_string(),
    );
}

fn selected_server_discovery(inner: &RuntimeInner, server_id: Option<&str>) -> Value {
    server_id
        .and_then(|id| inner.servers.get(id))
        .or_else(|| {
            inner
                .active_server_id
                .as_deref()
                .and_then(|id| inner.servers.get(id))
        })
        .map(|server| server.discovery.clone())
        .filter(|value| !value.is_null())
        .or_else(|| {
            inner
                .config
                .get("server.discovery")
                .and_then(|raw| serde_json::from_str::<Value>(raw).ok())
        })
        .unwrap_or(Value::Null)
}

fn ports_from_discovery(discovery: &Value) -> BTreeSet<u16> {
    let mut ports = BTreeSet::new();
    for key in ["occupiedPorts", "gateReservedPorts", "systemReservedPorts"] {
        if let Some(items) = discovery
            .pointer(&format!("/portDiscovery/{key}"))
            .and_then(Value::as_array)
        {
            for item in items {
                if let Some(port) = item
                    .get("port")
                    .and_then(Value::as_u64)
                    .and_then(|value| u16::try_from(value).ok())
                {
                    ports.insert(port);
                }
            }
        }
    }
    ports
}

fn gate_reserved_ports(inner: &RuntimeInner) -> BTreeSet<u16> {
    inner
        .tunnels
        .values()
        .map(|tunnel| tunnel.remote_port)
        .filter(|port| *port > 0)
        .collect()
}

async fn establish_connection(
    server_addr: &str,
    token: &str,
) -> Result<(Arc<TcpTransport>, String, Value), String> {
    establish_connection_with_session(server_addr, token, None, None).await
}

async fn establish_connection_with_session(
    server_addr: &str,
    token: &str,
    previous_session_id: Option<String>,
    client_id: Option<String>,
) -> Result<(Arc<TcpTransport>, String, Value), String> {
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
    let discovery = response
        .pointer("/data/server")
        .cloned()
        .unwrap_or(Value::Null);

    Ok((transport, session_id, discovery))
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
        Err(error) => return Err(format!("failed to read protocol frame: {error}")),
    }

    let length = u32::from_be_bytes(length) as usize;
    let mut payload = vec![0_u8; length];
    stream
        .read_exact(&mut payload)
        .await
        .map_err(|error| format!("failed to read protocol payload: {error}"))?;
    protocol
        .decode(&payload)
        .map(Some)
        .map_err(|error| format!("failed to decode protocol message: {error}"))
}

async fn write_protocol_message(
    stream: &mut TcpStream,
    protocol: &ProtocolManager,
    message: &Message,
) -> Result<(), String> {
    let payload = protocol
        .encode(message)
        .map_err(|error| format!("failed to encode protocol message: {error}"))?;
    let frame =
        Frame::new(payload).map_err(|error| format!("failed to create protocol frame: {error}"))?;
    let bytes = FrameEncoder::encode(&frame);
    stream
        .write_all(&bytes)
        .await
        .map_err(|error| format!("failed to write protocol frame: {error}"))?;
    stream
        .flush()
        .await
        .map_err(|error| format!("failed to flush protocol frame: {error}"))
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

fn statistics_access_logs(statistics: &Value) -> Option<&Vec<Value>> {
    statistics
        .pointer("/gateway/http/accessLogs")
        .or_else(|| statistics.pointer("/http/accessLogs"))
        .and_then(Value::as_array)
}

fn server_access_log_entry(server_id: &str, access: &Value) -> Value {
    let status = access
        .get("status")
        .or_else(|| access.get("statusCode"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let timestamp = access
        .get("timestamp")
        .and_then(Value::as_i64)
        .unwrap_or_else(|| Utc::now().timestamp_millis());
    let method = access
        .get("method")
        .and_then(Value::as_str)
        .unwrap_or("HTTP");
    let path = access.get("path").and_then(Value::as_str).unwrap_or("/");
    let host = access.get("host").and_then(Value::as_str).unwrap_or("");
    let latency_ms = access
        .get("latencyMs")
        .or_else(|| access.get("latency"))
        .and_then(Value::as_u64)
        .unwrap_or(0);
    let tunnel_id = access
        .get("tunnelId")
        .or_else(|| access.get("tunnel_id"))
        .and_then(Value::as_str);
    let level = if status >= 500 {
        "error"
    } else if status >= 400 {
        "warn"
    } else {
        "info"
    };

    // 复用服务端已有 access log，不新增业务协议；日志中心据此能定位数据面失败。
    json!({
        "level": level,
        "source": "server.http",
        "module": "data-plane",
        "message": format!("{method} {path} -> {status} ({latency_ms}ms) host={host}"),
        "timestamp": timestamp,
        "serverId": server_id,
        "tunnelId": tunnel_id,
        "method": method,
        "path": path,
        "host": host,
        "statusCode": status,
        "latencyMs": latency_ms,
        "bytesOut": access.get("bytes").and_then(Value::as_u64).unwrap_or(0),
        "scheme": access.get("scheme").and_then(Value::as_str).unwrap_or("http"),
        "tlsVersion": access.get("tlsVersion").cloned().unwrap_or(Value::Null),
        "sni": access.get("sni").cloned().unwrap_or(Value::Null),
    })
}

fn tunnel_control_payload(
    tunnel: &TunnelRecord,
    global_config: &BTreeMap<String, String>,
) -> Value {
    let performance = resolve_tunnel_performance(
        tunnel.performance_mode.as_deref(),
        tunnel.relay_workers,
        tunnel.max_connections,
        global_config,
    );
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
        "runtime": {
            "maxConnections": performance.max_connections,
            "relayWorkerWaitMs": performance.relay_worker_wait_ms,
        },
        "metadata": {
            "createdAt": tunnel.created_at,
            "updatedAt": tunnel.updated_at,
            "relayWorkers": performance.relay_workers,
            "performanceMode": tunnel.performance_mode.as_deref().unwrap_or("auto"),
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
        .filter(|tunnel| is_running_tunnel_status(&tunnel.status))
        .count() as u64;
    let current_connection = connected_server_count(inner) as u64;
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
    let connected_servers = connected_server_count(inner) as u64;
    let total_servers = inner.servers.len() as u64;
    let running_tunnel = inner
        .tunnels
        .values()
        .filter(|tunnel| is_running_tunnel_status(&tunnel.status))
        .count() as u64;
    let warning_tunnel = inner
        .tunnels
        .values()
        .filter(|tunnel| is_attention_tunnel_status(&tunnel.status))
        .count() as u64;
    let stopped_tunnel = tunnel_count
        .saturating_sub(running_tunnel)
        .saturating_sub(warning_tunnel);
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
    let has_connection_data = connected_servers > 0
        || inner.counters.connection_total > 0
        || inner.counters.auth_failure > 0;
    let has_runtime_samples = has_connection_data
        || tunnel_count > 0
        || traffic.total_bytes > 0
        || upload_speed_bps > 0.0
        || download_speed_bps > 0.0;
    json!({
        "overview": {
            "tunnelCount": tunnel_count,
            "runningTunnel": running_tunnel,
            "currentConnection": connected_servers,
            "todayTraffic": traffic.today_bytes,
            "totalTraffic": traffic.total_bytes,
            "averageRttMs": inner.counters.average_rtt_ms,
            "runtimeUptimeSeconds": inner.started_at.elapsed().as_secs(),
            "healthScore": health.get("score").and_then(Value::as_u64).unwrap_or(0)
        },
        "statistics": statistics,
        "realtimeSpeed": if has_runtime_samples {
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
                "current": connected_servers,
                "success": inner.counters.auth_success,
                "failure": inner.counters.auth_failure,
                "reconnect": inner.counters.reconnect_total
            }])
        } else {
            json!([])
        },
        "trafficTrend": if has_runtime_samples {
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
                { "label": "warning", "count": warning_tunnel },
                { "label": "stopped", "count": stopped_tunnel }
            ])
        },
        "serverStatus": if has_connection_data {
            json!([
                { "label": "online", "count": connected_servers },
                { "label": "warning", "count": 0 },
                { "label": "offline", "count": total_servers.saturating_sub(connected_servers) }
            ])
        } else {
            json!([])
        },
        "systemHealth": health,
        "tunnels": inner.tunnels.values().map(|tunnel| {
            let server = tunnel
                .server_id
                .as_deref()
                .and_then(|server_id| inner.servers.get(server_id))
                .or_else(|| {
                    inner
                        .active_server_id
                        .as_deref()
                        .and_then(|server_id| inner.servers.get(server_id))
                });
            json!({
            "id": &tunnel.id,
            "name": &tunnel.name,
            "serverId": &tunnel.server_id,
            "protocol": &tunnel.protocol,
            "status": &tunnel.status,
            "localHost": &tunnel.local_host,
            "localPort": tunnel.local_port,
            "remotePort": tunnel.remote_port,
            "host": &tunnel.host,
            "publicHost": server.map(|server| server.host.as_str()),
            "publicAddress": tunnel_public_address(
                tunnel,
                server.map(|server| server.host.as_str())
            ),
            "serverName": server.map(|server| server.name.as_str()).unwrap_or(""),
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
            })
        }).collect::<Vec<_>>(),
        "recentActivity": inner.logs.iter().rev().take(8).map(|log| json!({
            "id": format!("{}-{}", log.source, log.timestamp),
            "title": &log.message,
            "category": &log.source,
            "timestamp": log.timestamp
        })).collect::<Vec<_>>(),
        "visualSummary": dashboard_visual_summary(
            inner,
            now,
            tunnel_count,
            running_tunnel,
            warning_tunnel,
            stopped_tunnel
        ),
        "generatedAt": now
    })
}

fn dashboard_visual_summary(
    inner: &RuntimeInner,
    now: i64,
    tunnel_count: u64,
    running_tunnel: u64,
    warning_tunnel: u64,
    stopped_tunnel: u64,
) -> Value {
    let (request_buckets, error_buckets, request_total, error_total) =
        dashboard_request_summary(inner, now);

    json!({
        "metricCards": [
            { "key": "totalTunnels", "icon": "router", "tone": "primary" },
            { "key": "onlineTunnels", "icon": "check-circle", "tone": "success" },
            { "key": "activeConnections", "icon": "users", "tone": "secondary" },
            { "key": "traffic", "icon": "activity", "tone": "info" },
            { "key": "latency", "icon": "clock", "tone": "warning" },
            { "key": "runtimeUptime", "icon": "shield-check", "tone": "healthy" }
        ],
        "tunnelState": {
            "running": running_tunnel,
            "warning": warning_tunnel,
            "stopped": stopped_tunnel,
            "runningRate": dashboard_percent(running_tunnel, tunnel_count),
            "warningRate": dashboard_percent(warning_tunnel, tunnel_count),
            "stoppedRate": dashboard_percent(stopped_tunnel, tunnel_count)
        },
        "protocolDistribution": dashboard_protocol_distribution(inner, tunnel_count),
        "requestBuckets": request_buckets,
        "errorBuckets": error_buckets,
        "requestTotal": request_total,
        "errorTotal": error_total
    })
}

fn dashboard_protocol_distribution(inner: &RuntimeInner, tunnel_count: u64) -> Value {
    let protocol_order = ["tcp", "http", "https", "udp"];
    let mut counts = BTreeMap::<String, u64>::new();
    for tunnel in inner.tunnels.values() {
        *counts.entry(tunnel.protocol.clone()).or_insert(0) += 1;
    }

    let mut rows = protocol_order
        .iter()
        .map(|protocol| {
            let count = counts.get(*protocol).copied().unwrap_or(0);
            json!({
                "protocol": protocol,
                "count": count,
                "percent": dashboard_percent(count, tunnel_count)
            })
        })
        .collect::<Vec<_>>();

    for (protocol, count) in counts {
        if !protocol_order.contains(&protocol.as_str()) && count > 0 {
            rows.push(json!({
                "protocol": protocol,
                "count": count,
                "percent": dashboard_percent(count, tunnel_count)
            }));
        }
    }

    json!(rows)
}

fn dashboard_request_summary(inner: &RuntimeInner, now: i64) -> (Vec<u64>, Vec<u64>, u64, u64) {
    let request_total = inner
        .tunnels
        .values()
        .filter(|tunnel| tunnel.protocol == "http" || tunnel.protocol == "https")
        .map(|tunnel| tunnel.request_count)
        .sum::<u64>();
    let aggregate_error_total = inner
        .tunnels
        .values()
        .filter(|tunnel| tunnel.protocol == "http" || tunnel.protocol == "https")
        .map(|tunnel| tunnel.request_count.saturating_sub(tunnel.success_count))
        .sum::<u64>();
    let recent_error_total = inner
        .tunnels
        .values()
        .filter(|tunnel| tunnel.protocol == "http" || tunnel.protocol == "https")
        .flat_map(|tunnel| tunnel.recent_requests.iter())
        .filter(|request| request.status >= 400)
        .count() as u64;

    (
        dashboard_request_buckets(inner, now, false),
        dashboard_request_buckets(inner, now, true),
        request_total,
        aggregate_error_total.max(recent_error_total),
    )
}

fn dashboard_request_buckets(inner: &RuntimeInner, now: i64, errors_only: bool) -> Vec<u64> {
    const BUCKET_COUNT: usize = 18;
    const WINDOW_MS: i64 = 24 * 60 * 60 * 1000;

    let bucket_ms = WINDOW_MS / BUCKET_COUNT as i64;
    let mut buckets = vec![0_u64; BUCKET_COUNT];

    for request in inner
        .tunnels
        .values()
        .filter(|tunnel| tunnel.protocol == "http" || tunnel.protocol == "https")
        .flat_map(|tunnel| tunnel.recent_requests.iter())
    {
        if errors_only && request.status < 400 {
            continue;
        }

        let age = now - request.timestamp;
        if age < 0 || age > WINDOW_MS {
            continue;
        }

        let source_index = (age / bucket_ms).clamp(0, BUCKET_COUNT as i64 - 1) as usize;
        let bucket_index = BUCKET_COUNT - 1 - source_index;
        buckets[bucket_index] = buckets[bucket_index].saturating_add(1);
    }

    buckets
}

fn dashboard_percent(value: u64, total: u64) -> f64 {
    if total == 0 {
        0.0
    } else {
        (value as f64 / total as f64) * 100.0
    }
}

fn apply_worker_health_to_dashboard(
    dashboard: &mut Value,
    worker_statuses: &BTreeMap<String, &'static str>,
) {
    let Some(tunnels) = dashboard.get_mut("tunnels").and_then(Value::as_array_mut) else {
        return;
    };

    for tunnel in tunnels {
        let Some(tunnel_id) = tunnel.get("id").and_then(Value::as_str) else {
            continue;
        };
        let Some(status) = worker_statuses.get(tunnel_id) else {
            continue;
        };
        // 仅覆盖本地标记为运行中的隧道，停止中的配置不应因旧 worker 快照被重新点亮。
        if matches!(
            tunnel.get("status").and_then(Value::as_str),
            Some("running" | "recovering")
        ) {
            tunnel["status"] = Value::String((*status).to_string());
        }
    }
}

fn is_attention_tunnel_status(status: &str) -> bool {
    !is_running_tunnel_status(status) && !matches!(status, "stopped")
}

fn is_running_tunnel_status(status: &str) -> bool {
    matches!(status, "running" | "starting" | "restarting" | "recovering")
}

fn is_public_tunnel_port_available(port: u16) -> bool {
    port > 1023 || matches!(port, 80 | 443)
}

fn allows_shared_public_port(protocol: &str, port: u16) -> bool {
    matches!(protocol, "http" | "https") && matches!(port, 80 | 443)
}

fn public_port_available_for_tunnel(port: u16, occupied: &BTreeSet<u16>) -> bool {
    if !is_public_tunnel_port_available(port) {
        return false;
    }
    if !occupied.contains(&port) {
        return true;
    }
    matches!(port, 80 | 443)
}

fn tunnel_public_address(tunnel: &TunnelRecord, public_host: Option<&str>) -> String {
    // 前端只展示真实公网入口；本地监听地址单独放在“本地”字段中，避免误导用户访问 127.0.0.1:公网端口。
    let path = tunnel.path.as_deref().unwrap_or("/");
    let path = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{path}")
    };

    if matches!(tunnel.protocol.as_str(), "http" | "https") {
        if let Some(host) = tunnel
            .host
            .as_deref()
            .map(str::trim)
            .filter(|host| !host.is_empty())
        {
            let port = if tunnel.remote_port > 0 {
                tunnel.remote_port
            } else {
                default_public_port(&tunnel.protocol)
            };
            if should_omit_public_port(&tunnel.protocol, port) {
                return format!("{}://{}{}", tunnel.protocol, host, path);
            }
            return format!("{}://{}:{}{}", tunnel.protocol, host, port, path);
        }

        if let Some(host) = public_host.map(str::trim).filter(|host| !host.is_empty()) {
            return format!(
                "{}://{}:{}{}",
                tunnel.protocol, host, tunnel.remote_port, path
            );
        }
    }

    if let Some(host) = public_host.map(str::trim).filter(|host| !host.is_empty()) {
        return format!("{}:{}", host, tunnel.remote_port);
    }

    if tunnel.remote_port > 0 {
        return format!(":{}", tunnel.remote_port);
    }

    String::new()
}

fn health_json(inner: &RuntimeInner) -> Value {
    let now = Utc::now().timestamp_millis();
    let system = system_snapshot();
    let connection_status = if inner.connected {
        "healthy"
    } else {
        "offline"
    };
    let system_pressure = system.cpu_usage.max(system.memory_usage);
    let system_status = if system_pressure >= 90.0 {
        "warning"
    } else {
        "healthy"
    };
    let overall_status = if !inner.connected {
        "offline"
    } else if system_status == "warning" {
        "warning"
    } else {
        "healthy"
    };
    let connection_score = if inner.connected { 100_u64 } else { 0 };
    let heartbeat_score = if inner.connected || inner.counters.heartbeat_pong > 0 {
        100_u64
    } else {
        0
    };
    let runtime_score = if inner.connected { 100_u64 } else { 0 };
    let system_score = if system_pressure >= 95.0 {
        70_u64
    } else if system_pressure >= 90.0 {
        85_u64
    } else {
        100_u64
    };
    // 评分只按真实状态阈值变化，避免 CPU/内存采样抖动导致健康分每秒跳动。
    let score = (connection_score + heartbeat_score + runtime_score + system_score + 2) / 4;

    json!({
        "overall": overall_status,
        "score": score,
        "signals": [
            {
                "target": "connection",
                "status": connection_status,
                "message": if inner.connected { "TCP connection is authenticated" } else { "TCP connection is offline" },
                "score": connection_score,
                "timestamp": now
            },
            {
                "target": "heartbeat",
                "status": if inner.connected || inner.counters.heartbeat_pong > 0 { "healthy" } else { "offline" },
                "message": if inner.counters.heartbeat_pong > 0 { "Heartbeat completed at least once" } else if inner.connected { "Heartbeat sample is pending" } else { "Heartbeat has not completed yet" },
                "score": heartbeat_score,
                "timestamp": now
            },
            {
                "target": "runtime",
                "status": connection_status,
                "message": "Client runtime state is available",
                "score": runtime_score,
                "timestamp": now
            },
            {
                "target": "system",
                "status": system_status,
                "message": format!("CPU {:.0}% / Memory {:.0}%", system.cpu_usage, system.memory_usage),
                "score": system_score,
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

fn bytes_per_second(current: u64, previous: u64, elapsed_seconds: f64) -> f64 {
    if elapsed_seconds <= 0.0 {
        return 0.0;
    }

    // 使用累计字节差值计算真实瞬时速度，远程转发快照也能展示非 0B/s。
    current.saturating_sub(previous) as f64 / elapsed_seconds
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
    let session_bytes = upload_bytes.saturating_add(download_bytes);
    let today_bytes = if is_current_traffic_day(&inner.counters.today_traffic_date) {
        inner.counters.today_traffic_bytes
    } else {
        0
    };
    let total_bytes = inner.counters.total_traffic_bytes.max(session_bytes);

    RuntimeTraffic {
        upload_bytes,
        download_bytes,
        today_bytes,
        total_bytes,
    }
}

fn current_traffic_date() -> String {
    Utc::now().format("%Y-%m-%d").to_string()
}

fn is_current_traffic_day(date: &str) -> bool {
    date == current_traffic_date()
}

fn ensure_traffic_day(counters: &mut RuntimeCounters) {
    let today = current_traffic_date();
    if counters.today_traffic_date != today {
        counters.today_traffic_date = today;
        counters.today_traffic_bytes = 0;
    }
}

fn bootstrap_traffic_counters(inner: &mut RuntimeInner) {
    if !inner.counters.today_traffic_date.is_empty() {
        return;
    }

    let today = current_traffic_date();
    let session_bytes = inner
        .tunnels
        .values()
        .map(tunnel_traffic_bytes)
        .sum::<u64>();
    inner.counters.today_traffic_date = today;
    inner.counters.today_traffic_bytes = session_bytes;
    inner.counters.total_traffic_bytes = inner.counters.total_traffic_bytes.max(session_bytes);
}

fn tunnel_traffic_bytes(tunnel: &TunnelRecord) -> u64 {
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
}

fn snapshot_traffic_bytes(snapshot: &LocalTunnelRuntimeSnapshot) -> u64 {
    snapshot
        .upload_bytes
        .saturating_add(snapshot.download_bytes)
        .saturating_add(
            snapshot
                .recent_requests
                .iter()
                .map(|request| request.traffic_bytes)
                .sum::<u64>(),
        )
}

fn traffic_bytes_delta(previous_bytes: u64, next_bytes: u64) -> u64 {
    if next_bytes >= previous_bytes {
        next_bytes.saturating_sub(previous_bytes)
    } else {
        // 隧道重启后 relay 计数归零，只累计新会话产生的流量。
        next_bytes
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
    async fn worker_registry_updates_session_without_restarting_worker() {
        let registry = RelayWorkerRegistry::default();
        registry
            .register(
                "tunnel-1:0".to_string(),
                "tunnel-1".to_string(),
                "session-old".to_string(),
            )
            .await;
        registry
            .update_tunnel_session("tunnel-1", "session-new")
            .await;

        let snapshot = registry.snapshot().await;
        assert_eq!(snapshot["items"][0]["sessionId"], "session-new");
        assert_eq!(snapshot["items"][0]["state"], "recovering");
    }

    #[test]
    fn traffic_bytes_delta_handles_session_restart() {
        assert_eq!(traffic_bytes_delta(31_000_000, 0), 0);
        assert_eq!(traffic_bytes_delta(0, 5_000_000), 5_000_000);
        assert_eq!(traffic_bytes_delta(5_000_000, 10_000_000), 5_000_000);
        assert_eq!(traffic_bytes_delta(31_000_000, 3_000_000), 3_000_000);
    }

    #[test]
    fn worker_supervisor_keeps_a_bounded_warm_pool() {
        let pool = RelayWorkerPoolConfig {
            minimum: 8,
            maximum: 32,
        };
        assert_eq!(relay_pool_target(pool), 8);

        let clamped = RelayWorkerPoolConfig {
            minimum: 64,
            maximum: 32,
        };
        assert_eq!(relay_pool_target(clamped), 32);
    }

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
                token: "gate-integration-test-token-20260710-release-audit".to_string(),
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
                discovery: Value::Null,
                created_at: now,
                updated_at: now,
            },
        );
        inner.active_server_id = Some(server_id.clone());
        inner.session_id = Some(previous_session_id.clone());
        inner.connected = false;

        let mut tunnel =
            test_tunnel_record("tcp", target_addr.port(), unused_loopback_port().await?);
        tunnel.server_id = Some(server_id);
        tunnel.status = "running".to_string();
        tunnel.started_at = Some(now);
        inner.tunnels.insert(tunnel.id.clone(), tunnel);

        let runtime = ClientRuntimeState {
            storage_path,
            domain_repository: None,
            server_connections: Mutex::new(BTreeMap::new()),
            inner: Mutex::new(inner),
            local_tunnels: Mutex::new(BTreeMap::new()),
            tunnel_start_lock: Mutex::new(()),
            worker_registry: Arc::new(RelayWorkerRegistry::default()),
            control_io_lock: Mutex::new(()),
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
            server_id: None,
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
            performance_mode: None,
            relay_workers: None,
            max_connections: None,
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
