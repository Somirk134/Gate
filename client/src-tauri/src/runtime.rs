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
struct TunnelRecord {
    id: String,
    name: String,
    protocol: String,
    status: String,
    upload_speed_bps: f64,
    download_speed_bps: f64,
    connections: u64,
    uptime_seconds: u64,
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

        let response = send_request(&transport, Command::AuthLogin, json!({ "token": token })).await?;
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
            .request(Command::HeartbeatPing, json!({ "sentAt": Utc::now().timestamp_millis() }))
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
    ) -> Result<String, String> {
        let tunnel_id = Uuid::new_v4().to_string();
        let _ = self
            .request(
                Command::TunnelCreate,
                json!({
                    "id": tunnel_id,
                    "localPort": local_port,
                    "remotePort": remote_port,
                    "protocol": protocol.clone()
                }),
            )
            .await;

        let mut inner = self.inner.lock().await;
        inner.tunnels.insert(
            tunnel_id.clone(),
            TunnelRecord {
                id: tunnel_id.clone(),
                name: format!("{}-{}", protocol, local_port),
                protocol,
                status: "running".to_string(),
                upload_speed_bps: 0.0,
                download_speed_bps: 0.0,
                connections: 0,
                uptime_seconds: 0,
            },
        );
        inner.log("info", "tunnel", "Tunnel created");
        Ok(tunnel_id)
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
        let inner = self.inner.lock().await;
        statistics_json(&inner)
    }

    pub async fn dashboard(&self) -> Value {
        let inner = self.inner.lock().await;
        dashboard_json(&inner)
    }

    pub async fn health(&self) -> Value {
        let inner = self.inner.lock().await;
        health_json(&inner)
    }

    pub async fn metrics(&self) -> Value {
        let inner = self.inner.lock().await;
        let now = Utc::now().timestamp_millis();
        json!([
            metric("gate.connection.current", "Current connections", "gauge", "connection", "count", if inner.connected { 1.0 } else { 0.0 }, now),
            metric("gate.connection.rtt.average", "Average RTT", "gauge", "heartbeat", "milliseconds", inner.counters.average_rtt_ms, now),
            metric("gate.tunnel.count", "Tunnel count", "gauge", "tunnel", "count", inner.tunnels.len() as f64, now)
        ])
    }

    pub async fn logs(&self) -> Value {
        let inner = self.inner.lock().await;
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
            "uploadSpeedBps": tunnel.upload_speed_bps,
            "downloadSpeedBps": tunnel.download_speed_bps,
            "connections": tunnel.connections,
            "uptimeSeconds": tunnel.uptime_seconds
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
    let status = if inner.connected { "healthy" } else { "offline" };
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
