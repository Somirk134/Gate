use chrono::Utc;
use gate_protocol::{
    Body, Command, Frame, FrameEncoder, Message, MessageType, Metadata, ProtocolBuilder,
    ProtocolManager,
};
use serde_json::{json, Value};
use std::{
    collections::{HashMap, VecDeque},
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::Mutex,
    task::JoinHandle,
};
use tracing::{info, warn};
use uuid::Uuid;

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
    counters: GatewayCounters,
}

#[derive(Debug, Default)]
struct GatewayCounters {
    registered_tunnels: AtomicU64,
    active_connections: AtomicU64,
    total_connections: AtomicU64,
    failed_connections: AtomicU64,
}

#[derive(Debug, Clone)]
struct ClientSession {
    connected: bool,
    last_seen_at: i64,
}

#[derive(Debug)]
struct TunnelSession {
    config: Mutex<TunnelConfig>,
    relay_workers: Mutex<VecDeque<TcpStream>>,
    active_connections: AtomicU64,
    total_connections: AtomicU64,
    failed_connections: AtomicU64,
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
}

#[derive(Debug)]
struct ListenerHandle {
    tunnel_id: String,
    handle: JoinHandle<()>,
}

impl TunnelGateway {
    pub fn new() -> Self {
        let bind_ip = std::env::var("GATE_TUNNEL_BIND_ADDR")
            .ok()
            .and_then(|value| value.parse::<IpAddr>().ok())
            .unwrap_or(IpAddr::V4(Ipv4Addr::UNSPECIFIED));

        Self {
            inner: Arc::new(GatewayInner {
                bind_ip,
                sessions: Mutex::new(HashMap::new()),
                tunnels: Mutex::new(HashMap::new()),
                listeners: Mutex::new(HashMap::new()),
                counters: GatewayCounters::default(),
            }),
        }
    }

    pub async fn create_session(&self) -> String {
        let session_id = Uuid::new_v4().to_string();
        let session = ClientSession {
            connected: true,
            last_seen_at: Utc::now().timestamp_millis(),
        };
        self.inner
            .sessions
            .lock()
            .await
            .insert(session_id.clone(), session);
        session_id
    }

    pub async fn touch_session(&self, session_id: &str) {
        if let Some(session) = self.inner.sessions.lock().await.get_mut(session_id) {
            session.connected = true;
            session.last_seen_at = Utc::now().timestamp_millis();
        }
    }

    pub async fn close_session(&self, session_id: &str) {
        if let Some(session) = self.inner.sessions.lock().await.get_mut(session_id) {
            session.connected = false;
            session.last_seen_at = Utc::now().timestamp_millis();
        }

        let tunnels = {
            let tunnels = self.inner.tunnels.lock().await;
            tunnels.values().cloned().collect::<Vec<_>>()
        };

        for tunnel in tunnels {
            let matches_session = tunnel.config.lock().await.session_id == session_id;
            if matches_session {
                tunnel.relay_workers.lock().await.clear();
            }
        }
    }

    pub async fn register_tunnel(&self, session_id: &str, body: &Value) -> anyhow::Result<Value> {
        self.touch_session(session_id).await;
        let config = parse_tunnel_config(session_id, body)?;
        let tunnel_id = config.tunnel_id.clone();
        let mut stale_remote_port = None;

        {
            let mut tunnels = self.inner.tunnels.lock().await;
            if let Some(existing) = tunnels.get(&tunnel_id) {
                let mut existing_config = existing.config.lock().await;
                if existing_config.remote_port != config.remote_port {
                    stale_remote_port = Some(existing_config.remote_port);
                    existing.relay_workers.lock().await.clear();
                }
                *existing_config = config.clone();
            } else {
                let session = Arc::new(TunnelSession {
                    config: Mutex::new(config.clone()),
                    relay_workers: Mutex::new(VecDeque::new()),
                    active_connections: AtomicU64::new(0),
                    total_connections: AtomicU64::new(0),
                    failed_connections: AtomicU64::new(0),
                });
                tunnels.insert(tunnel_id.clone(), session);
                self.inner
                    .counters
                    .registered_tunnels
                    .fetch_add(1, Ordering::Relaxed);
            }
        }

        if let Some(remote_port) = stale_remote_port {
            if let Some(listener) = self.inner.listeners.lock().await.remove(&remote_port) {
                listener.handle.abort();
            }
        }

        self.ensure_listener(tunnel_id.clone(), config.remote_port)
            .await?;

        Ok(json!({
            "tunnelId": tunnel_id,
            "remotePort": config.remote_port,
            "protocol": config.protocol,
            "registeredAt": config.updated_at
        }))
    }

    pub async fn stop_tunnel(&self, tunnel_id: &str) -> anyhow::Result<bool> {
        let tunnel = self.inner.tunnels.lock().await.remove(tunnel_id);
        let mut removed_listener = false;

        if let Some(tunnel) = tunnel {
            tunnel.relay_workers.lock().await.clear();
            let remote_port = tunnel.config.lock().await.remote_port;
            if let Some(listener) = self.inner.listeners.lock().await.remove(&remote_port) {
                listener.handle.abort();
                removed_listener = true;
            }
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

        Ok(())
    }

    pub async fn statistics(&self) -> Value {
        let sessions = self.inner.sessions.lock().await;
        let tunnels = self.inner.tunnels.lock().await;
        let mut tunnel_items = Vec::with_capacity(tunnels.len());

        for tunnel in tunnels.values() {
            let config = tunnel.config.lock().await.clone();
            let queued_workers = tunnel.relay_workers.lock().await.len();
            tunnel_items.push(json!({
                "tunnelId": config.tunnel_id,
                "sessionId": config.session_id,
                "protocol": config.protocol,
                "remotePort": config.remote_port,
                "localTarget": format!("{}:{}", config.local_host, config.local_port),
                "host": config.host,
                "path": config.path,
                "enabled": config.enabled,
                "relayWorkers": queued_workers,
                "activeConnections": tunnel.active_connections.load(Ordering::Relaxed),
                "totalConnections": tunnel.total_connections.load(Ordering::Relaxed),
                "failedConnections": tunnel.failed_connections.load(Ordering::Relaxed),
                "updatedAt": config.updated_at
            }));
        }

        json!({
            "sessions": sessions.len(),
            "onlineSessions": sessions.values().filter(|session| session.connected).count(),
            "tunnels": tunnel_items,
            "registeredTunnels": self.inner.counters.registered_tunnels.load(Ordering::Relaxed),
            "activeConnections": self.inner.counters.active_connections.load(Ordering::Relaxed),
            "totalConnections": self.inner.counters.total_connections.load(Ordering::Relaxed),
            "failedConnections": self.inner.counters.failed_connections.load(Ordering::Relaxed)
        })
    }

    async fn ensure_listener(&self, tunnel_id: String, remote_port: u16) -> anyhow::Result<()> {
        let mut listeners = self.inner.listeners.lock().await;
        if let Some(existing) = listeners.get(&remote_port) {
            if existing.tunnel_id == tunnel_id {
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

        listeners.insert(remote_port, ListenerHandle { tunnel_id, handle });
        info!(
            target: "gate_gateway",
            addr = %bound_addr,
            "隧道网关开始监听公网端口"
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
                                "隧道网关转发失败"
                            );
                        }
                    });
                }
                Err(source) => {
                    warn!(
                        target: "gate_gateway",
                        addr = %bound_addr,
                        error = %source,
                        "隧道网关接受连接失败"
                    );
                    break;
                }
            }
        }
    }

    async fn handle_public_connection(
        &self,
        tunnel_id: String,
        mut public_stream: TcpStream,
        remote_addr: SocketAddr,
    ) -> anyhow::Result<()> {
        let tunnel = {
            let tunnels = self.inner.tunnels.lock().await;
            tunnels
                .get(&tunnel_id)
                .cloned()
                .ok_or_else(|| anyhow::anyhow!("tunnel is not registered"))?
        };

        tunnel.active_connections.fetch_add(1, Ordering::Relaxed);
        tunnel.total_connections.fetch_add(1, Ordering::Relaxed);
        self.inner
            .counters
            .active_connections
            .fetch_add(1, Ordering::Relaxed);
        self.inner
            .counters
            .total_connections
            .fetch_add(1, Ordering::Relaxed);

        let result = async {
            let mut relay_stream = self.next_relay_worker(&tunnel).await?;
            let config = tunnel.config.lock().await.clone();
            let protocol = ProtocolBuilder::new().build();
            let start = Message::request(
                Command::TunnelRelayStart,
                Body::Json(json!({
                    "relayId": Uuid::new_v4().to_string(),
                    "tunnelId": config.tunnel_id,
                    "protocol": config.protocol,
                    "remoteAddr": remote_addr.to_string(),
                    "localHost": config.local_host,
                    "localPort": config.local_port,
                    "host": config.host,
                    "path": config.path,
                    "metadata": config.metadata
                })),
                Metadata::default(),
            );
            write_message(&mut relay_stream, &protocol, &start).await?;

            info!(
                target: "gate_gateway",
                tunnel_id = %tunnel_id,
                remote_addr = %remote_addr,
                "隧道转发开始"
            );

            let (upload_bytes, download_bytes) =
                tokio::io::copy_bidirectional(&mut public_stream, &mut relay_stream).await?;
            info!(
                target: "gate_gateway",
                tunnel_id = %tunnel_id,
                remote_addr = %remote_addr,
                upload_bytes,
                download_bytes,
                "隧道转发完成"
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
            tunnel.failed_connections.fetch_add(1, Ordering::Relaxed);
            self.inner
                .counters
                .failed_connections
                .fetch_add(1, Ordering::Relaxed);
        }

        result
    }

    async fn next_relay_worker(&self, tunnel: &Arc<TunnelSession>) -> anyhow::Result<TcpStream> {
        tunnel
            .relay_workers
            .lock()
            .await
            .pop_front()
            .ok_or_else(|| anyhow::anyhow!("no relay worker is available for tunnel"))
    }
}

impl Default for TunnelGateway {
    fn default() -> Self {
        Self::new()
    }
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
    })
}

fn json_u16(body: &Value, key: &str) -> Option<u16> {
    let value = body.get(key)?.as_u64()?;
    u16::try_from(value).ok().filter(|port| *port > 0)
}

fn optional_string(body: &Value, key: &str) -> Option<String> {
    body.get(key)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub async fn read_message(
    stream: &mut TcpStream,
    protocol: &ProtocolManager,
) -> anyhow::Result<Option<Message>> {
    let mut length = [0_u8; 4];
    match stream.read_exact(&mut length).await {
        Ok(_) => {}
        Err(source) if source.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
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

    #[tokio::test]
    async fn gateway_relays_public_connection_to_waiting_worker() -> anyhow::Result<()> {
        let gateway = TunnelGateway::new();
        let session_id = gateway.create_session().await;
        let remote_port = unused_loopback_port().await?;
        let tunnel_id = "relay-test";
        let body = json!({
            "id": tunnel_id,
            "protocol": "tcp",
            "remotePort": remote_port,
            "localHost": "127.0.0.1",
            "localPort": 18080
        });

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
        gateway.stop_tunnel(tunnel_id).await?;
        Ok(())
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
