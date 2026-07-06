use super::dashboard::{DashboardData, DashboardProvider};
use super::error::StatisticsResult;
use super::exporter::{ExportPayload, Exporter};
use super::health::HealthReport;
use super::metrics::Metric;
use super::monitor::MonitoringCenter;
use super::statistics::Statistics;

/// Rust service facade for the monitoring center.
pub struct StatisticsService {
    center: MonitoringCenter,
}

impl StatisticsService {
    /// Creates a statistics service.
    pub fn new(center: MonitoringCenter) -> Self {
        Self { center }
    }

    /// Returns the inner monitoring center.
    pub fn center(&self) -> &MonitoringCenter {
        &self.center
    }

    /// Returns the mutable inner monitoring center.
    pub fn center_mut(&mut self) -> &mut MonitoringCenter {
        &mut self.center
    }

    /// Collects one statistics snapshot.
    pub fn collect_once(&mut self) -> StatisticsResult<Statistics> {
        self.center.collect_once()
    }

    /// Returns dashboard data.
    pub fn dashboard(&self) -> DashboardData {
        self.center.dashboard_data()
    }

    /// Returns the current health report.
    pub fn health(&self) -> HealthReport {
        self.center.health_report()
    }

    /// Exports the current statistics snapshot with a caller-provided exporter.
    pub fn export(&self, exporter: &dyn Exporter) -> StatisticsResult<ExportPayload> {
        exporter.export_statistics(&self.center.statistics())
    }
}

/// Metrics service boundary.
pub trait MetricsService {
    /// Collects raw metric samples.
    fn collect_metrics(&mut self) -> StatisticsResult<Vec<Metric>>;
}

/// Health service boundary.
pub trait HealthService {
    /// Returns current health.
    fn health_report(&self) -> HealthReport;
}

/// Dashboard service boundary.
pub trait DashboardService {
    /// Returns current dashboard payload.
    fn dashboard_data(&self) -> DashboardData;
}

/// Export service boundary.
pub trait ExportService {
    /// Exports current statistics.
    fn export(&self, exporter: &dyn Exporter) -> StatisticsResult<ExportPayload>;
}
