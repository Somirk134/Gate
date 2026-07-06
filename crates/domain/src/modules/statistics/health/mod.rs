use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Health state emitted by monitoring providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Offline,
}

impl HealthStatus {
    /// Returns a sortable severity score.
    pub fn severity(self) -> u8 {
        match self {
            Self::Healthy => 0,
            Self::Warning => 1,
            Self::Critical => 2,
            Self::Offline => 3,
        }
    }
}

/// Health target supported by the center.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthTarget {
    Tunnel,
    Connection,
    Runtime,
    Heartbeat,
    Server,
    System,
    Custom(String),
}

/// Health signal from a module.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthSignal {
    pub target: HealthTarget,
    pub status: HealthStatus,
    pub message: String,
    pub score: f64,
    pub timestamp: DateTime<Utc>,
}

impl HealthSignal {
    /// Creates a new health signal.
    pub fn new(
        target: HealthTarget,
        status: HealthStatus,
        message: impl Into<String>,
        score: f64,
    ) -> Self {
        Self {
            target,
            status,
            message: message.into(),
            score,
            timestamp: Utc::now(),
        }
    }
}

/// Aggregated health report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HealthReport {
    pub overall: HealthStatus,
    pub signals: Vec<HealthSignal>,
    pub updated_at: DateTime<Utc>,
}

impl Default for HealthReport {
    fn default() -> Self {
        Self {
            overall: HealthStatus::Healthy,
            signals: Vec::new(),
            updated_at: Utc::now(),
        }
    }
}

/// Trait implemented by modules that can publish health signals.
pub trait HealthProvider {
    /// Returns the latest health signal.
    fn health_signal(&self) -> HealthSignal;
}

/// Central health state manager.
#[derive(Debug, Clone, Default)]
pub struct HealthCenter {
    signals: HashMap<HealthTarget, HealthSignal>,
}

impl HealthCenter {
    /// Creates an empty health center.
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates a target health signal.
    pub fn update(&mut self, signal: HealthSignal) {
        self.signals.insert(signal.target.clone(), signal);
    }

    /// Removes a target from the report.
    pub fn remove(&mut self, target: &HealthTarget) -> Option<HealthSignal> {
        self.signals.remove(target)
    }

    /// Returns the aggregated health report.
    pub fn report(&self) -> HealthReport {
        let mut signals = self.signals.values().cloned().collect::<Vec<_>>();
        signals.sort_by_key(|signal| signal.status.severity());
        let overall = signals
            .iter()
            .map(|signal| signal.status)
            .max_by_key(|status| status.severity())
            .unwrap_or(HealthStatus::Healthy);

        HealthReport {
            overall,
            signals,
            updated_at: Utc::now(),
        }
    }

    /// Clears all health signals.
    pub fn reset(&mut self) {
        self.signals.clear();
    }
}
