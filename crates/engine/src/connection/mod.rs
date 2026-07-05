//! Connection identity, state, context, pool, and manager.

use crate::config::ProtocolKind;
use crate::core::TunnelId;
use crate::error::ConnectionError;
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectionId(Uuid);

impl ConnectionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ConnectionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ConnectionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionState {
    Created,
    Connecting,
    Connected,
    Paused,
    Closing,
    Closed,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionContext {
    pub tunnel_id: TunnelId,
    pub connection_id: ConnectionId,
    pub protocol: ProtocolKind,
    pub peer_addr: Option<String>,
    pub created_at_millis: u64,
}

impl ConnectionContext {
    pub fn new(tunnel_id: TunnelId, protocol: ProtocolKind) -> Self {
        Self {
            tunnel_id,
            connection_id: ConnectionId::new(),
            protocol,
            peer_addr: None,
            created_at_millis: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}

#[derive(Debug)]
pub struct Connection {
    id: ConnectionId,
    context: ConnectionContext,
    state: RwLock<ConnectionState>,
}

impl Connection {
    pub fn new(context: ConnectionContext) -> Self {
        Self {
            id: context.connection_id,
            context,
            state: RwLock::new(ConnectionState::Created),
        }
    }

    pub fn id(&self) -> ConnectionId {
        self.id
    }

    pub fn context(&self) -> &ConnectionContext {
        &self.context
    }

    pub fn state(&self) -> ConnectionState {
        *self.state.read()
    }
}

#[derive(Default)]
pub struct ConnectionPool {
    connections: DashMap<ConnectionId, Arc<Connection>>,
}

impl ConnectionPool {
    pub fn insert(&self, connection: Connection) -> Arc<Connection> {
        let connection = Arc::new(connection);
        self.connections.insert(connection.id(), Arc::clone(&connection));
        connection
    }

    pub fn get(&self, id: &ConnectionId) -> Option<Arc<Connection>> {
        self.connections.get(id).map(|entry| Arc::clone(entry.value()))
    }

    pub fn remove(&self, id: &ConnectionId) -> Option<Arc<Connection>> {
        self.connections.remove(id).map(|(_, connection)| connection)
    }

    pub fn len(&self) -> usize {
        self.connections.len()
    }

    pub fn is_empty(&self) -> bool {
        self.connections.is_empty()
    }
}

#[derive(Default)]
pub struct ConnectionManager {
    pool: ConnectionPool,
}

impl ConnectionManager {
    pub fn create(&self, context: ConnectionContext) -> Arc<Connection> {
        self.pool.insert(Connection::new(context))
    }

    pub fn get(&self, id: &ConnectionId) -> Result<Arc<Connection>, ConnectionError> {
        self.pool
            .get(id)
            .ok_or(ConnectionError::NotFound { id: *id })
    }

    pub fn close(&self, id: &ConnectionId) -> Result<(), ConnectionError> {
        self.pool
            .remove(id)
            .map(|_| ())
            .ok_or(ConnectionError::NotFound { id: *id })
    }

    pub fn pool(&self) -> &ConnectionPool {
        &self.pool
    }
}
