use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::health::HealthReport;
use super::statistics::{
    ConnectionStatistics, RuntimeStatistics, SystemStatistics, TrafficStatistics,
    TunnelStatistics,
};

/// Overview values shown at the top of the monitoring dashboard.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct OverviewStatistics {
    pub tunnel_count: u64,
    pub running_tunnel: u64,
    pub current_connection: u64,
    pub today_traffic: u64,
    pub total_traffic: u64,
    pub average_rtt_ms: f64,
    pub runtime_uptime_seconds: u64,
    pub health_score: f64,
}

/// Realtime speed chart point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealtimeSpeedPoint {
    pub timestamp: DateTime<Utc>,
    pub upload_bps: f64,
    pub download_bps: f64,
}

/// Connection trend chart point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ConnectionTrendPoint {
    pub timestamp: DateTime<Utc>,
    pub current: u64,
    pub success: u64,
    pub failure: u64,
    pub reconnect: u64,
}

/// Traffic trend chart point.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrafficTrendPoint {
    pub timestamp: DateTime<Utc>,
    pub upload_bytes: u64,
    pub download_bytes: u64,
}

/// Generic status bucket used by tunnel and server panels.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct StatusBucket {
    pub label: String,
    pub count: u64,
}

/// Server status summary for the dashboard.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServerStatusSummary {
    pub online: u64,
    pub warning: u64,
    pub offline: u64,
}

/// Recent activity item.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecentActivity {
    pub id: String,
    pub title: String,
    pub category: String,
    pub timestamp: DateTime<Utc>,
}

/// Unified dashboard payload.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DashboardData {
    pub overview: OverviewStatistics,
    pub traffic: TrafficStatistics,
    pub realtime_speed: Vec<RealtimeSpeedPoint>,
    pub tunnel_status: Vec<StatusBucket>,
    pub tunnel: TunnelStatistics,
    pub server_status: ServerStatusSummary,
    pub system_health: HealthReport,
    pub system: SystemStatistics,
    pub connection: ConnectionStatistics,
    pub connection_trend: Vec<ConnectionTrendPoint>,
    pub traffic_trend: Vec<TrafficTrendPoint>,
    pub runtime: RuntimeStatistics,
    pub recent_activity: Vec<RecentActivity>,
    pub generated_at: DateTime<Utc>,
}

impl Default for DashboardData {
    fn default() -> Self {
        Self {
            overview: OverviewStatistics::default(),
            traffic: TrafficStatistics::default(),
            realtime_speed: Vec::new(),
            tunnel_status: Vec::new(),
            tunnel: TunnelStatistics::default(),
            server_status: ServerStatusSummary::default(),
            system_health: HealthReport::default(),
            system: SystemStatistics::default(),
            connection: ConnectionStatistics::default(),
            connection_trend: Vec::new(),
            traffic_trend: Vec::new(),
            runtime: RuntimeStatistics::default(),
            recent_activity: Vec::new(),
            generated_at: Utc::now(),
        }
    }
}

/// Trait implemented by dashboard data providers.
pub trait DashboardProvider {
    /// Returns a complete dashboard snapshot.
    fn dashboard_data(&self) -> DashboardData;
}
