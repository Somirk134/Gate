//! Tunnel trait and tunnel identity.

use crate::error::TunnelError;
use crate::health::HealthStatus;
use crate::statistics::TunnelStatistics;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Stable tunnel identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TunnelId(Uuid);

impl TunnelId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn as_uuid(&self) -> Uuid {
        self.0
    }
}

impl Default for TunnelId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TunnelId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Public tunnel state exposed to managers and UI adapters.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TunnelStatus {
    Created,
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
    Error,
}

/// Protocol-agnostic tunnel contract.
pub trait Tunnel: Send + Sync {
    fn id(&self) -> TunnelId;

    fn start(&self) -> BoxFuture<'static, Result<(), TunnelError>>;

    fn stop(&self) -> BoxFuture<'static, Result<(), TunnelError>>;

    fn restart(&self) -> BoxFuture<'static, Result<(), TunnelError>>;

    fn status(&self) -> TunnelStatus;

    fn statistics(&self) -> TunnelStatistics;

    fn health(&self) -> HealthStatus;
}
