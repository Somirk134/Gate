//! Health aggregation for connection stability.
//!
//! Health checks consume signals from other modules and produce one unified
//! status. They do not perform network probes directly.

use crate::config::HealthConfig;
use crate::core::Tunnel;
use crate::heartbeat::HeartbeatState;
use dashmap::DashMap;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};

/// Unified health status emitted by the stability core.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Offline,
}

/// Component checked by [`HealthManager`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthCheckTarget {
    Connection,
    Heartbeat,
    Authentication,
    Runtime,
    Tunnel,
    Server,
}

/// Single component health signal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthSignal {
    pub target: HealthCheckTarget,
    pub status: HealthStatus,
    pub score: u8,
    pub message: Option<String>,
    pub checked_at_millis: u64,
}

impl HealthSignal {
    pub fn new(
        target: HealthCheckTarget,
        status: HealthStatus,
        score: u8,
        message: Option<String>,
    ) -> Self {
        Self {
            target,
            status,
            score,
            message,
            checked_at_millis: now_millis(),
        }
    }
}

/// Aggregated health report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthReport {
    pub status: HealthStatus,
    pub score: u8,
    pub checked_at_millis: u64,
    pub components: Vec<HealthSignal>,
}

/// Async health checker trait.
pub trait HealthChecker: Send + Sync {
    fn check_all(&self) -> BoxFuture<'_, HealthReport>;
}

/// Health manager that aggregates component signals.
#[derive(Debug)]
pub struct HealthManager {
    config: HealthConfig,
    signals: DashMap<HealthCheckTarget, HealthSignal>,
}

impl Default for HealthManager {
    fn default() -> Self {
        Self::new(HealthConfig::default())
    }
}

impl HealthManager {
    pub fn new(config: HealthConfig) -> Self {
        Self {
            config,
            signals: DashMap::new(),
        }
    }

    pub async fn record_signal(&self, signal: HealthSignal) -> HealthReport {
        self.signals.insert(signal.target, signal);
        self.report().await
    }

    pub async fn check_connection(&self, alive: bool, score: u8) -> HealthSignal {
        let status = if alive {
            self.status_from_score(score)
        } else {
            HealthStatus::Offline
        };
        let signal = HealthSignal::new(HealthCheckTarget::Connection, status, score, None);
        self.signals.insert(signal.target, signal.clone());
        signal
    }

    pub async fn check_heartbeat(&self, state: HeartbeatState) -> HealthSignal {
        let (status, score, message) = match state {
            HeartbeatState::Running | HeartbeatState::WaitingPong => (HealthStatus::Healthy, 100, None),
            HeartbeatState::Retrying => (
                HealthStatus::Warning,
                self.config.warning_score_threshold,
                Some("heartbeat retrying".to_string()),
            ),
            HeartbeatState::Timeout => (
                HealthStatus::Critical,
                self.config.critical_score_threshold,
                Some("heartbeat timeout".to_string()),
            ),
            HeartbeatState::Idle => (HealthStatus::Warning, 70, Some("heartbeat idle".to_string())),
            HeartbeatState::Stopped => {
                (HealthStatus::Offline, 0, Some("heartbeat stopped".to_string()))
            }
        };
        let signal = HealthSignal::new(HealthCheckTarget::Heartbeat, status, score, message);
        self.signals.insert(signal.target, signal.clone());
        signal
    }

    pub async fn check_authentication(&self, authenticated: bool) -> HealthSignal {
        let signal = if authenticated {
            HealthSignal::new(HealthCheckTarget::Authentication, HealthStatus::Healthy, 100, None)
        } else {
            HealthSignal::new(
                HealthCheckTarget::Authentication,
                HealthStatus::Critical,
                0,
                Some("authentication unavailable".to_string()),
            )
        };
        self.signals.insert(signal.target, signal.clone());
        signal
    }

    pub async fn check_runtime(&self, running: bool) -> HealthSignal {
        let signal = if running {
            HealthSignal::new(HealthCheckTarget::Runtime, HealthStatus::Healthy, 100, None)
        } else {
            HealthSignal::new(
                HealthCheckTarget::Runtime,
                HealthStatus::Offline,
                0,
                Some("runtime stopped".to_string()),
            )
        };
        self.signals.insert(signal.target, signal.clone());
        signal
    }

    pub async fn check_tunnel(&self, tunnel: &dyn Tunnel) -> HealthSignal {
        let status = tunnel.health();
        let score = score_from_status(status);
        let signal = HealthSignal::new(HealthCheckTarget::Tunnel, status, score, None);
        self.signals.insert(signal.target, signal.clone());
        signal
    }

    pub async fn check_server(&self, online: bool, score: u8) -> HealthSignal {
        let status = if online {
            self.status_from_score(score)
        } else {
            HealthStatus::Offline
        };
        let signal = HealthSignal::new(HealthCheckTarget::Server, status, score, None);
        self.signals.insert(signal.target, signal.clone());
        signal
    }

    pub async fn report(&self) -> HealthReport {
        let components: Vec<_> = self
            .signals
            .iter()
            .map(|entry| entry.value().clone())
            .collect();
        if components.is_empty() {
            return HealthReport {
                status: HealthStatus::Warning,
                score: self.config.warning_score_threshold,
                checked_at_millis: now_millis(),
                components,
            };
        }

        let score =
            (components.iter().map(|signal| signal.score as u64).sum::<u64>()
                / components.len() as u64) as u8;
        let status = if components
            .iter()
            .any(|signal| signal.status == HealthStatus::Offline)
        {
            HealthStatus::Offline
        } else if components
            .iter()
            .any(|signal| signal.status == HealthStatus::Critical)
        {
            HealthStatus::Critical
        } else if components
            .iter()
            .any(|signal| signal.status == HealthStatus::Warning)
        {
            HealthStatus::Warning
        } else {
            self.status_from_score(score)
        };

        HealthReport {
            status,
            score,
            checked_at_millis: now_millis(),
            components,
        }
    }

    pub fn config(&self) -> &HealthConfig {
        &self.config
    }

    fn status_from_score(&self, score: u8) -> HealthStatus {
        if score == 0 {
            return HealthStatus::Offline;
        }

        if score < self.config.critical_score_threshold {
            HealthStatus::Critical
        } else if score < self.config.warning_score_threshold {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        }
    }
}

impl HealthChecker for HealthManager {
    fn check_all(&self) -> BoxFuture<'_, HealthReport> {
        Box::pin(async move { HealthManager::report(self).await })
    }
}

fn score_from_status(status: HealthStatus) -> u8 {
    match status {
        HealthStatus::Healthy => 100,
        HealthStatus::Warning => 75,
        HealthStatus::Critical => 25,
        HealthStatus::Offline => 0,
    }
}

fn now_millis() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}
