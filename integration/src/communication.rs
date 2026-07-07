use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Authentication counters collected from the real protocol handshake.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AuthSnapshot {
    pub success: u64,
    pub failure: u64,
    pub active_session: u64,
    pub rejected_connection: u64,
}

/// Heartbeat counters collected from Ping/Pong traffic.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatSnapshot {
    pub ping: u64,
    pub pong: u64,
    pub timeout: u64,
    pub last_rtt_ms: Option<u64>,
    pub average_rtt_ms: f64,
}

impl HeartbeatSnapshot {
    pub fn record_pong(&mut self, rtt: Duration) {
        self.pong += 1;
        let rtt_ms = rtt.as_millis().min(u128::from(u64::MAX)) as u64;
        self.last_rtt_ms = Some(rtt_ms);
        self.average_rtt_ms = if self.pong == 1 {
            rtt_ms as f64
        } else {
            ((self.average_rtt_ms * (self.pong - 1) as f64) + rtt_ms as f64) / self.pong as f64
        };
    }
}

/// Unified Alpha statistics used by integration tests and dashboard snapshots.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AlphaStatistics {
    pub connection_current: u64,
    pub connection_total: u64,
    pub request_total: u64,
    pub response_total: u64,
    pub event_total: u64,
    pub reconnect_total: u64,
    pub disconnect_total: u64,
    pub traffic_upload_bytes: u64,
    pub traffic_download_bytes: u64,
    pub runtime_session_count: u64,
    pub runtime_error_count: u64,
    pub auth: AuthSnapshot,
    pub heartbeat: HeartbeatSnapshot,
}

impl AlphaStatistics {
    pub fn record_connect(&mut self) {
        self.connection_current += 1;
        self.connection_total += 1;
    }

    pub fn record_disconnect(&mut self) {
        self.connection_current = self.connection_current.saturating_sub(1);
        self.disconnect_total += 1;
    }
}
