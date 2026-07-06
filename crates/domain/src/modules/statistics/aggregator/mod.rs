use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::error::StatisticsResult;
use super::metrics::Metric;
use super::statistics::Statistics;

/// Supported aggregation periods.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AggregationPeriod {
    Realtime,
    Minute,
    Hour,
    Day,
    History,
}

/// Aggregation window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AggregationWindow {
    pub period: AggregationPeriod,
    pub started_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub samples: Vec<Statistics>,
}

impl AggregationWindow {
    /// Creates an empty aggregation window.
    pub fn new(period: AggregationPeriod) -> Self {
        let now = Utc::now();
        Self {
            period,
            started_at: now,
            updated_at: now,
            samples: Vec::new(),
        }
    }

    /// Adds a sample to the window.
    pub fn push(&mut self, statistics: Statistics) {
        self.updated_at = Utc::now();
        self.samples.push(statistics);
    }

    /// Returns the latest sample.
    pub fn latest(&self) -> Option<&Statistics> {
        self.samples.last()
    }
}

/// Aggregator contract.
pub trait Aggregator {
    /// Ingests metrics from collectors.
    fn ingest_metrics(&mut self, metrics: Vec<Metric>) -> StatisticsResult<Statistics>;

    /// Ingests a full statistics snapshot.
    fn ingest_statistics(&mut self, statistics: Statistics) -> StatisticsResult<()>;

    /// Returns a period snapshot.
    fn snapshot(&self, period: AggregationPeriod) -> Option<Statistics>;

    /// Resets all windows.
    fn reset(&mut self);
}

/// Statistics aggregator used by the monitoring center.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatisticsAggregator {
    realtime: AggregationWindow,
    minute: AggregationWindow,
    hour: AggregationWindow,
    day: AggregationWindow,
    history: AggregationWindow,
    history_enabled: bool,
}

impl StatisticsAggregator {
    /// Creates a new aggregator.
    pub fn new(history_enabled: bool) -> Self {
        Self {
            realtime: AggregationWindow::new(AggregationPeriod::Realtime),
            minute: AggregationWindow::new(AggregationPeriod::Minute),
            hour: AggregationWindow::new(AggregationPeriod::Hour),
            day: AggregationWindow::new(AggregationPeriod::Day),
            history: AggregationWindow::new(AggregationPeriod::History),
            history_enabled,
        }
    }

    /// Returns a window by period.
    pub fn window(&self, period: AggregationPeriod) -> &AggregationWindow {
        match period {
            AggregationPeriod::Realtime => &self.realtime,
            AggregationPeriod::Minute => &self.minute,
            AggregationPeriod::Hour => &self.hour,
            AggregationPeriod::Day => &self.day,
            AggregationPeriod::History => &self.history,
        }
    }

    fn apply_known_metric(statistics: &mut Statistics, metric: &Metric) {
        let Some(value) = metric.value.as_f64() else {
            return;
        };

        match metric.name() {
            "gate.system.cpu.usage" => statistics.system.cpu_usage = value,
            "gate.system.memory.usage" => statistics.system.memory_usage = value,
            "gate.connection.current" => statistics.connection.current_connection = value as u64,
            "gate.connection.rtt.average" => statistics.connection.average_rtt_ms = value,
            "gate.runtime.tasks.running" => statistics.runtime.running_task = value as u64,
            "gate.runtime.workers" => statistics.runtime.worker_count = value as u64,
            "gate.tunnel.count" => statistics.tunnel.tunnel_count = value as u64,
            "gate.tunnel.running" => statistics.tunnel.running_tunnel = value as u64,
            "gate.traffic.upload.bps" => {
                statistics.network.egress_bps = value;
                statistics.network.traffic.upload_speed_bps = value;
            }
            "gate.traffic.download.bps" => {
                statistics.network.ingress_bps = value;
                statistics.network.traffic.download_speed_bps = value;
            }
            _ => {}
        }
    }
}

impl Default for StatisticsAggregator {
    fn default() -> Self {
        Self::new(false)
    }
}

impl Aggregator for StatisticsAggregator {
    fn ingest_metrics(&mut self, metrics: Vec<Metric>) -> StatisticsResult<Statistics> {
        let mut statistics = self
            .realtime
            .latest()
            .cloned()
            .unwrap_or_else(Statistics::default);

        for metric in &metrics {
            Self::apply_known_metric(&mut statistics, metric);
        }
        statistics.collected_at = Utc::now();
        self.ingest_statistics(statistics.clone())?;
        Ok(statistics)
    }

    fn ingest_statistics(&mut self, statistics: Statistics) -> StatisticsResult<()> {
        tracing::debug!(target: "gate::statistics", "Aggregation updated");
        self.realtime.samples.clear();
        self.realtime.push(statistics.clone());
        self.minute.push(statistics.clone());
        self.hour.push(statistics.clone());
        self.day.push(statistics.clone());
        if self.history_enabled {
            self.history.push(statistics);
        }
        Ok(())
    }

    fn snapshot(&self, period: AggregationPeriod) -> Option<Statistics> {
        self.window(period).latest().cloned()
    }

    fn reset(&mut self) {
        *self = Self::new(self.history_enabled);
    }
}
