use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::alert::AlertEvent;
use super::health::HealthReport;
use super::metrics::Metric;
use super::statistics::Statistics;

/// Unified event emitted by the monitoring center.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StatisticsEvent {
    StatisticsUpdated {
        statistics: Statistics,
        timestamp: DateTime<Utc>,
    },
    MetricsCollected {
        collector: String,
        metrics: Vec<Metric>,
        timestamp: DateTime<Utc>,
    },
    HealthChanged {
        report: HealthReport,
        timestamp: DateTime<Utc>,
    },
    AlertTriggered {
        event: AlertEvent,
        timestamp: DateTime<Utc>,
    },
    CollectorRegistered {
        collector: String,
        timestamp: DateTime<Utc>,
    },
    CollectorRemoved {
        collector: String,
        timestamp: DateTime<Utc>,
    },
}

impl StatisticsEvent {
    /// Creates a `StatisticsUpdated` event.
    pub fn statistics_updated(statistics: Statistics) -> Self {
        Self::StatisticsUpdated {
            statistics,
            timestamp: Utc::now(),
        }
    }

    /// Creates a `MetricsCollected` event.
    pub fn metrics_collected(collector: impl Into<String>, metrics: Vec<Metric>) -> Self {
        Self::MetricsCollected {
            collector: collector.into(),
            metrics,
            timestamp: Utc::now(),
        }
    }

    /// Creates a `HealthChanged` event.
    pub fn health_changed(report: HealthReport) -> Self {
        Self::HealthChanged {
            report,
            timestamp: Utc::now(),
        }
    }

    /// Creates an `AlertTriggered` event.
    pub fn alert_triggered(event: AlertEvent) -> Self {
        Self::AlertTriggered {
            event,
            timestamp: Utc::now(),
        }
    }
}
