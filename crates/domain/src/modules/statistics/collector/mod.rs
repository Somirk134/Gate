use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::error::{StatisticsError, StatisticsResult};
use super::metrics::{
    Metric, MetricDescriptor, MetricKind, MetricScope, MetricUnit, MetricValue, MetricsProvider,
};

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

/// In-memory collector registry with future plugin support.
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

/// Mock collector used until real module collectors are wired in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockMetricsCollector {
    name: String,
    state: CollectorState,
    metrics: HashMap<String, Metric>,
}

impl MockMetricsCollector {
    /// Creates a mock collector.
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            state: CollectorState::Created,
            metrics: HashMap::new(),
        }
    }

    /// Creates a mock collector with baseline system metrics.
    pub fn with_system_defaults() -> Self {
        let mut collector = Self::new("mock.system");
        let cpu = Metric::new(
            MetricDescriptor::new(
                MetricScope::System,
                "gate.system.cpu.usage",
                MetricUnit::Percent,
                "Mock CPU usage",
            ),
            MetricKind::Gauge,
            MetricValue::Float(32.0),
        );
        let memory = Metric::new(
            MetricDescriptor::new(
                MetricScope::System,
                "gate.system.memory.usage",
                MetricUnit::Percent,
                "Mock memory usage",
            ),
            MetricKind::Gauge,
            MetricValue::Float(58.0),
        );
        collector.metrics.insert(cpu.name().to_string(), cpu);
        collector.metrics.insert(memory.name().to_string(), memory);
        collector
    }
}

impl MetricsProvider for MockMetricsCollector {
    fn metrics(&self) -> Vec<Metric> {
        self.metrics.values().cloned().collect()
    }
}

impl Collector for MockMetricsCollector {
    fn name(&self) -> &str {
        &self.name
    }

    fn state(&self) -> CollectorState {
        self.state
    }

    fn register(&mut self) -> StatisticsResult<()> {
        self.state = CollectorState::Registered;
        Ok(())
    }

    fn collect(&mut self) -> StatisticsResult<Vec<Metric>> {
        self.state = CollectorState::Running;
        tracing::debug!(target: "gate::statistics", collector = %self.name, "Metrics collected");
        Ok(self.metrics())
    }

    fn update(&mut self, metric: Metric) -> StatisticsResult<()> {
        self.metrics.insert(metric.name().to_string(), metric);
        Ok(())
    }

    fn remove(&mut self, name: &str) -> StatisticsResult<Option<Metric>> {
        Ok(self.metrics.remove(name))
    }

    fn reset(&mut self) -> StatisticsResult<()> {
        self.metrics.clear();
        self.state = CollectorState::Created;
        Ok(())
    }

    fn flush(&mut self) -> StatisticsResult<Vec<Metric>> {
        Ok(self.metrics.values().cloned().collect())
    }
}
