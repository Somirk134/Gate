use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::sampler::SamplingInterval;

/// Collector configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CollectorConfig {
    pub enabled: bool,
    pub namespace: String,
    pub allow_plugins: bool,
    pub flush_interval: Duration,
}

impl Default for CollectorConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            namespace: "gate".to_string(),
            allow_plugins: true,
            flush_interval: Duration::from_secs(30),
        }
    }
}

/// Builder for collector configuration.
#[derive(Debug, Clone, Default)]
pub struct CollectorConfigBuilder {
    config: CollectorConfig,
}

impl CollectorConfigBuilder {
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    pub fn namespace(mut self, namespace: impl Into<String>) -> Self {
        self.config.namespace = namespace.into();
        self
    }

    pub fn allow_plugins(mut self, allow_plugins: bool) -> Self {
        self.config.allow_plugins = allow_plugins;
        self
    }

    pub fn flush_interval(mut self, flush_interval: Duration) -> Self {
        self.config.flush_interval = flush_interval;
        self
    }

    pub fn build(self) -> CollectorConfig {
        self.config
    }
}

/// Sampling configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SamplingConfig {
    pub enabled: bool,
    pub interval: SamplingInterval,
    pub jitter: Duration,
    pub max_samples: usize,
}

impl Default for SamplingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: SamplingInterval::FiveSeconds,
            jitter: Duration::from_millis(250),
            max_samples: 720,
        }
    }
}

/// Builder for sampling configuration.
#[derive(Debug, Clone, Default)]
pub struct SamplingConfigBuilder {
    config: SamplingConfig,
}

impl SamplingConfigBuilder {
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    pub fn interval(mut self, interval: SamplingInterval) -> Self {
        self.config.interval = interval;
        self
    }

    pub fn jitter(mut self, jitter: Duration) -> Self {
        self.config.jitter = jitter;
        self
    }

    pub fn max_samples(mut self, max_samples: usize) -> Self {
        self.config.max_samples = max_samples;
        self
    }

    pub fn build(self) -> SamplingConfig {
        self.config
    }
}

/// Dashboard configuration.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DashboardConfig {
    pub enabled: bool,
    pub refresh_interval: Duration,
    pub realtime_points: usize,
    pub trend_points: usize,
}

impl Default for DashboardConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            refresh_interval: Duration::from_secs(5),
            realtime_points: 60,
            trend_points: 24,
        }
    }
}

/// Builder for dashboard configuration.
#[derive(Debug, Clone, Default)]
pub struct DashboardConfigBuilder {
    config: DashboardConfig,
}

impl DashboardConfigBuilder {
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    pub fn refresh_interval(mut self, refresh_interval: Duration) -> Self {
        self.config.refresh_interval = refresh_interval;
        self
    }

    pub fn realtime_points(mut self, realtime_points: usize) -> Self {
        self.config.realtime_points = realtime_points;
        self
    }

    pub fn trend_points(mut self, trend_points: usize) -> Self {
        self.config.trend_points = trend_points;
        self
    }

    pub fn build(self) -> DashboardConfig {
        self.config
    }
}

/// Alert configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlertConfig {
    pub enabled: bool,
    pub cpu_high_percent: f64,
    pub memory_high_percent: f64,
    pub traffic_overflow_bytes: u64,
    pub heartbeat_timeout: Duration,
}

impl Default for AlertConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cpu_high_percent: 85.0,
            memory_high_percent: 90.0,
            traffic_overflow_bytes: 100 * 1024 * 1024 * 1024,
            heartbeat_timeout: Duration::from_secs(15),
        }
    }
}

/// Builder for alert configuration.
#[derive(Debug, Clone, Default)]
pub struct AlertConfigBuilder {
    config: AlertConfig,
}

impl AlertConfigBuilder {
    pub fn enabled(mut self, enabled: bool) -> Self {
        self.config.enabled = enabled;
        self
    }

    pub fn cpu_high_percent(mut self, value: f64) -> Self {
        self.config.cpu_high_percent = value;
        self
    }

    pub fn memory_high_percent(mut self, value: f64) -> Self {
        self.config.memory_high_percent = value;
        self
    }

    pub fn traffic_overflow_bytes(mut self, value: u64) -> Self {
        self.config.traffic_overflow_bytes = value;
        self
    }

    pub fn heartbeat_timeout(mut self, value: Duration) -> Self {
        self.config.heartbeat_timeout = value;
        self
    }

    pub fn build(self) -> AlertConfig {
        self.config
    }
}

/// Root statistics center configuration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StatisticsConfig {
    pub collector: CollectorConfig,
    pub sampling: SamplingConfig,
    pub dashboard: DashboardConfig,
    pub alert: AlertConfig,
}

impl Default for StatisticsConfig {
    fn default() -> Self {
        Self {
            collector: CollectorConfig::default(),
            sampling: SamplingConfig::default(),
            dashboard: DashboardConfig::default(),
            alert: AlertConfig::default(),
        }
    }
}

/// Builder for the root statistics center configuration.
#[derive(Debug, Clone, Default)]
pub struct StatisticsConfigBuilder {
    config: StatisticsConfig,
}

impl StatisticsConfigBuilder {
    pub fn collector(mut self, collector: CollectorConfig) -> Self {
        self.config.collector = collector;
        self
    }

    pub fn sampling(mut self, sampling: SamplingConfig) -> Self {
        self.config.sampling = sampling;
        self
    }

    pub fn dashboard(mut self, dashboard: DashboardConfig) -> Self {
        self.config.dashboard = dashboard;
        self
    }

    pub fn alert(mut self, alert: AlertConfig) -> Self {
        self.config.alert = alert;
        self
    }

    pub fn build(self) -> StatisticsConfig {
        self.config
    }
}
