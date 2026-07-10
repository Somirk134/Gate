use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::error::{StatisticsError, StatisticsResult};
use super::metrics::{Metric, MetricsProvider};

/// Runtime state of a collector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CollectorState {
    Created,
    Registered,
    Running,
    Stopped,
    Failed,
}

/// Collector contract used by the monitoring center.
pub trait Collector: MetricsProvider + Send {
    /// Unique collector name.
    fn name(&self) -> &str;

    /// Current collector state.
    fn state(&self) -> CollectorState;

    /// Registers collector resources.
    fn register(&mut self) -> StatisticsResult<()>;

    /// Collects point-in-time metrics.
    fn collect(&mut self) -> StatisticsResult<Vec<Metric>>;

    /// Updates or inserts a metric sample.
    fn update(&mut self, metric: Metric) -> StatisticsResult<()>;

    /// Removes a metric by name.
    fn remove(&mut self, name: &str) -> StatisticsResult<Option<Metric>>;

    /// Resets collector internal state.
    fn reset(&mut self) -> StatisticsResult<()>;

    /// Flushes buffered metrics.
    fn flush(&mut self) -> StatisticsResult<Vec<Metric>>;
}

/// In-memory collector registry for runtime metric collectors.
#[derive(Default)]
pub struct CollectorRegistry {
    collectors: HashMap<String, Box<dyn Collector>>,
}

impl CollectorRegistry {
    /// Creates an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Registers a collector.
    pub fn register(&mut self, mut collector: Box<dyn Collector>) -> StatisticsResult<()> {
        let name = collector.name().to_string();
        if self.collectors.contains_key(&name) {
            return Err(StatisticsError::CollectorAlreadyRegistered(name));
        }

        tracing::info!(target: "gate::statistics", collector = %name, "Collector start");
        collector.register()?;
        self.collectors.insert(name, collector);
        Ok(())
    }

    /// Removes a collector.
    pub fn remove(&mut self, name: &str) -> StatisticsResult<Box<dyn Collector>> {
        tracing::info!(target: "gate::statistics", collector = %name, "Collector stop");
        self.collectors
            .remove(name)
            .ok_or_else(|| StatisticsError::CollectorNotFound(name.to_string()))
    }

    /// Collects all registered metrics.
    pub fn collect_all(&mut self) -> StatisticsResult<Vec<Metric>> {
        let mut metrics = Vec::new();
        for collector in self.collectors.values_mut() {
            metrics.extend(collector.collect()?);
        }
        Ok(metrics)
    }

    /// Returns registered collector names.
    pub fn names(&self) -> Vec<String> {
        self.collectors.keys().cloned().collect()
    }

    /// Returns collector count.
    pub fn len(&self) -> usize {
        self.collectors.len()
    }

    /// Returns true when the registry is empty.
    pub fn is_empty(&self) -> bool {
        self.collectors.is_empty()
    }
}
