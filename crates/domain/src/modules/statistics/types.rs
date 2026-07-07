//! Public type aliases and naming conventions for monitoring.

pub use super::dashboard::{
    ConnectionTrendPoint, DashboardData, OverviewStatistics, RealtimeSpeedPoint, RecentActivity,
    TrafficTrendPoint,
};
pub use super::health::{HealthReport, HealthSignal, HealthStatus, HealthTarget};
pub use super::metrics::{
    Metric, MetricDescriptor, MetricKind, MetricScope, MetricUnit, MetricValue,
};
pub use super::statistics::{
    AuthenticationStatistics, ClientStatistics, ConnectionStatistics, HeartbeatStatistics,
    NetworkStatistics, ProjectStatistics, RuntimeStatistics, ServerStatistics, Statistics,
    SystemStatistics, TrafficStatistics, TunnelStatistics,
};

/// Canonical metric name prefix.
pub const METRIC_PREFIX: &str = "gate";

/// Naming convention used by this module.
///
/// Format: `gate.<scope>.<resource>.<measurement>[.<unit>]`.
pub const METRIC_NAMING_CONVENTION: &str = "gate.<scope>.<resource>.<measurement>[.<unit>]";
