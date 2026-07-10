//! Runtime statistics models for tunnels and traffic.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Aggregated tunnel statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TunnelStatistics {
    pub connection_count: u64,
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub packets: u64,
    pub errors: u64,
    pub running_time: Duration,
    pub today: TrafficSnapshot,
    pub total: TrafficSnapshot,
}

/// Traffic snapshot for a time range.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TrafficSnapshot {
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub packets: u64,
    pub errors: u64,
}

/// Unified stability metrics shared by heartbeat, reconnect, recovery, and
/// connection monitoring views.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StabilityMetrics {
    pub ping_count: u64,
    pub average_rtt_ms: f64,
    pub last_rtt_ms: Option<u64>,
    pub reconnect_count: u64,
    pub recovery_time_ms: u64,
    pub timeout_count: u64,
    pub heartbeat_count: u64,
    pub connection_duration_ms: u64,
}
