//! Engine module manager.

use crate::connection::ConnectionManager;
use crate::connection_monitor::ConnectionMonitorManager;
use crate::health::HealthManager;
use crate::heartbeat::HeartbeatManager;
use crate::reconnect::ReconnectManager;
use crate::router::TunnelRouter;
use crate::session::SessionManager;
use crate::session_recovery::SessionRecoveryManager;
use crate::state_sync::StateSyncManager;
use std::sync::Arc;

/// Aggregates engine subsystem managers.
#[derive(Clone)]
pub struct EngineManager {
    pub router: Arc<TunnelRouter>,
    pub connections: Arc<ConnectionManager>,
    pub sessions: Arc<SessionManager>,
    pub heartbeat: Arc<HeartbeatManager>,
    pub reconnect: Arc<ReconnectManager>,
    pub recovery: Arc<SessionRecoveryManager>,
    pub monitor: Arc<ConnectionMonitorManager>,
    pub state_sync: Arc<StateSyncManager>,
    pub health: Arc<HealthManager>,
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
            reconnect: Arc::new(ReconnectManager::default()),
            recovery: Arc::new(SessionRecoveryManager::default()),
            monitor: Arc::new(ConnectionMonitorManager::default()),
            state_sync: Arc::new(StateSyncManager::default()),
            health: Arc::new(HealthManager::default()),
        }
    }
}
