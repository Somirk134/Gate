use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::error::StatisticsResult;
use super::metrics::Metric;
use super::statistics::Statistics;

/// Export formats supported by the monitoring center contract.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Prometheus,
    OpenTelemetry,
}

/// Exported payload descriptor.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExportPayload {
    pub format: ExportFormat,
    pub content_type: String,
    pub bytes: Vec<u8>,
    pub exported_at: DateTime<Utc>,
}

impl ExportPayload {
    /// Creates an export payload.
    pub fn new(format: ExportFormat, content_type: impl Into<String>, bytes: Vec<u8>) -> Self {
        Self {
            format,
            content_type: content_type.into(),
            bytes,
            exported_at: Utc::now(),
        }
    }
}

/// Exporter contract implemented by concrete monitoring integrations.
pub trait Exporter {
    /// Returns the exporter format.
    fn format(&self) -> ExportFormat;

    /// Exports a statistics snapshot.
    fn export_statistics(&self, statistics: &Statistics) -> StatisticsResult<ExportPayload>;

    /// Exports raw metrics.
    fn export_metrics(&self, metrics: &[Metric]) -> StatisticsResult<ExportPayload>;

    /// Flushes any buffered export state.
    fn flush(&mut self) -> StatisticsResult<()>;
}
