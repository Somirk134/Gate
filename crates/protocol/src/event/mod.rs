use serde::{Deserialize, Serialize};

/// Unified protocol event names. Business payloads are carried by `Body`.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProtocolEvent {
    TunnelStarted,
    TunnelStopped,
    TunnelUpdated,
    ServerConnected,
    ServerDisconnected,
    ServerStatusChanged,
    LogReceived,
    StatisticsUpdated,
    ProjectCreated,
    ProjectDeleted,
    HeartbeatTimeout,
    ConfigurationChanged,
    Custom(String),
}
