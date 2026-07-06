use anyhow::{anyhow, Result};
use gate_communication::{
    TcpTransport, Transport, TransportEndpoint, TransportState,
};
use gate_protocol::{Command, Message};
use serde_json::{json, Value};
use std::{net::SocketAddr, sync::Arc, time::Instant};

use crate::{communication::HeartbeatSnapshot, protocol};

/// Client lifecycle state used by integration tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlphaClientState {
    Created,
    Connected,
    Authenticated,
    Running,
    Disconnected,
    Failed,
}

/// Real TCP client that uses `gate-communication` transport.
pub struct AlphaClient {
    transport: Arc<TcpTransport>,
    state: AlphaClientState,
    session_id: Option<String>,
    heartbeat: HeartbeatSnapshot,
}

impl AlphaClient {
    pub fn new() -> Self {
        Self {
            transport: Arc::new(TcpTransport::new()),
            state: AlphaClientState::Created,
            session_id: None,
            heartbeat: HeartbeatSnapshot::default(),
        }
    }

    pub fn state(&self) -> AlphaClientState {
        self.state
    }

    pub fn session_id(&self) -> Option<&str> {
        self.session_id.as_deref()
    }

    pub fn transport_state(&self) -> TransportState {
        self.transport.state()
    }

    pub fn heartbeat_snapshot(&self) -> &HeartbeatSnapshot {
        &self.heartbeat
    }

    pub async fn connect(&mut self, addr: SocketAddr) -> Result<()> {
        self.transport
            .connect(TransportEndpoint::Tcp {
                host: addr.ip().to_string(),
                port: addr.port(),
            })
            .await?;
        self.state = AlphaClientState::Connected;
        Ok(())
    }

    pub async fn authenticate(&mut self, token: &str) -> Result<String> {
        let response = self
            .request(Command::AuthLogin, json!({ "token": token }))
            .await?;
        let body = protocol::json_body(&response)?;
        if body.get("ok").and_then(Value::as_bool) != Some(true) {
            self.state = AlphaClientState::Failed;
            return Err(anyhow!("authentication rejected: {body}"));
        }

        let session_id = body
            .pointer("/data/sessionId")
            .and_then(Value::as_str)
            .ok_or_else(|| anyhow!("authentication response did not include a session id"))?
            .to_string();
        self.session_id = Some(session_id.clone());
        self.state = AlphaClientState::Authenticated;
        Ok(session_id)
    }

    pub async fn send_heartbeat(&mut self) -> Result<u64> {
        self.heartbeat.ping += 1;
        let sent_at = Instant::now();
        let response = self
            .request(Command::HeartbeatPing, json!({ "sentAt": chrono::Utc::now() }))
            .await?;
        let body = protocol::json_body(&response)?;
        if body.get("ok").and_then(Value::as_bool) != Some(true) {
            self.heartbeat.timeout += 1;
            return Err(anyhow!("heartbeat rejected: {body}"));
        }

        let rtt = sent_at.elapsed();
        self.heartbeat.record_pong(rtt);
        self.state = AlphaClientState::Running;
        Ok(self.heartbeat.last_rtt_ms.unwrap_or_default())
    }

    pub async fn request(&self, command: Command, body: Value) -> Result<Message> {
        let message = protocol::request(command, body);
        self.transport.send(message).await?;
        let response = self
            .transport
            .receive()
            .await?
            .ok_or_else(|| anyhow!("connection closed before response"))?;
        Ok(response)
    }

    pub async fn receive(&self) -> Result<Option<Message>> {
        Ok(self.transport.receive().await?)
    }

    pub async fn reconnect(&mut self) -> Result<()> {
        self.transport.reconnect().await?;
        self.state = AlphaClientState::Connected;
        Ok(())
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        self.transport.disconnect().await?;
        self.state = AlphaClientState::Disconnected;
        Ok(())
    }
}

impl Default for AlphaClient {
    fn default() -> Self {
        Self::new()
    }
}
