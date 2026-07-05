//! Engine module manager.

use crate::connection::ConnectionManager;
use crate::health::HealthChecker;
use crate::heartbeat::HeartbeatManager;
use crate::router::TunnelRouter;
use crate::session::SessionManager;
use std::sync::Arc;

/// Aggregates engine subsystem managers.
#[derive(Clone)]
pub struct EngineManager {
    pub router: Arc<TunnelRouter>,
    pub connections: Arc<ConnectionManager>,
    pub sessions: Arc<SessionManager>,
    pub heartbeat: Arc<HeartbeatManager>,
    pub health: Arc<HealthChecker>,
}

impl Default for EngineManager {
    fn default() -> Self {
        Self::new()
    }
}

impl EngineManager {
    pub fn new() -> Self {
        Self {
            router: Arc::new(TunnelRouter::default()),
            connections: Arc::new(ConnectionManager::default()),
            sessions: Arc::new(SessionManager::default()),
            heartbeat: Arc::new(HeartbeatManager::default()),
            health: Arc::new(HealthChecker::default()),
        }
    }
}
