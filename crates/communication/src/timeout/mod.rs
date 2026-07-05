//! Timeout policy shared by requests, transport, and connection lifecycle.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Communication timeout categories.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TimeoutKind {
    Request,
    Heartbeat,
    Connection,
    Read,
    Write,
}

/// Unified timeout configuration expressed in milliseconds for stable config IO.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeoutConfig {
    pub request_timeout_ms: u64,
    pub heartbeat_timeout_ms: u64,
    pub connection_timeout_ms: u64,
    pub read_timeout_ms: u64,
    pub write_timeout_ms: u64,
}

impl TimeoutConfig {
    pub fn timeout_for(&self, kind: TimeoutKind) -> Duration {
        Duration::from_millis(match kind {
            TimeoutKind::Request => self.request_timeout_ms,
            TimeoutKind::Heartbeat => self.heartbeat_timeout_ms,
            TimeoutKind::Connection => self.connection_timeout_ms,
            TimeoutKind::Read => self.read_timeout_ms,
            TimeoutKind::Write => self.write_timeout_ms,
        })
    }
}

impl Default for TimeoutConfig {
    fn default() -> Self {
        Self {
            request_timeout_ms: 30_000,
            heartbeat_timeout_ms: 15_000,
            connection_timeout_ms: 10_000,
            read_timeout_ms: 30_000,
            write_timeout_ms: 30_000,
        }
    }
}
