use crate::error::AppError;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthTarget {
    App,
    Cpu,
    Memory,
    Disk,
    Network,
    Database,
    Redis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Unknown,
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthConfig {
    pub enabled_targets: Vec<HealthTarget>,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            enabled_targets: vec![HealthTarget::App],
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthReport {
    pub target: HealthTarget,
    pub status: HealthStatus,
    pub checked_at: DateTime<Utc>,
    pub message: Option<String>,
}

pub trait HealthCheck: Send + Sync {
    fn target(&self) -> HealthTarget;

    fn check(&self) -> Result<HealthReport, AppError>;
}

pub trait HealthChecker: Send + Sync {
    fn register(&self, check: Box<dyn HealthCheck>) -> Result<(), AppError>;

    fn check_all(&self) -> Result<Vec<HealthReport>, AppError>;
}
