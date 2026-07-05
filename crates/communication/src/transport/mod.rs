//! Transport abstraction for TCP today and WebSocket/QUIC later.

use serde::{Deserialize, Serialize};

use crate::shared::CommunicationFuture;
use gate_protocol::Message;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransportKind {
    Tcp,
    WebSocket,
    Quic,
    Mock,
    Custom,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransportEndpoint {
    Tcp { host: String, port: u16 },
    WebSocket { url: String },
    Quic { host: String, port: u16 },
    Custom(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TransportState {
    Created,
    Connecting,
    Connected,
    Running,
    Reconnecting,
    Disconnecting,
    Disconnected,
    Closed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportCapabilities {
    pub kind: TransportKind,
    pub supports_reconnect: bool,
    pub supports_priority_queue: bool,
    pub supports_binary: bool,
}

impl TransportCapabilities {
    pub fn tcp() -> Self {
        Self {
            kind: TransportKind::Tcp,
            supports_reconnect: true,
            supports_priority_queue: false,
            supports_binary: true,
        }
    }
}

/// Async transport boundary. Implementations own concrete IO details.
pub trait Transport: Send + Sync {
    fn name(&self) -> &'static str;

    fn kind(&self) -> TransportKind;

    fn state(&self) -> TransportState;

    fn capabilities(&self) -> TransportCapabilities;

    fn connect<'a>(&'a self, endpoint: TransportEndpoint) -> CommunicationFuture<'a, ()>;

    fn disconnect<'a>(&'a self) -> CommunicationFuture<'a, ()>;

    fn send<'a>(&'a self, message: Message) -> CommunicationFuture<'a, ()>;

    fn receive<'a>(&'a self) -> CommunicationFuture<'a, Option<Message>>;

    fn reconnect<'a>(&'a self) -> CommunicationFuture<'a, ()>;
}
