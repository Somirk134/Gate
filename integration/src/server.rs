use anyhow::{Context, Result};
use gate_protocol::{Command, MessageType};
use serde_json::{json, Value};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
    time::Instant,
};
use tokio::{
    net::{TcpListener, TcpStream},
    sync::{watch, RwLock},
    task::JoinHandle,
};
use uuid::Uuid;

use crate::{communication::AlphaStatistics, protocol};

/// Server lifecycle state for the Alpha integration runtime.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaServerState {
    Created,
    Boot,
    Listen,
    Accept,
    Authenticate,
    Runtime,
    Running,
    Shutdown,
}

/// Configuration for the integration protocol server.
#[derive(Debug, Clone)]
pub struct AlphaServerConfig {
    pub listen_addr: SocketAddr,
    pub auth_token: String,
}

impl Default for AlphaServerConfig {
    fn default() -> Self {
        Self {
            listen_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0),
            auth_token: "gate-alpha-token".to_string(),
        }
    }
}

#[derive(Debug)]
struct AlphaServerInner {
    config: AlphaServerConfig,
    state: RwLock<AlphaServerState>,
    bound_addr: RwLock<Option<SocketAddr>>,
    statistics: RwLock<AlphaStatistics>,
    shutdown: watch::Sender<bool>,
    task: RwLock<Option<JoinHandle<()>>>,
}

/// Real protocol server used by integration tests.
#[derive(Clone, Debug)]
pub struct AlphaServer {
    inner: Arc<AlphaServerInner>,
}

impl AlphaServer {
    pub fn new(config: AlphaServerConfig) -> Self {
        let (shutdown, _) = watch::channel(false);
        Self {
            inner: Arc::new(AlphaServerInner {
                config,
                state: RwLock::new(AlphaServerState::Created),
                bound_addr: RwLock::new(None),
                statistics: RwLock::new(AlphaStatistics::default()),
                shutdown,
                task: RwLock::new(None),
            }),
        }
    }

    pub async fn start(&self) -> Result<SocketAddr> {
        *self.inner.state.write().await = AlphaServerState::Boot;
        let listener = TcpListener::bind(self.inner.config.listen_addr)
            .await
            .context("bind Alpha integration server")?;
        let bound_addr = listener.local_addr()?;
        *self.inner.bound_addr.write().await = Some(bound_addr);
        *self.inner.state.write().await = AlphaServerState::Listen;

        let server = self.clone();
        let task = tokio::spawn(async move {
            server.accept_loop(listener).await;
        });
        *self.inner.task.write().await = Some(task);
        Ok(bound_addr)
    }

    pub async fn state(&self) -> AlphaServerState {
        *self.inner.state.read().await
    }

    pub async fn statistics(&self) -> AlphaStatistics {
        self.inner.statistics.read().await.clone()
    }

    pub async fn bound_addr(&self) -> Option<SocketAddr> {
        *self.inner.bound_addr.read().await
    }

    pub async fn shutdown(&self) -> Result<()> {
        *self.inner.state.write().await = AlphaServerState::Shutdown;
        let _ = self.inner.shutdown.send(true);
        if let Some(task) = self.inner.task.write().await.take() {
            task.abort();
        }
        Ok(())
    }

    async fn accept_loop(self, listener: TcpListener) {
        let mut shutdown = self.inner.shutdown.subscribe();

        loop {
            tokio::select! {
                _ = shutdown.changed() => break,
                accepted = listener.accept() => {
                    let Ok((stream, _remote)) = accepted else {
                        continue;
                    };
                    *self.inner.state.write().await = AlphaServerState::Accept;
                    self.inner.statistics.write().await.record_connect();

                    let server = self.clone();
                    tokio::spawn(async move {
                        server.handle_connection(stream).await;
                    });
                }
            }
        }
    }

    async fn handle_connection(self, mut stream: TcpStream) {
        let protocol = protocol::alpha_protocol();
        let mut authenticated = false;
        let mut session_id: Option<String> = None;
        let connected_at = Instant::now();

        loop {
            let message = match protocol::read_message(&mut stream, &protocol).await {
                Ok(Some(message)) => message,
                Ok(None) => break,
                Err(_) => break,
            };

            match message.header.message_type {
                MessageType::Request | MessageType::Heartbeat => {
                    self.inner.statistics.write().await.request_total += 1;
                }
                MessageType::Event => {
                    self.inner.statistics.write().await.event_total += 1;
                }
                _ => {}
            }

            match message.header.command {
                Command::AuthLogin => {
                    *self.inner.state.write().await = AlphaServerState::Authenticate;
                    let body = protocol::json_body(&message).unwrap_or(Value::Null);
                    let token = body.get("token").and_then(Value::as_str);
                    if token == Some(self.inner.config.auth_token.as_str()) {
                        authenticated = true;
                        let id = Uuid::new_v4().to_string();
                        session_id = Some(id.clone());
                        let mut statistics = self.inner.statistics.write().await;
                        statistics.auth.success += 1;
                        statistics.auth.active_session += 1;
                        drop(statistics);
                        let response = protocol::response_for(
                            &message,
                            protocol::ok(json!({ "sessionId": id })),
                        );
                        let _ = protocol::write_message(&mut stream, &protocol, &response).await;
                        self.inner.statistics.write().await.response_total += 1;
                        *self.inner.state.write().await = AlphaServerState::Running;
                    } else {
                        let mut statistics = self.inner.statistics.write().await;
                        statistics.auth.failure += 1;
                        statistics.auth.rejected_connection += 1;
                        drop(statistics);
                        let response = protocol::response_for(
                            &message,
                            protocol::err("AUTH_FAILED", "invalid token"),
                        );
                        let _ = protocol::write_message(&mut stream, &protocol, &response).await;
                        self.inner.statistics.write().await.response_total += 1;
                        break;
                    }
                }
                Command::HeartbeatPing if authenticated => {
                    let mut statistics = self.inner.statistics.write().await;
                    statistics.heartbeat.ping += 1;
                    statistics.heartbeat.record_pong(connected_at.elapsed());
                    drop(statistics);

                    let response = protocol::response_for(
                        &message,
                        protocol::ok(json!({
                            "sessionId": &session_id,
                            "kind": "pong"
                        })),
                    );
                    let _ = protocol::write_message(&mut stream, &protocol, &response).await;
                    self.inner.statistics.write().await.response_total += 1;
                }
                Command::StatisticsQuery if authenticated => {
                    let statistics = self.inner.statistics.read().await.clone();
                    let response =
                        protocol::response_for(&message, protocol::ok(json!(statistics)));
                    let _ = protocol::write_message(&mut stream, &protocol, &response).await;
                    self.inner.statistics.write().await.response_total += 1;
                }
                Command::LogSubscribe if authenticated => {
                    let response = protocol::response_for(
                        &message,
                        protocol::ok(json!({ "subscribed": true })),
                    );
                    let _ = protocol::write_message(&mut stream, &protocol, &response).await;
                    let event = protocol::event(
                        "runtime.log",
                        json!({
                            "level": "info",
                            "source": "integration",
                            "message": "Runtime Log"
                        }),
                    );
                    let _ = protocol::write_message(&mut stream, &protocol, &event).await;
                    let mut statistics = self.inner.statistics.write().await;
                    statistics.response_total += 1;
                    statistics.event_total += 1;
                }
                Command::SystemShutdown if authenticated => {
                    let response =
                        protocol::response_for(&message, protocol::ok(json!({ "shutdown": true })));
                    let _ = protocol::write_message(&mut stream, &protocol, &response).await;
                    self.inner.statistics.write().await.response_total += 1;
                    break;
                }
                _ if !authenticated => {
                    let response = protocol::response_for(
                        &message,
                        protocol::err("AUTH_REQUIRED", "authenticate before sending commands"),
                    );
                    let _ = protocol::write_message(&mut stream, &protocol, &response).await;
                    self.inner.statistics.write().await.response_total += 1;
                    break;
                }
                _ => {
                    let response =
                        protocol::response_for(&message, protocol::ok(json!({ "accepted": true })));
                    let _ = protocol::write_message(&mut stream, &protocol, &response).await;
                    self.inner.statistics.write().await.response_total += 1;
                }
            }
        }

        let mut statistics = self.inner.statistics.write().await;
        statistics.record_disconnect();
        if authenticated {
            statistics.auth.active_session = statistics.auth.active_session.saturating_sub(1);
        }
    }
}

impl Default for AlphaServer {
    fn default() -> Self {
        Self::new(AlphaServerConfig::default())
    }
}
