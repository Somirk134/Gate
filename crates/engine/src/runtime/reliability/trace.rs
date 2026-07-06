//! Runtime trace context shared by reliability components.

use crate::connection::ConnectionId;
use crate::core::TunnelId;
use crate::session::SessionId;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TraceId(Uuid);

impl TraceId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TraceId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TraceId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RuntimeId(Uuid);

impl RuntimeId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for RuntimeId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RuntimeId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.0)
    }
}

/// Stable correlation identifiers for runtime, tunnel, session, and connection logs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeTraceContext {
    pub trace_id: TraceId,
    pub runtime_id: RuntimeId,
    pub tunnel_id: TunnelId,
    pub session_id: Option<SessionId>,
    pub connection_id: Option<ConnectionId>,
}

impl RuntimeTraceContext {
    pub fn new(runtime_id: RuntimeId, tunnel_id: TunnelId) -> Self {
        Self {
            trace_id: TraceId::new(),
            runtime_id,
            tunnel_id,
            session_id: None,
            connection_id: None,
        }
    }

    pub fn with_session(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_connection(mut self, connection_id: ConnectionId) -> Self {
        self.connection_id = Some(connection_id);
        self
    }
}
