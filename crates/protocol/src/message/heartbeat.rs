use serde::{Deserialize, Serialize};

/// Heartbeat ping payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ping {
    pub nonce: String,
}

impl Default for Ping {
    fn default() -> Self {
        Self {
            nonce: "v1".to_owned(),
        }
    }
}

/// Heartbeat pong payload.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Pong {
    pub nonce: String,
}

impl Default for Pong {
    fn default() -> Self {
        Self {
            nonce: "v1".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatInterval(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatTimeout(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReconnectInterval(pub u64);

/// Protocolized heartbeat timing knobs in milliseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeartbeatConfig {
    pub interval: HeartbeatInterval,
    pub timeout: HeartbeatTimeout,
    pub reconnect_interval: ReconnectInterval,
}

impl Default for HeartbeatConfig {
    fn default() -> Self {
        Self {
            interval: HeartbeatInterval(30_000),
            timeout: HeartbeatTimeout(90_000),
            reconnect_interval: ReconnectInterval(5_000),
        }
    }
}
