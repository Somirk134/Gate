use thiserror::Error;

/// Domain-level result for monitoring and statistics operations.
pub type StatisticsResult<T> = Result<T, StatisticsError>;

/// Errors produced by the monitoring center contracts.
#[derive(Debug, Error)]
pub enum StatisticsError {
    /// A collector with the same name already exists.
    #[error("collector already registered: {0}")]
    CollectorAlreadyRegistered(String),

    /// A collector could not be found.
    #[error("collector not found: {0}")]
    CollectorNotFound(String),

    /// A metric could not be found.
    #[error("metric not found: {0}")]
    MetricNotFound(String),

    /// A metric update was rejected because the value is invalid.
    #[error("invalid metric value for {name}: {reason}")]
    InvalidMetricValue { name: String, reason: String },

    /// An exporter was asked to emit an unsupported format.
    #[error("unsupported export format: {0:?}")]
    UnsupportedExportFormat(crate::modules::statistics::exporter::ExportFormat),

    /// The sampler configuration is not valid.
    #[error("invalid sampling configuration: {0}")]
    InvalidSamplingConfig(String),

    /// A generic operation failed.
    #[error("statistics operation failed: {0}")]
    OperationFailed(String),
}
