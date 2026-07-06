//! Transport abstraction for TCP today and WebSocket/QUIC later.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::Mutex,
};

use crate::{
    error::TransportError,
    shared::{CommunicationFuture, CommunicationLogger},
};
use gate_protocol::{Frame, FrameEncoder, Message, ProtocolBuilder, ProtocolManager};
use parking_lot::RwLock;

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

/// Length-prefixed TCP transport backed by the Gate V1 JSON protocol.
#[derive(Clone)]
pub struct TcpTransport {
    state: Arc<RwLock<TransportState>>,
    endpoint: Arc<RwLock<Option<TransportEndpoint>>>,
    stream: Arc<Mutex<Option<TcpStream>>>,
    protocol: Arc<Mutex<ProtocolManager>>,
}

impl TcpTransport {
    /// Creates a disconnected TCP transport.
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(TransportState::Created)),
            endpoint: Arc::new(RwLock::new(None)),
            stream: Arc::new(Mutex::new(None)),
            protocol: Arc::new(Mutex::new(ProtocolBuilder::new().build())),
        }
    }

    /// Returns the endpoint used for the current or last connection.
    pub fn endpoint(&self) -> Option<TransportEndpoint> {
        self.endpoint.read().clone()
    }

    fn set_state(&self, state: TransportState) {
        *self.state.write() = state;
    }

    async fn connect_tcp(&self, endpoint: TransportEndpoint) -> Result<(), TransportError> {
        let TransportEndpoint::Tcp { host, port } = endpoint.clone() else {
            return Err(TransportError::Unavailable(
                "TcpTransport only supports TCP endpoints".to_string(),
            ));
        };

        self.set_state(TransportState::Connecting);
        let stream = TcpStream::connect((host.as_str(), port))
            .await
            .map_err(|source| TransportError::Connect(source.to_string()))?;
        stream
            .set_nodelay(true)
            .map_err(|source| TransportError::Connect(source.to_string()))?;

        *self.endpoint.write() = Some(endpoint);
        *self.stream.lock().await = Some(stream);
        self.set_state(TransportState::Connected);
        Ok(())
    }

    async fn write_message(&self, message: Message) -> Result<(), TransportError> {
        let mut guard = self.stream.lock().await;
        let stream = guard.as_mut().ok_or(TransportError::Closed)?;
        let payload = {
            let protocol = self.protocol.lock().await;
            protocol
                .encode(&message)
                .map_err(|source| TransportError::Write(source.to_string()))?
        };
        let frame =
            Frame::new(payload).map_err(|source| TransportError::Write(source.to_string()))?;
        let bytes = FrameEncoder::encode(&frame);

        stream
            .write_all(&bytes)
            .await
            .map_err(|source| TransportError::Write(source.to_string()))?;
        stream
            .flush()
            .await
            .map_err(|source| TransportError::Write(source.to_string()))?;
        CommunicationLogger::send(None, &message);
        Ok(())
    }

    async fn read_message(&self) -> Result<Option<Message>, TransportError> {
        let mut guard = self.stream.lock().await;
        let stream = guard.as_mut().ok_or(TransportError::Closed)?;
        let mut length = [0_u8; 4];

        match stream.read_exact(&mut length).await {
            Ok(_) => {}
            Err(source) if source.kind() == std::io::ErrorKind::UnexpectedEof => {
                self.set_state(TransportState::Disconnected);
                return Ok(None);
            }
            Err(source) => return Err(TransportError::Read(source.to_string())),
        }

        let length = u32::from_be_bytes(length) as usize;
        let mut payload = vec![0_u8; length];
        stream
            .read_exact(&mut payload)
            .await
            .map_err(|source| TransportError::Read(source.to_string()))?;
        let message = {
            let protocol = self.protocol.lock().await;
            protocol
                .decode(&payload)
                .map_err(|source| TransportError::Read(source.to_string()))?
        };

        CommunicationLogger::receive(None, &message);
        Ok(Some(message))
    }
}

impl Default for TcpTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for TcpTransport {
    fn name(&self) -> &'static str {
        "tcp"
    }

    fn kind(&self) -> TransportKind {
        TransportKind::Tcp
    }

    fn state(&self) -> TransportState {
        *self.state.read()
    }

    fn capabilities(&self) -> TransportCapabilities {
        TransportCapabilities::tcp()
    }

    fn connect<'a>(&'a self, endpoint: TransportEndpoint) -> CommunicationFuture<'a, ()> {
        Box::pin(async move { self.connect_tcp(endpoint).await.map_err(Into::into) })
    }

    fn disconnect<'a>(&'a self) -> CommunicationFuture<'a, ()> {
        Box::pin(async move {
            self.set_state(TransportState::Disconnecting);
            if let Some(mut stream) = self.stream.lock().await.take() {
                stream
                    .shutdown()
                    .await
                    .map_err(|source| TransportError::Disconnect(source.to_string()))?;
            }
            self.set_state(TransportState::Disconnected);
            Ok(())
        })
    }

    fn send<'a>(&'a self, message: Message) -> CommunicationFuture<'a, ()> {
        Box::pin(async move { self.write_message(message).await.map_err(Into::into) })
    }

    fn receive<'a>(&'a self) -> CommunicationFuture<'a, Option<Message>> {
        Box::pin(async move { self.read_message().await.map_err(Into::into) })
    }

    fn reconnect<'a>(&'a self) -> CommunicationFuture<'a, ()> {
        Box::pin(async move {
            let endpoint = self.endpoint();
            self.disconnect().await?;
            self.set_state(TransportState::Reconnecting);
            let endpoint = endpoint.ok_or_else(|| {
                TransportError::Reconnect("cannot reconnect before initial connect".to_string())
            })?;
            self.connect_tcp(endpoint).await?;
            Ok(())
        })
    }
}
