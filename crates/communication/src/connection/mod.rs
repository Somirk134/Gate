//! Connection identity, metadata, lifecycle state, and shared connection trait.

use chrono::{DateTime, Utc};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt};
use uuid::Uuid;

use crate::{
    error::{CommunicationResult, ConnectionError},
    shared::{CommunicationFuture, CommunicationLogger},
};

/// Stable identity for a client/server connection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ConnectionId(Uuid);

impl ConnectionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(self) -> Uuid {
        self.0
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

/// Side that owns a connection instance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConnectionRole {
    Client,
    Server,
    Mock,
}

/// Lifecycle shared by client, server, and transport-backed connections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConnectionState {
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

impl ConnectionState {
    pub fn can_transition_to(self, next: Self) -> bool {
        matches!(
            (self, next),
            (Self::Created, Self::Connecting)
                | (Self::Connecting, Self::Connected)
                | (Self::Connected, Self::Authenticated)
                | (Self::Connected, Self::Running)
                | (Self::Authenticated, Self::Running)
                | (Self::Running, Self::Reconnecting)
                | (Self::Reconnecting, Self::Connected)
                | (Self::Connected, Self::Disconnected)
                | (Self::Authenticated, Self::Disconnected)
                | (Self::Running, Self::Disconnected)
                | (Self::Reconnecting, Self::Disconnected)
                | (Self::Connecting, Self::Disconnected)
                | (Self::Disconnected, Self::Connecting)
                | (Self::Disconnected, Self::Closed)
                | (Self::Created, Self::Closed)
                | (Self::Connecting, Self::Failed)
                | (Self::Connected, Self::Failed)
                | (Self::Authenticated, Self::Failed)
                | (Self::Running, Self::Failed)
                | (Self::Reconnecting, Self::Failed)
                | (Self::Failed, Self::Closed)
        )
    }
}

/// Metadata attached to a connection for diagnostics and routing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct ConnectionMetadata {
    pub endpoint: Option<String>,
    pub transport: Option<String>,
    pub client_id: Option<String>,
    pub server_id: Option<String>,
    pub protocol_version: Option<String>,
    pub tags: BTreeMap<String, String>,
}

/// Communication-level counters for one connection.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionStatistics {
    pub connected_count: u64,
    pub reconnect_count: u64,
    pub failed_count: u64,
    pub send_count: u64,
    pub receive_count: u64,
    pub average_latency_ms: f64,
}

impl Default for ConnectionStatistics {
    fn default() -> Self {
        Self {
            connected_count: 0,
            reconnect_count: 0,
            failed_count: 0,
            send_count: 0,
            receive_count: 0,
            average_latency_ms: 0.0,
        }
    }
}

/// Immutable connection context with mutable lifecycle stored separately.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionContext {
    pub id: ConnectionId,
    pub role: ConnectionRole,
    pub metadata: ConnectionMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ConnectionContext {
    pub fn new(role: ConnectionRole, metadata: ConnectionMetadata) -> Self {
        let now = Utc::now();

        Self {
            id: ConnectionId::new(),
            role,
            metadata,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Common connection boundary used by client, server, and mocks.
pub trait Connection: Send + Sync {
    fn id(&self) -> ConnectionId;

    fn context(&self) -> ConnectionContext;

    fn state(&self) -> ConnectionState;

    fn statistics(&self) -> ConnectionStatistics;

    fn transition(&self, next: ConnectionState) -> CommunicationResult<()>;

    fn close<'a>(&'a self) -> CommunicationFuture<'a, ()>;
}

/// Minimal in-memory connection implementation for managers and mocks.
#[derive(Debug)]
pub struct BasicConnection {
    context: RwLock<ConnectionContext>,
    state: RwLock<ConnectionState>,
    statistics: RwLock<ConnectionStatistics>,
}

impl BasicConnection {
    pub fn new(context: ConnectionContext) -> Self {
        Self {
            context: RwLock::new(context),
            state: RwLock::new(ConnectionState::Created),
            statistics: RwLock::new(ConnectionStatistics::default()),
        }
    }

    pub fn record_send(&self) {
        self.statistics.write().send_count += 1;
    }

    pub fn record_receive(&self) {
        self.statistics.write().receive_count += 1;
    }

    pub fn record_latency(&self, latency_ms: f64) {
        self.statistics.write().average_latency_ms = latency_ms;
    }
}

impl Connection for BasicConnection {
    fn id(&self) -> ConnectionId {
        self.context.read().id
    }

    fn context(&self) -> ConnectionContext {
        self.context.read().clone()
    }

    fn state(&self) -> ConnectionState {
        *self.state.read()
    }

    fn statistics(&self) -> ConnectionStatistics {
        self.statistics.read().clone()
    }

    fn transition(&self, next: ConnectionState) -> CommunicationResult<()> {
        let mut state = self.state.write();
        let current = *state;

        if current == next {
            return Ok(());
        }

        if !current.can_transition_to(next) {
            return Err(ConnectionError::InvalidTransition {
                from: current,
                to: next,
            }
            .into());
        }

        *state = next;
        self.context.write().updated_at = Utc::now();

        {
            let mut statistics = self.statistics.write();

            match next {
                ConnectionState::Connected => statistics.connected_count += 1,
                ConnectionState::Reconnecting => statistics.reconnect_count += 1,
                ConnectionState::Failed => statistics.failed_count += 1,
                _ => {}
            }
        }

        CommunicationLogger::connection(self.id(), next);

        Ok(())
    }

    fn close<'a>(&'a self) -> CommunicationFuture<'a, ()> {
        Box::pin(async move {
            match self.state() {
                ConnectionState::Closed => {}
                ConnectionState::Created
                | ConnectionState::Failed
                | ConnectionState::Disconnected => {
                    self.transition(ConnectionState::Closed)?;
                }
                _ => {
                    self.transition(ConnectionState::Disconnected)?;
                    self.transition(ConnectionState::Closed)?;
                }
            }

            Ok(())
        })
    }
}
