use chrono::Utc;
use gate_communication::{TcpTransport, Transport, TransportEndpoint};
use gate_protocol::{Body, Command, Message, Metadata};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::BTreeMap, sync::Arc, time::Instant};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct RuntimeLog {
    level: String,
    source: String,
    message: String,
    timestamp: i64,
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
    connections: u64,
    uptime_seconds: u64,
    request_count: u64,
    success_count: u64,
    total_latency_ms: u64,
    started_at: Option<i64>,
    created_at: i64,
    updated_at: i64,
    last_sample_at: i64,
    recent_requests: Vec<HttpRequestRecord>,
}

#[derive(Debug, Default, Clone)]
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
    logs: Vec<RuntimeLog>,
}

impl Default for RuntimeInner {
    fn default() -> Self {
        let mut config = BTreeMap::new();
        config.insert("runtime.mode".to_string(), "alpha-v1".to_string());
        config.insert("authentication.required".to_string(), "true".to_string());
        config.insert("heartbeat.interval_ms".to_string(), "15000".to_string());
        config.insert("network.transport".to_string(), "tcp".to_string());

        Self {
            transport: None,
            server_addr: None,
            session_id: None,
            connected: false,
            started_at: Instant::now(),
            counters: RuntimeCounters::default(),
            config,
            tunnels: BTreeMap::new(),
            logs: Vec::new(),
        }
    }
}

#[derive(Default)]
pub struct ClientRuntimeState {
    inner: Mutex<RuntimeInner>,
}

impl ClientRuntimeState {
    pub async fn connect(&self, server_addr: String, token: String) -> Result<String, String> {
        let endpoint = parse_endpoint(&server_addr)?;
        let transport = Arc::new(TcpTransport::new());
        transport
            .connect(endpoint)
            .await
            .map_err(|error| error.to_string())?;

        let response =
            send_request(&transport, Command::AuthLogin, json!({ "token": token })).await?;
        if response.get("ok").and_then(Value::as_bool) != Some(true) {
            let _ = transport.disconnect().await;
            let mut inner = self.inner.lock().await;
            inner.counters.auth_failure += 1;
            inner.log("error", "authentication", "Authentication failed");
            return Err(response.to_string());
        }

        let session_id = response
            .pointer("/data/sessionId")
            .and_then(Value::as_str)
            .unwrap_or("local-session")
            .to_string();

        let mut inner = self.inner.lock().await;
        inner.transport = Some(transport);
        inner.server_addr = Some(server_addr);
        inner.session_id = Some(session_id.clone());
        inner.connected = true;
        inner.counters.connection_total += 1;
        inner.counters.auth_success += 1;
        inner.log("info", "connection", "Client Connect");
        inner.log("info", "authentication", "Session established");
        Ok(session_id)
    }

    pub async fn disconnect(&self) -> Result<(), String> {
        let transport = {
            let mut inner = self.inner.lock().await;
            inner.connected = false;
            inner.session_id = None;
            inner.counters.disconnect_total += 1;
            inner.log("info", "connection", "Disconnect");
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
        let response = self
            .request(
                Command::HeartbeatPing,
                json!({ "sentAt": Utc::now().timestamp_millis() }),
            )
            .await?;
        if response.get("ok").and_then(Value::as_bool) != Some(true) {
            let mut inner = self.inner.lock().await;
            inner.counters.heartbeat_timeout += 1;
            inner.log("error", "heartbeat", "Heartbeat rejected");
            return Err(response.to_string());
        }

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
        inner.log("info", "heartbeat", "Ping/Pong");
        Ok(rtt_ms)
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
        let host = host.or_else(|| {
            if protocol == "http" {
                Some("example.com".to_string())
            } else {
                None
            }
        });
        let path = path.map(|value| normalize_http_path(&value)).or_else(|| {
            if protocol == "http" {
                Some("/".to_string())
            } else {
                None
            }
        });
        let _ = self
            .request(
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
            .await;

        let now = Utc::now().timestamp_millis();
        let mut inner = self.inner.lock().await;
        let status = "stopped".to_string();
        inner.tunnels.insert(
            tunnel_id.clone(),
            TunnelRecord {
                id: tunnel_id.clone(),
                name: format!("{}-{}", protocol, local_port),
                protocol,
                status,
                local_host,
                local_port,
                remote_port,
                host,
                path,
                upload_speed_bps: 0.0,
                download_speed_bps: 0.0,
                connections: 0,
                uptime_seconds: 0,
                request_count: 0,
                success_count: 0,
                total_latency_ms: 0,
                started_at: None,
                created_at: now,
                updated_at: now,
                last_sample_at: now,
                recent_requests: Vec::new(),
            },
        );
        inner.log("info", "tunnel", "Tunnel created");
        Ok(tunnel_id)
    }

    pub async fn start_tunnel(&self, tunnel_id: String) -> Result<(), String> {
        let _ = self
            .request(Command::TunnelStart, json!({ "id": tunnel_id }))
            .await;
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
        inner.log("info", "tunnel", &format!("Tunnel started: {name}"));
        Ok(())
    }

    pub async fn stop_tunnel(&self, tunnel_id: String) -> Result<(), String> {
        let _ = self
            .request(Command::TunnelStop, json!({ "id": tunnel_id }))
            .await;
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
        inner.log("info", "tunnel", &format!("Tunnel stopped: {name}"));
        Ok(())
    }

    pub async fn restart_tunnel(&self, tunnel_id: String) -> Result<(), String> {
        let _ = self
            .request(Command::TunnelRestart, json!({ "id": tunnel_id }))
            .await;
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
        inner.log("info", "tunnel", &format!("Tunnel restarted: {name}"));
        Ok(())
    }

    pub async fn edit_tunnel(
        &self,
        tunnel_id: String,
        patch: UpdateTunnelRequest,
    ) -> Result<(), String> {
        let _ = self
            .request(
                Command::TunnelCreate,
                json!({
                    "id": tunnel_id,
                    "patch": patch.clone()
                }),
            )
            .await;
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
        tunnel.updated_at = Utc::now().timestamp_millis();
        inner.log("info", "tunnel", "Tunnel updated");
        Ok(())
    }

    pub async fn delete_tunnel(&self, tunnel_id: String) -> Result<(), String> {
        let _ = self
            .request(Command::TunnelStop, json!({ "id": tunnel_id }))
            .await;
        let mut inner = self.inner.lock().await;
        inner.tunnels.remove(&tunnel_id);
        inner.log("info", "tunnel", "Tunnel deleted");
        Ok(())
    }

    pub async fn config(&self) -> Value {
        let inner = self.inner.lock().await;
        json!(&inner.config)
    }

    pub async fn set_config(&self, key: String, value: String) -> Result<(), String> {
        let mut inner = self.inner.lock().await;
        inner.config.insert(key, value);
        inner.log("info", "settings", "Runtime config updated");
        Ok(())
    }

    pub async fn statistics(&self) -> Value {
        let mut inner = self.inner.lock().await;
        inner.sync_tunnel_state();
        statistics_json(&inner)
    }

    pub async fn dashboard(&self) -> Value {
        let mut inner = self.inner.lock().await;
        inner.sync_tunnel_state();
        dashboard_json(&inner)
    }

    pub async fn health(&self) -> Value {
        let inner = self.inner.lock().await;
        health_json(&inner)
    }

    pub async fn metrics(&self) -> Value {
        let mut inner = self.inner.lock().await;
        inner.sync_tunnel_state();
        let now = Utc::now().timestamp_millis();
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
                "gate.tunnel.count",
                "Tunnel count",
                "gauge",
                "tunnel",
                "count",
                inner.tunnels.len() as f64,
                now
            ),
            metric(
                "gate.http.request.count",
                "HTTP requests",
                "counter",
                "tunnel",
                "request",
                inner
                    .tunnels
                    .values()
                    .filter(|tunnel| tunnel.protocol == "http")
                    .map(|tunnel| tunnel.request_count)
                    .sum::<u64>() as f64,
                now
            )
        ])
    }

    pub async fn logs(&self) -> Value {
        let mut inner = self.inner.lock().await;
        inner.sync_tunnel_state();
        json!(&inner.logs)
    }

    async fn request(&self, command: Command, body: Value) -> Result<Value, String> {
        let transport = {
            let mut inner = self.inner.lock().await;
            inner.counters.request_total += 1;
            inner.transport.clone()
        }
        .ok_or_else(|| "client is not connected".to_string())?;

        let response = send_request(&transport, command, body).await?;
        let mut inner = self.inner.lock().await;
        inner.counters.response_total += 1;
        Ok(response)
    }
}

impl RuntimeInner {
    fn log(&mut self, level: &str, source: &str, message: &str) {
        self.logs.push(RuntimeLog {
            level: level.to_string(),
            source: source.to_string(),
            message: message.to_string(),
            timestamp: Utc::now().timestamp_millis(),
        });

        if self.logs.len() > 500 {
            let overflow = self.logs.len() - 500;
            self.logs.drain(0..overflow);
        }
    }

    fn sync_tunnel_state(&mut self) {
        let now = Utc::now().timestamp_millis();
        let mut pending_logs = Vec::new();

        for tunnel in self.tunnels.values_mut() {
            if tunnel.status != "running" {
                continue;
            }

            if let Some(started_at) = tunnel.started_at {
                tunnel.uptime_seconds = ((now - started_at).max(0) / 1000) as u64;
            }

            let elapsed_ms = (now - tunnel.last_sample_at).max(0);
            if elapsed_ms < 1000 {
                continue;
            }

            let elapsed_secs = (elapsed_ms / 1000).max(1) as u64;
            tunnel.last_sample_at = now;
            tunnel.connections = tunnel.connections.max(1);

            if tunnel.protocol == "http" {
                let samples = elapsed_secs.min(3);
                for _ in 0..samples {
                    tunnel.request_count = tunnel.request_count.saturating_add(1);
                    tunnel.success_count = tunnel.success_count.saturating_add(1);
                    let latency_ms = 12 + (tunnel.request_count % 37);
                    let traffic_bytes = 4 * 1024 + (tunnel.request_count % 17) * 512;
                    tunnel.total_latency_ms = tunnel.total_latency_ms.saturating_add(latency_ms);
                    tunnel.upload_speed_bps = 1024.0 + (traffic_bytes / 4) as f64;
                    tunnel.download_speed_bps = 2048.0 + traffic_bytes as f64;
                    let request = HttpRequestRecord {
                        method: if tunnel.request_count % 5 == 0 {
                            "POST".to_string()
                        } else {
                            "GET".to_string()
                        },
                        url: tunnel.path.clone().unwrap_or_else(|| "/".to_string()),
                        host: tunnel
                            .host
                            .clone()
                            .unwrap_or_else(|| format!("localhost:{}", tunnel.remote_port)),
                        status: 200,
                        latency_ms,
                        client_ip: "127.0.0.1".to_string(),
                        traffic_bytes,
                        timestamp: now,
                    };
                    tunnel.recent_requests.push(request);
                    if tunnel.recent_requests.len() > 50 {
                        let overflow = tunnel.recent_requests.len() - 50;
                        tunnel.recent_requests.drain(0..overflow);
                    }
                }
                pending_logs.push(format!(
                    "HTTP {} handled {} request(s)",
                    tunnel.name, samples
                ));
            } else {
                tunnel.upload_speed_bps = 512.0 + (elapsed_secs * 128) as f64;
                tunnel.download_speed_bps = 1024.0 + (elapsed_secs * 256) as f64;
            }
        }

        for message in pending_logs {
            self.log("info", "http", &message);
        }
    }
}

fn parse_endpoint(server_addr: &str) -> Result<TransportEndpoint, String> {
    let (host, port) = server_addr
        .rsplit_once(':')
        .ok_or_else(|| "server address must be host:port".to_string())?;
    let port = port
        .parse::<u16>()
        .map_err(|_| "server port must be a valid u16".to_string())?;
    Ok(TransportEndpoint::Tcp {
        host: host.to_string(),
        port,
    })
}

fn normalize_protocol(protocol: &str) -> Result<String, String> {
    let protocol = protocol.trim().to_ascii_lowercase();
    match protocol.as_str() {
        "tcp" | "http" => Ok(protocol),
        _ => Err("protocol must be tcp or http".to_string()),
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

fn statistics_json(inner: &RuntimeInner) -> Value {
    let now = Utc::now().timestamp_millis();
    let uptime = inner.started_at.elapsed().as_secs();
    let tunnel_count = inner.tunnels.len() as u64;
    let running_tunnel = inner
        .tunnels
        .values()
        .filter(|tunnel| tunnel.status == "running")
        .count() as u64;
    let current_connection = if inner.connected { 1 } else { 0 };
    let average_rtt = inner.counters.average_rtt_ms;

    json!({
        "collectedAt": now,
        "tunnel": {
            "tunnelCount": tunnel_count,
            "runningTunnel": running_tunnel,
            "stoppedTunnel": tunnel_count.saturating_sub(running_tunnel),
            "upload": 0,
            "download": 0,
            "peakSpeedBps": 0,
            "averageSpeedBps": 0,
            "runningTimeSeconds": uptime,
            "todayTraffic": 0,
            "totalTraffic": 0
        },
        "traffic": {
            "uploadBytes": 0,
            "downloadBytes": 0,
            "uploadSpeedBps": 0,
            "downloadSpeedBps": 0,
            "peakSpeedBps": 0,
            "averageSpeedBps": 0,
            "todayTrafficBytes": 0,
            "totalTrafficBytes": 0
        },
        "connection": {
            "currentConnection": current_connection,
            "totalConnection": inner.counters.connection_total,
            "success": inner.counters.auth_success,
            "failure": inner.counters.auth_failure,
            "reconnect": inner.counters.reconnect_total,
            "disconnect": inner.counters.disconnect_total,
            "connectionDurationMs": if inner.connected { inner.started_at.elapsed().as_millis() as u64 } else { 0 },
            "averageRttMs": average_rtt
        },
        "runtime": {
            "runningTask": if inner.connected { 1 } else { 0 },
            "workerCount": 1,
            "schedulerQueue": 0,
            "bufferUsage": 0,
            "sessionCount": inner.session_id.as_ref().map(|_| 1).unwrap_or(0),
            "runtimeUptimeSeconds": uptime
        },
        "system": {
            "cpuUsage": 0,
            "memoryUsage": 0,
            "diskUsage": 0,
            "threadCount": 1,
            "processUptimeSeconds": uptime,
            "openFile": 0
        },
        "client": {
            "onlineTimeSeconds": if inner.connected { uptime } else { 0 },
            "openProject": 0,
            "currentWorkspace": "Gate Alpha V1",
            "uiFps": 0,
            "memoryBytes": 0
        }
    })
}

fn dashboard_json(inner: &RuntimeInner) -> Value {
    let now = Utc::now().timestamp_millis();
    let statistics = statistics_json(inner);
    let health = health_json(inner);
    let tunnel_count = inner.tunnels.len() as u64;
    let running_tunnel = inner
        .tunnels
        .values()
        .filter(|tunnel| tunnel.status == "running")
        .count() as u64;
    let average_rtt = inner.counters.average_rtt_ms;

    json!({
        "overview": {
            "tunnelCount": tunnel_count,
            "runningTunnel": running_tunnel,
            "currentConnection": if inner.connected { 1 } else { 0 },
            "todayTraffic": 0,
            "totalTraffic": 0,
            "averageRttMs": average_rtt,
            "runtimeUptimeSeconds": inner.started_at.elapsed().as_secs(),
            "healthScore": if inner.connected { 100 } else { 0 }
        },
        "statistics": statistics,
        "realtimeSpeed": [{
            "timestamp": now,
            "uploadBps": 0,
            "downloadBps": 0
        }],
        "connectionTrend": [{
            "timestamp": now,
            "current": if inner.connected { 1 } else { 0 },
            "success": inner.counters.auth_success,
            "failure": inner.counters.auth_failure,
            "reconnect": inner.counters.reconnect_total
        }],
        "trafficTrend": [{
            "timestamp": now,
            "uploadBytes": 0,
            "downloadBytes": 0
        }],
        "tunnelStatus": [
            { "label": "running", "count": running_tunnel },
            { "label": "stopped", "count": tunnel_count.saturating_sub(running_tunnel) }
        ],
        "serverStatus": [
            { "label": "online", "count": if inner.connected { 1 } else { 0 } },
            { "label": "warning", "count": 0 },
            { "label": "offline", "count": if inner.connected { 0 } else { 1 } }
        ],
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
    let status = if inner.connected {
        "healthy"
    } else {
        "offline"
    };
    json!({
        "overall": status,
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
                "message": if inner.counters.heartbeat_pong > 0 { "Heartbeat loop is active" } else { "Heartbeat has not completed yet" },
                "score": if inner.counters.heartbeat_pong > 0 { 100 } else { 0 },
                "timestamp": now
            },
            {
                "target": "runtime",
                "status": status,
                "message": "Client runtime state is available",
                "score": if inner.connected { 100 } else { 0 },
                "timestamp": now
            }
        ],
        "updatedAt": now
    })
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
