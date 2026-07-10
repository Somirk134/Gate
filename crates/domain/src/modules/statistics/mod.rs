//! Monitoring and statistics center domain contracts.
//!
//! This module is intentionally infrastructure-free. It defines the contracts,
//! value objects, configuration, events, and runtime boundaries used by tunnel,
//! server, runtime, authentication, and dashboard modules.

pub mod aggregator;
pub mod alert;
pub mod collector;
pub mod config;
pub mod dashboard;
pub mod entity;
pub mod error;
pub mod event;
pub mod exporter;
pub mod handler;
pub mod health;
pub mod metrics;
pub mod monitor;
pub mod repository;
pub mod sampler;
pub mod service;
pub mod statistics;
pub mod types;

pub use aggregator::{AggregationPeriod, AggregationWindow, Aggregator, StatisticsAggregator};
pub use alert::{AlertEvent, AlertKind, AlertManager, AlertRule, AlertSeverity};
pub use collector::{Collector, CollectorRegistry, CollectorState};
pub use config::{
    AlertConfig, AlertConfigBuilder, CollectorConfig, CollectorConfigBuilder, DashboardConfig,
    DashboardConfigBuilder, SamplingConfig, SamplingConfigBuilder, StatisticsConfig,
    StatisticsConfigBuilder,
};
pub use dashboard::{
    ConnectionTrendPoint, DashboardData, DashboardProvider, OverviewStatistics, RealtimeSpeedPoint,
    RecentActivity, ServerStatusSummary, StatusBucket, TrafficTrendPoint,
};
pub use entity::*;
pub use error::*;
pub use event::*;
pub use exporter::{ExportFormat, ExportPayload, Exporter};
pub use health::{
    HealthCenter, HealthProvider, HealthReport, HealthSignal, HealthStatus, HealthTarget,
};
pub use metrics::{
    AverageMetric, CounterMetric, GaugeMetric, HistogramMetric, MaxMetric, Metric,
    MetricDescriptor, MetricInstrument, MetricKind, MetricLabel, MetricScope, MetricUnit,
    MetricValue, MetricsProvider, MinMetric, PeakMetric, RateMetric,
};
pub use monitor::{MonitorLifecycleState, MonitoringCenter};
pub use repository::*;
pub use sampler::{Sampler, SamplingInterval, SamplingStrategy, StatisticsSampler};
pub use service::*;
pub use statistics::*;
pub use types::{METRIC_NAMING_CONVENTION, METRIC_PREFIX};
