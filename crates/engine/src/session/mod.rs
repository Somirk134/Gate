//! Session identity, context, store, and manager.

use crate::core::TunnelId;
use crate::error::EngineError;
use dashmap::DashMap;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use uuid::Uuid;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionContext {
    pub session_id: SessionId,
    pub tunnel_id: TunnelId,
    pub created_at_millis: u64,
}

impl SessionContext {
    pub fn new(tunnel_id: TunnelId) -> Self {
        Self {
            session_id: SessionId::new(),
            tunnel_id,
            created_at_millis: chrono::Utc::now().timestamp_millis() as u64,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: SessionId,
    pub context: SessionContext,
}

impl Session {
    pub fn new(context: SessionContext) -> Self {
        Self {
            id: context.session_id,
            context,
        }
    }
}

/// Reserved persistence boundary for future durable sessions.
pub trait SessionStore: Send + Sync {
    fn save(&self, session: Session) -> BoxFuture<'static, Result<(), EngineError>>;

    fn load(&self, id: SessionId) -> BoxFuture<'static, Result<Option<Session>, EngineError>>;

    fn delete(&self, id: SessionId) -> BoxFuture<'static, Result<(), EngineError>>;
}

#[derive(Default)]
pub struct SessionManager {
    sessions: DashMap<SessionId, Arc<Session>>,
}

impl SessionManager {
    pub fn create(&self, context: SessionContext) -> Arc<Session> {
        let session = Arc::new(Session::new(context));
        self.sessions.insert(session.id, Arc::clone(&session));
        session
    }

    pub fn get(&self, id: &SessionId) -> Option<Arc<Session>> {
        self.sessions.get(id).map(|entry| Arc::clone(entry.value()))
    }

    pub fn remove(&self, id: &SessionId) -> Option<Arc<Session>> {
        self.sessions.remove(id).map(|(_, session)| session)
    }
}
