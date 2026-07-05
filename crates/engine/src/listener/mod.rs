//! Listener abstractions for inbound traffic.

use crate::config::ProtocolKind;
use crate::error::ProtocolError;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ListenerId(Uuid);

impl ListenerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for ListenerId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for ListenerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ListenerStatus {
    Created,
    Listening,
    Paused,
    Stopped,
    Error,
}

/// Inbound listener trait.
pub trait Listener: Send + Sync {
    fn id(&self) -> ListenerId;

    fn protocol(&self) -> ProtocolKind;

    fn start(&self) -> BoxFuture<'static, Result<(), ProtocolError>>;

    fn stop(&self) -> BoxFuture<'static, Result<(), ProtocolError>>;

    fn status(&self) -> ListenerStatus;
}

#[derive(Debug, Clone)]
pub struct TcpListener {
    id: ListenerId,
}

impl TcpListener {
    pub fn new() -> Self {
        Self {
            id: ListenerId::new(),
        }
    }
}

impl Default for TcpListener {
    fn default() -> Self {
        Self::new()
    }
}

impl Listener for TcpListener {
    fn id(&self) -> ListenerId {
        self.id
    }

    fn protocol(&self) -> ProtocolKind {
        ProtocolKind::Tcp
    }

    fn start(&self) -> BoxFuture<'static, Result<(), ProtocolError>> {
        Box::pin(async { Ok(()) })
    }

    fn stop(&self) -> BoxFuture<'static, Result<(), ProtocolError>> {
        Box::pin(async { Ok(()) })
    }

    fn status(&self) -> ListenerStatus {
        ListenerStatus::Created
    }
}

#[derive(Debug, Clone)]
pub struct HttpListener {
    id: ListenerId,
}

impl HttpListener {
    pub fn new() -> Self {
        Self {
            id: ListenerId::new(),
        }
    }
}

impl Default for HttpListener {
    fn default() -> Self {
        Self::new()
    }
}

impl Listener for HttpListener {
    fn id(&self) -> ListenerId {
        self.id
    }

    fn protocol(&self) -> ProtocolKind {
        ProtocolKind::Http
    }

    fn start(&self) -> BoxFuture<'static, Result<(), ProtocolError>> {
        Box::pin(async { Ok(()) })
    }

    fn stop(&self) -> BoxFuture<'static, Result<(), ProtocolError>> {
        Box::pin(async { Ok(()) })
    }

    fn status(&self) -> ListenerStatus {
        ListenerStatus::Created
    }
}
