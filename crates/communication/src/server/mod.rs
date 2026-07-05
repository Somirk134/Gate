//! Server-side communication primitives and registries.

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{fmt, sync::Arc};
use uuid::Uuid;

use crate::{
    connection::{
        BasicConnection, Connection, ConnectionContext, ConnectionId, ConnectionMetadata,
        ConnectionRole, ConnectionState,
    },
    dispatcher::CommandDispatcher,
    error::{CommunicationResult, ConnectionError},
    metrics::CommunicationMetrics,
    transport::Transport,
};

pub use crate::session::ServerSession;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServerState {
    Created,
    Starting,
    Running,
    Draining,
    Stopped,
    Failed,
}

pub struct ServerTransport {
    inner: Arc<dyn Transport>,
}

impl ServerTransport {
    pub fn new(inner: Arc<dyn Transport>) -> Self {
        Self { inner }
    }

    pub fn inner(&self) -> Arc<dyn Transport> {
        Arc::clone(&self.inner)
    }
}

#[derive(Clone)]
pub struct ServerConnection {
    inner: Arc<BasicConnection>,
}

impl ServerConnection {
    pub fn new(metadata: ConnectionMetadata) -> Self {
        Self {
            inner: Arc::new(BasicConnection::new(ConnectionContext::new(
                ConnectionRole::Server,
                metadata,
            ))),
        }
    }
}

impl Connection for ServerConnection {
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
pub struct ServerDispatcher {
    command: CommandDispatcher,
}

impl ServerDispatcher {
    pub fn command(&self) -> &CommandDispatcher {
        &self.command
    }
}

#[derive(Default)]
pub struct ConnectionRegistry {
    connections: DashMap<ConnectionId, Arc<ServerConnection>>,
}

impl ConnectionRegistry {
    pub fn insert(&self, connection: ServerConnection) -> Arc<ServerConnection> {
        let connection = Arc::new(connection);
        self.connections.insert(connection.id(), Arc::clone(&connection));
        connection
    }

    pub fn get(&self, id: &ConnectionId) -> Option<Arc<ServerConnection>> {
        self.connections.get(id).map(|entry| Arc::clone(entry.value()))
    }

    pub fn remove(&self, id: &ConnectionId) -> Option<Arc<ServerConnection>> {
        self.connections.remove(id).map(|(_, connection)| connection)
    }

    pub fn len(&self) -> usize {
        self.connections.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClientId(Uuid);

impl ClientId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ClientId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ClientId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Default)]
pub struct ClientRegistry {
    clients: DashMap<ClientId, ConnectionId>,
}

impl ClientRegistry {
    pub fn bind(&self, client_id: ClientId, connection_id: ConnectionId) {
        self.clients.insert(client_id, connection_id);
    }

    pub fn connection_id(&self, client_id: &ClientId) -> Option<ConnectionId> {
        self.clients.get(client_id).map(|entry| *entry.value())
    }

    pub fn unbind(&self, client_id: &ClientId) -> Option<ConnectionId> {
        self.clients.remove(client_id).map(|(_, connection_id)| connection_id)
    }
}

pub struct ConnectionManager {
    registry: ConnectionRegistry,
    client_registry: ClientRegistry,
    metrics: Arc<CommunicationMetrics>,
}

impl ConnectionManager {
    pub fn new(metrics: Arc<CommunicationMetrics>) -> Self {
        Self {
            registry: ConnectionRegistry::default(),
            client_registry: ClientRegistry::default(),
            metrics,
        }
    }

    pub fn register(&self, connection: ServerConnection) -> Arc<ServerConnection> {
        self.metrics.record_connected();
        self.registry.insert(connection)
    }

    pub fn get(&self, id: &ConnectionId) -> CommunicationResult<Arc<ServerConnection>> {
        self.registry
            .get(id)
            .ok_or_else(|| ConnectionError::NotFound { id: *id }.into())
    }

    pub fn close(&self, id: &ConnectionId) -> CommunicationResult<()> {
        self.registry
            .remove(id)
            .map(|_| ())
            .ok_or_else(|| ConnectionError::NotFound { id: *id }.into())
    }

    pub fn registry(&self) -> &ConnectionRegistry {
        &self.registry
    }

    pub fn client_registry(&self) -> &ClientRegistry {
        &self.client_registry
    }

    pub fn metrics(&self) -> Arc<CommunicationMetrics> {
        Arc::clone(&self.metrics)
    }
}

pub fn create_server_session() -> ServerSession {
    ServerSession::new()
}
