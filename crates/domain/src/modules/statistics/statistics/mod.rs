use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Traffic statistics shared by tunnel, server, client, and dashboard views.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TrafficStatistics {
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub upload_speed_bps: f64,
    pub download_speed_bps: f64,
    pub peak_speed_bps: f64,
    pub average_speed_bps: f64,
    pub today_traffic_bytes: u64,
    pub total_traffic_bytes: u64,
}

/// Tunnel statistics.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TunnelStatistics {
    pub tunnel_count: u64,
    pub running_tunnel: u64,
    pub stopped_tunnel: u64,
    pub upload: u64,
    pub download: u64,
    pub peak_speed_bps: f64,
    pub average_speed_bps: f64,
    pub running_time_seconds: u64,
    pub today_traffic: u64,
    pub total_traffic: u64,
}

/// Server statistics.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ServerStatistics {
    pub server_count: u64,
    pub online_server: u64,
    pub offline_server: u64,
    pub warning_server: u64,
    pub average_rtt_ms: f64,
    pub traffic: TrafficStatistics,
}

/// Connection statistics.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ConnectionStatistics {
    pub current_connection: u64,
    pub total_connection: u64,
    pub success: u64,
    pub failure: u64,
    pub reconnect: u64,
    pub disconnect: u64,
    pub connection_duration_ms: u64,
    pub average_rtt_ms: f64,
}

/// Runtime statistics.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RuntimeStatistics {
    pub running_task: u64,
    pub worker_count: u64,
    pub scheduler_queue: u64,
    pub buffer_usage: f64,
    pub session_count: u64,
    pub runtime_uptime_seconds: u64,
}

/// Heartbeat statistics.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct HeartbeatStatistics {
    pub heartbeat_count: u64,
    pub timeout_count: u64,
    pub last_rtt_ms: Option<u64>,
    pub average_rtt_ms: f64,
    pub missed_heartbeat: u64,
}

/// Authentication statistics.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct AuthenticationStatistics {
    pub login_success: u64,
    pub login_failure: u64,
    pub token_refresh: u64,
    pub active_session: u64,
    pub rejected_request: u64,
}

/// Project statistics.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ProjectStatistics {
    pub project_count: u64,
    pub open_project: u64,
    pub active_project: u64,
    pub current_workspace: Option<String>,
}

/// System statistics.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct SystemStatistics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: Option<f64>,
    pub thread_count: u64,
    pub process_uptime_seconds: u64,
    pub open_file: Option<u64>,
}

/// Network statistics.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct NetworkStatistics {
    pub ingress_bps: f64,
    pub egress_bps: f64,
    pub packet_loss: f64,
    pub latency_ms: f64,
    pub traffic: TrafficStatistics,
}

/// Client statistics.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ClientStatistics {
    pub online_time_seconds: u64,
    pub open_project: u64,
    pub current_workspace: Option<String>,
    pub ui_fps: Option<f64>,
    pub memory_bytes: u64,
}

/// Unified statistics snapshot for the whole system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Statistics {
    pub collected_at: DateTime<Utc>,
    pub tunnel: TunnelStatistics,
    pub server: ServerStatistics,
    pub connection: ConnectionStatistics,
    pub runtime: RuntimeStatistics,
    pub heartbeat: HeartbeatStatistics,
    pub authentication: AuthenticationStatistics,
    pub project: ProjectStatistics,
    pub system: SystemStatistics,
    pub network: NetworkStatistics,
    pub client: ClientStatistics,
}

impl Default for Statistics {
    fn default() -> Self {
        Self {
            collected_at: Utc::now(),
            tunnel: TunnelStatistics::default(),
            server: ServerStatistics::default(),
            connection: ConnectionStatistics::default(),
            runtime: RuntimeStatistics::default(),
            heartbeat: HeartbeatStatistics::default(),
            authentication: AuthenticationStatistics::default(),
            project: ProjectStatistics::default(),
            system: SystemStatistics::default(),
            network: NetworkStatistics::default(),
            client: ClientStatistics::default(),
        }
    }
}
