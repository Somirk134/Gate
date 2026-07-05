//! Session context and session manager.

use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt, sync::Arc};
use uuid::Uuid;

use crate::{error::CommunicationResult, shared::CommunicationFuture};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(Uuid);

impl SessionId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionRole {
    Client,
    Server,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionState {
    Created,
    Active,
    Suspended,
    Closed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SessionContext {
    pub id: SessionId,
    pub role: SessionRole,
    pub state: SessionState,
    pub connection_id: Option<crate::connection::ConnectionId>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub attributes: BTreeMap<String, String>,
}

impl SessionContext {
    pub fn new(role: SessionRole) -> Self {
        let now = Utc::now();

        Self {
            id: SessionId::new(),
            role,
            state: SessionState::Created,
            connection_id: None,
            created_at: now,
            updated_at: now,
            attributes: BTreeMap::new(),
        }
    }
}

pub trait Session: Send + Sync {
    fn id(&self) -> SessionId;

    fn context(&self) -> SessionContext;

    fn close<'a>(&'a self) -> CommunicationFuture<'a, ()>;
}

#[derive(Debug, Clone)]
pub struct ClientSession {
    context: SessionContext,
}

impl ClientSession {
    pub fn new() -> Self {
        Self {
            context: SessionContext::new(SessionRole::Client),
        }
    }
}

impl Default for ClientSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Session for ClientSession {
    fn id(&self) -> SessionId {
        self.context.id
    }

    fn context(&self) -> SessionContext {
        self.context.clone()
    }

    fn close<'a>(&'a self) -> CommunicationFuture<'a, ()> {
        Box::pin(async move { Ok(()) })
    }
}

#[derive(Debug, Clone)]
pub struct ServerSession {
    context: SessionContext,
}

impl ServerSession {
    pub fn new() -> Self {
        Self {
            context: SessionContext::new(SessionRole::Server),
        }
    }
}

impl Default for ServerSession {
    fn default() -> Self {
        Self::new()
    }
}

impl Session for ServerSession {
    fn id(&self) -> SessionId {
        self.context.id
    }

    fn context(&self) -> SessionContext {
        self.context.clone()
    }

    fn close<'a>(&'a self) -> CommunicationFuture<'a, ()> {
        Box::pin(async move { Ok(()) })
    }
}

#[derive(Default)]
pub struct SessionManager {
    sessions: DashMap<SessionId, Arc<dyn Session>>,
}

impl SessionManager {
    pub fn insert(&self, session: Arc<dyn Session>) {
        self.sessions.insert(session.id(), session);
    }

    pub fn get(&self, id: &SessionId) -> Option<Arc<dyn Session>> {
        self.sessions.get(id).map(|entry| Arc::clone(entry.value()))
    }

    pub fn remove(&self, id: &SessionId) -> Option<Arc<dyn Session>> {
        self.sessions.remove(id).map(|(_, session)| session)
    }

    pub fn len(&self) -> usize {
        self.sessions.len()
    }

    pub fn is_empty(&self) -> bool {
        self.sessions.is_empty()
    }
}

pub trait SessionStore: Send + Sync {
    fn save<'a>(&'a self, context: SessionContext) -> CommunicationFuture<'a, ()>;

    fn load<'a>(&'a self, id: SessionId) -> CommunicationFuture<'a, Option<SessionContext>>;

    fn delete<'a>(&'a self, id: SessionId) -> CommunicationFuture<'a, ()>;
}

pub fn ensure_session(_context: &SessionContext) -> CommunicationResult<()> {
    Ok(())
}
