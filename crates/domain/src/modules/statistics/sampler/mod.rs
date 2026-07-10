use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::error::StatisticsResult;
use super::statistics::Statistics;

/// Supported sampling intervals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SamplingInterval {
    OneSecond,
    FiveSeconds,
    TenSeconds,
    ThirtySeconds,
    OneMinute,
}

impl SamplingInterval {
    /// Converts the interval to a standard duration.
    pub fn as_duration(self) -> Duration {
        match self {
            Self::OneSecond => Duration::from_secs(1),
            Self::FiveSeconds => Duration::from_secs(5),
            Self::TenSeconds => Duration::from_secs(10),
            Self::ThirtySeconds => Duration::from_secs(30),
            Self::OneMinute => Duration::from_secs(60),
        }
    }
}

/// Sampling strategy metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SamplingStrategy {
    pub interval: SamplingInterval,
    pub max_samples: usize,
    pub retain_history: bool,
}

impl Default for SamplingStrategy {
    fn default() -> Self {
        Self {
            interval: SamplingInterval::FiveSeconds,
            max_samples: 720,
            retain_history: false,
        }
    }
}

/// Trait implemented by statistics samplers.
pub trait Sampler {
    /// Returns the active sampling strategy.
    fn strategy(&self) -> &SamplingStrategy;

    /// Updates the active sampling strategy.
    fn set_strategy(&mut self, strategy: SamplingStrategy);

    /// Samples a statistics snapshot.
    fn sample(&mut self, statistics: Statistics) -> StatisticsResult<()>;

    /// Flushes the buffered samples.
    fn flush(&mut self) -> StatisticsResult<Vec<Statistics>>;
}

/// In-memory sampler used by the monitoring center.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatisticsSampler {
    strategy: SamplingStrategy,
    samples: Vec<Statistics>,
    last_sampled_at: Option<DateTime<Utc>>,
}

impl StatisticsSampler {
    /// Creates a sampler with the provided strategy.
    pub fn new(strategy: SamplingStrategy) -> Self {
        Self {
            strategy,
            samples: Vec::new(),
            last_sampled_at: None,
        }
    }

    /// Returns buffered samples.
    pub fn samples(&self) -> &[Statistics] {
        &self.samples
    }

    /// Returns the timestamp of the last sampled snapshot.
    pub fn last_sampled_at(&self) -> Option<DateTime<Utc>> {
        self.last_sampled_at.clone()
    }
}

impl Default for StatisticsSampler {
    fn default() -> Self {
        Self::new(SamplingStrategy::default())
    }
}

impl Sampler for StatisticsSampler {
    fn strategy(&self) -> &SamplingStrategy {
        &self.strategy
    }

    fn set_strategy(&mut self, strategy: SamplingStrategy) {
        self.strategy = strategy;
        if self.samples.len() > self.strategy.max_samples {
            let drain = self.samples.len() - self.strategy.max_samples;
            self.samples.drain(0..drain);
        }
    }

    fn sample(&mut self, statistics: Statistics) -> StatisticsResult<()> {
        tracing::debug!(
            target: "gate::statistics",
            interval = ?self.strategy.interval,
            "Sampling statistics snapshot"
        );
        self.samples.push(statistics);
        self.last_sampled_at = Some(Utc::now());
        if self.samples.len() > self.strategy.max_samples {
            self.samples.remove(0);
        }
        Ok(())
    }

    fn flush(&mut self) -> StatisticsResult<Vec<Statistics>> {
        tracing::debug!(target: "gate::statistics", "Flushing sampled statistics");
        Ok(std::mem::take(&mut self.samples))
    }
}
