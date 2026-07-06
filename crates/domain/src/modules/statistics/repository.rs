use super::dashboard::DashboardData;
use super::error::StatisticsResult;
use super::metrics::Metric;
use super::statistics::Statistics;

/// Repository contract for statistics snapshots.
///
/// This is a contract only. The current phase intentionally does not connect a
/// database or persistent storage.
pub trait StatisticsRepository {
    fn save_snapshot(&mut self, snapshot: Statistics) -> StatisticsResult<()>;

    fn latest_snapshot(&self) -> Option<Statistics>;

    fn list_snapshots(&self) -> Vec<Statistics>;
}

/// Repository contract for raw metrics.
pub trait MetricsRepository {
    fn save_metrics(&mut self, metrics: Vec<Metric>) -> StatisticsResult<()>;

    fn latest_metrics(&self) -> Vec<Metric>;
}

/// Repository contract for dashboard payloads.
pub trait DashboardRepository {
    fn save_dashboard(&mut self, dashboard: DashboardData) -> StatisticsResult<()>;

    fn latest_dashboard(&self) -> Option<DashboardData>;
}
