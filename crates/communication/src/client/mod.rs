//! Client-side communication primitives.

use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::{
    connection::{
        BasicConnection, Connection, ConnectionContext, ConnectionId, ConnectionMetadata,
        ConnectionRole, ConnectionState,
    },
    dispatcher::{CommandDispatcher, EventDispatcher, ResponseDispatcher},
    error::CommunicationResult,
    request::{Request, RequestManager, ResponseReceiver},
    shared::CommunicationLogger,
    transport::{Transport, TransportEndpoint},
};
use gate_protocol::Message;

pub use crate::session::ClientSession;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ClientState {
    Created,
    Connecting,
    Connected,
    Authenticated,
    Running,
    Reconnecting,
    Disconnected,
    Closed,
    Failed,
}

pub struct ClientTransport {
    inner: Arc<dyn Transport>,
}

impl ClientTransport {
    pub fn new(inner: Arc<dyn Transport>) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> Arc<dyn Transport> {
        Arc::clone(&self.inner)
    }
}

#[derive(Clone)]
pub struct ClientConnection {
    inner: Arc<BasicConnection>,
}

impl ClientConnection {
    pub fn new(metadata: ConnectionMetadata) -> Self {
        Self {
            inner: Arc::new(BasicConnection::new(ConnectionContext::new(
                ConnectionRole::Client,
                metadata,
            ))),
        }
    }

    pub fn inner(&self) -> Arc<BasicConnection> {
        Arc::clone(&self.inner)
    }
}

impl Connection for ClientConnection {
    fn id(&self) -> ConnectionId {
        self.inner.id()
    }

    fn context(&self) -> ConnectionContext {
        self.inner.context()
    }

    fn state(&self) -> ConnectionState {
        self.inner.state()
    }

    fn statistics(&self) -> crate::connection::ConnectionStatistics {
        self.inner.statistics()
    }

    fn transition(&self, next: ConnectionState) -> CommunicationResult<()> {
        self.inner.transition(next)
    }

    fn close<'a>(&'a self) -> crate::shared::CommunicationFuture<'a, ()> {
        self.inner.close()
    }
}

#[derive(Default)]
pub struct ClientRequestManager {
    inner: Arc<RequestManager>,
}

impl ClientRequestManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RequestManager::default()),
        }
    }

    pub fn register(&self, request: Request) -> CommunicationResult<ResponseReceiver> {
        self.inner.register(request)
    }

    pub fn inner(&self) -> Arc<RequestManager> {
        Arc::clone(&self.inner)
    }
}

#[derive(Default)]
pub struct ClientEventManager {
    inner: Arc<EventDispatcher>,
}

impl ClientEventManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(EventDispatcher::default()),
        }
    }

    pub fn inner(&self) -> Arc<EventDispatcher> {
        Arc::clone(&self.inner)
    }
}

pub struct ClientDispatcher {
    pub command: CommandDispatcher,
    pub response: ResponseDispatcher,
}

impl ClientDispatcher {
    pub fn new(request_manager: Arc<RequestManager>) -> Self {
        Self {
            command: CommandDispatcher::default(),
            response: ResponseDispatcher::new(request_manager),
        }
    }
}

#[derive(Default)]
pub struct ConnectionPool {
    connections: DashMap<ConnectionId, Arc<ClientConnection>>,
}

impl ConnectionPool {
    pub fn insert(&self, connection: ClientConnection) -> Arc<ClientConnection> {
        let connection = Arc::new(connection);
        self.connections
            .insert(connection.id(), Arc::clone(&connection));
        connection
    }

    pub fn get(&self, id: &ConnectionId) -> Option<Arc<ClientConnection>> {
        self.connections
            .get(id)
            .map(|entry| Arc::clone(entry.value()))
    }

    pub fn remove(&self, id: &ConnectionId) -> Option<Arc<ClientConnection>> {
        self.connections
            .remove(id)
            .map(|(_, connection)| connection)
    }

    pub fn len(&self) -> usize {
        self.connections.len()
    }

    pub fn is_empty(&self) -> bool {
        self.connections.is_empty()
    }
}

/// Client communication facade for Rust/Tauri commands.
pub struct CommunicationService {
    state: RwLock<ClientState>,
    transport: ClientTransport,
    request_manager: ClientRequestManager,
    event_manager: ClientEventManager,
    pub session: ClientSession,
}

impl CommunicationService {
    pub fn new(transport: Arc<dyn Transport>) -> Self {
        Self {
            state: RwLock::new(ClientState::Created),
            transport: ClientTransport::new(transport),
            request_manager: ClientRequestManager::new(),
            event_manager: ClientEventManager::new(),
            session: ClientSession::new(),
        }
    }

    pub fn state(&self) -> ClientState {
        *self.state.read()
    }

    pub async fn connect(&self, endpoint: TransportEndpoint) -> CommunicationResult<()> {
        *self.state.write() = ClientState::Connecting;
        let transport = self.transport.inner();
        transport.connect(endpoint).await?;
        *self.state.write() = ClientState::Connected;
        Ok(())
    }

    pub async fn disconnect(&self) -> CommunicationResult<()> {
        let transport = self.transport.inner();
        transport.disconnect().await?;
        *self.state.write() = ClientState::Disconnected;
        CommunicationLogger::disconnect(None, "client requested disconnect");
        Ok(())
    }

    pub async fn notify(&self, message: Message) -> CommunicationResult<()> {
        let transport = self.transport.inner();
        transport.send(message).await
    }

    pub fn register_request(&self, request: Request) -> CommunicationResult<ResponseReceiver> {
        self.request_manager.register(request)
    }

    pub fn request_manager(&self) -> Arc<RequestManager> {
        self.request_manager.inner()
    }

    pub fn event_manager(&self) -> Arc<EventDispatcher> {
        self.event_manager.inner()
    }

    pub async fn request(&self, request: Request) -> CommunicationResult<ResponseReceiver> {
        let message = request.message.clone();
        let receiver = self.register_request(request)?;
        let transport = self.transport.inner();
        transport.send(message).await?;
        Ok(receiver)
    }
}
