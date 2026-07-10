use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Category of a metric instrument.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricKind {
    /// Monotonic increasing value.
    Counter,
    /// Point-in-time value.
    Gauge,
    /// Distribution bucket metric.
    Histogram,
    /// Derived per-second value.
    Rate,
    /// Derived average value.
    Average,
    /// Highest observed value.
    Peak,
    /// Lowest observed value.
    Min,
    /// Highest observed value for a window.
    Max,
}

/// Logical metric scope used by the unified naming scheme.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricScope {
    Tunnel,
    Project,
    Server,
    Connection,
    Heartbeat,
    Authentication,
    Runtime,
    System,
    Network,
    Client,
    Custom(String),
}

/// Metric unit metadata.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MetricUnit {
    Count,
    Bytes,
    BytesPerSecond,
    Milliseconds,
    Seconds,
    Percent,
    Ratio,
    OperationsPerSecond,
    Custom(String),
}

/// Metric label.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MetricLabel {
    pub key: String,
    pub value: String,
}

impl MetricLabel {
    /// Creates a new metric label.
    pub fn new(key: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
        }
    }
}

/// Metric value representation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum MetricValue {
    Integer(i64),
    Unsigned(u64),
    Float(f64),
    Distribution(Vec<f64>),
}

impl MetricValue {
    /// Converts numeric values to f64 for aggregation.
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Self::Integer(value) => Some(*value as f64),
            Self::Unsigned(value) => Some(*value as f64),
            Self::Float(value) => Some(*value),
            Self::Distribution(_) => None,
        }
    }
}

/// Stable metadata for an instrument.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricDescriptor {
    pub name: String,
    pub description: String,
    pub scope: MetricScope,
    pub unit: MetricUnit,
    pub labels: Vec<MetricLabel>,
}

impl MetricDescriptor {
    /// Creates a metric descriptor using the Gate naming convention.
    pub fn new(
        scope: MetricScope,
        name: impl Into<String>,
        unit: MetricUnit,
        description: impl Into<String>,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            scope,
            unit,
            labels: Vec::new(),
        }
    }

    /// Adds a label to the descriptor.
    pub fn with_label(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.labels.push(MetricLabel::new(key, value));
        self
    }
}

/// Immutable metric sample emitted by collectors.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Metric {
    pub descriptor: MetricDescriptor,
    pub kind: MetricKind,
    pub value: MetricValue,
    pub timestamp: DateTime<Utc>,
}

impl Metric {
    /// Creates a new metric sample at the current timestamp.
    pub fn new(descriptor: MetricDescriptor, kind: MetricKind, value: MetricValue) -> Self {
        Self {
            descriptor,
            kind,
            value,
            timestamp: Utc::now(),
        }
    }

    /// Returns the canonical metric name.
    pub fn name(&self) -> &str {
        &self.descriptor.name
    }
}

/// Shared interface implemented by all metric instruments.
pub trait MetricInstrument {
    /// Returns static instrument metadata.
    fn descriptor(&self) -> &MetricDescriptor;

    /// Returns the instrument category.
    fn kind(&self) -> MetricKind;

    /// Records a new numeric observation.
    fn record(&mut self, value: f64);

    /// Exports the current instrument state as an immutable metric sample.
    fn snapshot(&self) -> Metric;

    /// Clears the instrument state.
    fn reset(&mut self);
}

/// A source that can expose a point-in-time metric set.
pub trait MetricsProvider {
    /// Returns all metrics from the provider.
    fn metrics(&self) -> Vec<Metric>;

    /// Finds a metric by canonical name.
    fn metric(&self, name: &str) -> Option<Metric> {
        self.metrics()
            .into_iter()
            .find(|metric| metric.descriptor.name == name)
    }
}

macro_rules! scalar_metric {
    ($name:ident, $kind:expr, $record:expr) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        pub struct $name {
            descriptor: MetricDescriptor,
            value: f64,
        }

        impl $name {
            /// Creates a scalar metric instrument.
            pub fn new(descriptor: MetricDescriptor) -> Self {
                Self {
                    descriptor,
                    value: 0.0,
                }
            }

            /// Returns the raw scalar value.
            pub fn value(&self) -> f64 {
                self.value
            }
        }

        impl MetricInstrument for $name {
            fn descriptor(&self) -> &MetricDescriptor {
                &self.descriptor
            }

            fn kind(&self) -> MetricKind {
                $kind
            }

            fn record(&mut self, value: f64) {
                self.value = $record(self.value, value);
            }

            fn snapshot(&self) -> Metric {
                Metric::new(
                    self.descriptor.clone(),
                    self.kind(),
                    MetricValue::Float(self.value),
                )
            }

            fn reset(&mut self) {
                self.value = 0.0;
            }
        }
    };
}

scalar_metric!(
    CounterMetric,
    MetricKind::Counter,
    |current: f64, incoming: f64| { current + incoming.max(0.0) }
);

scalar_metric!(
    GaugeMetric,
    MetricKind::Gauge,
    |_current: f64, incoming: f64| incoming
);

scalar_metric!(
    RateMetric,
    MetricKind::Rate,
    |_current: f64, incoming: f64| incoming
);

scalar_metric!(
    AverageMetric,
    MetricKind::Average,
    |current: f64, incoming: f64| {
        if current == 0.0 {
            incoming
        } else {
            (current + incoming) / 2.0
        }
    }
);

scalar_metric!(
    PeakMetric,
    MetricKind::Peak,
    |current: f64, incoming: f64| { current.max(incoming) }
);

scalar_metric!(MinMetric, MetricKind::Min, |current: f64, incoming: f64| {
    if current == 0.0 {
        incoming
    } else {
        current.min(incoming)
    }
});

scalar_metric!(MaxMetric, MetricKind::Max, |current: f64, incoming: f64| {
    current.max(incoming)
});

/// Histogram instrument with lightweight bucket storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistogramMetric {
    descriptor: MetricDescriptor,
    samples: Vec<f64>,
}

impl HistogramMetric {
    /// Creates a histogram metric instrument.
    pub fn new(descriptor: MetricDescriptor) -> Self {
        Self {
            descriptor,
            samples: Vec::new(),
        }
    }

    /// Returns observed samples.
    pub fn samples(&self) -> &[f64] {
        &self.samples
    }
}

impl MetricInstrument for HistogramMetric {
    fn descriptor(&self) -> &MetricDescriptor {
        &self.descriptor
    }

    fn kind(&self) -> MetricKind {
        MetricKind::Histogram
    }

    fn record(&mut self, value: f64) {
        self.samples.push(value);
    }

    fn snapshot(&self) -> Metric {
        Metric::new(
            self.descriptor.clone(),
            MetricKind::Histogram,
            MetricValue::Distribution(self.samples.clone()),
        )
    }

    fn reset(&mut self) {
        self.samples.clear();
    }
}
