use super::dashboard::{DashboardData, DashboardProvider};
use super::error::StatisticsResult;
use super::monitor::MonitoringCenter;
use super::statistics::Statistics;

/// Command-like handler used by the application layer.
pub struct StatisticsHandler;

impl StatisticsHandler {
    /// Collects a statistics snapshot through the monitoring center.
    pub fn collect(center: &mut MonitoringCenter) -> StatisticsResult<Statistics> {
        center.collect_once()
    }

    /// Reads dashboard data through the monitoring center.
    pub fn dashboard(center: &MonitoringCenter) -> DashboardData {
        center.dashboard_data()
    }
}
