//! Mock communication components for unit and integration testing.

use parking_lot::RwLock;
use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
};

use gate_protocol::{Message, MessageType};

use crate::{
    client::ClientState,
    dispatcher::Dispatcher,
    error::{CommunicationResult, DispatcherError, TransportError},
    handler::HandlerContext,
    queue::{IncomingQueue, MessageEnvelope, OutgoingQueue},
    request::{Request, Response},
    server::ServerState,
    shared::{CommunicationFuture, CommunicationLogger},
    transport::{Transport, TransportCapabilities, TransportEndpoint, TransportKind, TransportState},
};

#[derive(Clone)]
pub struct MockTransport {
    state: Arc<RwLock<TransportState>>,
    endpoint: Arc<RwLock<Option<TransportEndpoint>>>,
    incoming: Arc<IncomingQueue>,
    outgoing: Arc<OutgoingQueue>,
    reconnect_count: Arc<AtomicU64>,
}

impl MockTransport {
    pub fn new() -> Self {
        Self {
            state: Arc::new(RwLock::new(TransportState::Created)),
            endpoint: Arc::new(RwLock::new(None)),
            incoming: Arc::new(IncomingQueue::default()),
            outgoing: Arc::new(OutgoingQueue::default()),
            reconnect_count: Arc::new(AtomicU64::new(0)),
        }
    }

    pub fn push_incoming(&self, message: Message) {
        self.incoming.enqueue(MessageEnvelope::new(message));
    }

    pub fn pop_outgoing(&self) -> Option<Message> {
        self.outgoing.dequeue().map(|envelope| envelope.message)
    }

    pub fn reconnect_count(&self) -> u64 {
        self.reconnect_count.load(Ordering::Relaxed)
    }
}

impl Default for MockTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for MockTransport {
    fn name(&self) -> &'static str {
        "mock"
    }

    fn kind(&self) -> TransportKind {
        TransportKind::Mock
    }

    fn state(&self) -> TransportState {
        *self.state.read()
    }

    fn capabilities(&self) -> TransportCapabilities {
        TransportCapabilities {
            kind: TransportKind::Mock,
            supports_reconnect: true,
            supports_priority_queue: true,
            supports_binary: true,
        }
    }

    fn connect<'a>(&'a self, endpoint: TransportEndpoint) -> CommunicationFuture<'a, ()> {
        Box::pin(async move {
            *self.state.write() = TransportState::Connecting;
            *self.endpoint.write() = Some(endpoint);
            *self.state.write() = TransportState::Connected;
            Ok(())
        })
    }

    fn disconnect<'a>(&'a self) -> CommunicationFuture<'a, ()> {
        Box::pin(async move {
            *self.state.write() = TransportState::Disconnected;
            Ok(())
        })
    }

    fn send<'a>(&'a self, message: Message) -> CommunicationFuture<'a, ()> {
        Box::pin(async move {
            if matches!(self.state(), TransportState::Closed | TransportState::Failed) {
                return Err(TransportError::Closed.into());
            }

            CommunicationLogger::send(None, &message);
            self.outgoing.enqueue(MessageEnvelope::new(message));
            Ok(())
        })
    }

    fn receive<'a>(&'a self) -> CommunicationFuture<'a, Option<Message>> {
        Box::pin(async move {
            let message = self.incoming.dequeue().map(|envelope| envelope.message);

            if let Some(message) = &message {
                CommunicationLogger::receive(None, message);
            }

            Ok(message)
        })
    }

    fn reconnect<'a>(&'a self) -> CommunicationFuture<'a, ()> {
        Box::pin(async move {
            *self.state.write() = TransportState::Reconnecting;
            self.reconnect_count.fetch_add(1, Ordering::Relaxed);
            *self.state.write() = TransportState::Connected;
            Ok(())
        })
    }
}

pub struct MockDispatcher;

impl Dispatcher for MockDispatcher {
    fn dispatch<'a>(
        &'a self,
        message: Message,
        _context: HandlerContext,
    ) -> CommunicationFuture<'a, Option<Message>> {
        Box::pin(async move {
            if message.header.message_type != MessageType::Request {
                return Err(DispatcherError::UnsupportedMessageType {
                    message_type: message.header.message_type,
                }
                .into());
            }

            let request = Request::new(message, 30_000);
            Ok(Some(MockResponse::from_request(&request).message))
        })
    }
}

pub struct MockResponse;

impl MockResponse {
    pub fn from_request(request: &Request) -> Response {
        Response::empty(request.id, request.command())
    }
}

pub struct MockClient {
    transport: Arc<MockTransport>,
    state: RwLock<ClientState>,
}

impl MockClient {
    pub fn new(transport: Arc<MockTransport>) -> Self {
        Self {
            transport,
            state: RwLock::new(ClientState::Created),
        }
    }

    pub fn state(&self) -> ClientState {
        *self.state.read()
    }

    pub async fn connect(&self, endpoint: TransportEndpoint) -> CommunicationResult<()> {
        *self.state.write() = ClientState::Connecting;
        self.transport.connect(endpoint).await?;
        *self.state.write() = ClientState::Connected;
        Ok(())
    }

    pub async fn disconnect(&self) -> CommunicationResult<()> {
        self.transport.disconnect().await?;
        *self.state.write() = ClientState::Disconnected;
        Ok(())
    }

    pub async fn reconnect(&self) -> CommunicationResult<()> {
        *self.state.write() = ClientState::Reconnecting;
        self.transport.reconnect().await?;
        *self.state.write() = ClientState::Connected;
        Ok(())
    }

    pub async fn send(&self, message: Message) -> CommunicationResult<()> {
        self.transport.send(message).await
    }
}

pub struct MockServer {
    transport: Arc<MockTransport>,
    state: RwLock<ServerState>,
}

impl MockServer {
    pub fn new(transport: Arc<MockTransport>) -> Self {
        Self {
            transport,
            state: RwLock::new(ServerState::Created),
        }
    }

    pub fn state(&self) -> ServerState {
        *self.state.read()
    }

    pub async fn start(&self, endpoint: TransportEndpoint) -> CommunicationResult<()> {
        *self.state.write() = ServerState::Starting;
        self.transport.connect(endpoint).await?;
        *self.state.write() = ServerState::Running;
        Ok(())
    }

    pub async fn stop(&self) -> CommunicationResult<()> {
        self.transport.disconnect().await?;
        *self.state.write() = ServerState::Stopped;
        Ok(())
    }
}
