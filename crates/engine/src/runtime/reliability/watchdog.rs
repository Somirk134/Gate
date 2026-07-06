//! Runtime watchdog heuristics.

use crate::runtime::reliability::RuntimeReliabilityMetrics;
use crate::runtime::worker::TaskStatistics;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchdogConfig {
    pub heartbeat_timeout: Duration,
    pub max_blocking_duration: Duration,
    pub max_task_growth: u64,
    pub max_memory_growth_bytes: u64,
    pub max_channel_depth: u64,
}

impl Default for WatchdogConfig {
    fn default() -> Self {
        Self {
            heartbeat_timeout: Duration::from_secs(30),
            max_blocking_duration: Duration::from_secs(10),
            max_task_growth: 1024,
            max_memory_growth_bytes: 256 * 1024 * 1024,
            max_channel_depth: 10_000,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WatchdogFindingKind {
    DeadLoop,
    Deadlock,
    LongBlocking,
    TaskLeak,
    MemoryGrowth,
    ChannelBlocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WatchdogSeverity {
    Info,
    Warning,
    Critical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchdogFinding {
    pub kind: WatchdogFindingKind,
    pub severity: WatchdogSeverity,
    pub message: String,
    pub observed_at_millis: u64,
}

#[derive(Debug)]
pub struct RuntimeWatchdog {
    config: WatchdogConfig,
    previous_metrics: parking_lot::RwLock<Option<RuntimeReliabilityMetrics>>,
}

impl Default for RuntimeWatchdog {
    fn default() -> Self {
        Self::new(WatchdogConfig::default())
    }
}

impl RuntimeWatchdog {
    pub fn new(config: WatchdogConfig) -> Self {
        Self {
            config,
            previous_metrics: parking_lot::RwLock::new(None),
        }
    }

    pub fn inspect(
        &self,
        metrics: RuntimeReliabilityMetrics,
        task_statistics: Option<TaskStatistics>,
        last_heartbeat_age: Option<Duration>,
        last_progress_age: Option<Duration>,
    ) -> Vec<WatchdogFinding> {
        let mut findings = Vec::new();

        if last_heartbeat_age
            .map(|age| age >= self.config.heartbeat_timeout)
            .unwrap_or(false)
        {
            findings.push(finding(
                WatchdogFindingKind::Deadlock,
                WatchdogSeverity::Critical,
                "heartbeat timeout exceeded watchdog threshold",
            ));
        }

        if last_progress_age
            .map(|age| age >= self.config.max_blocking_duration)
            .unwrap_or(false)
        {
            findings.push(finding(
                WatchdogFindingKind::LongBlocking,
                WatchdogSeverity::Warning,
                "runtime made no observable progress within blocking threshold",
            ));
        }

        if metrics.channel_depth >= self.config.max_channel_depth {
            findings.push(finding(
                WatchdogFindingKind::ChannelBlocked,
                WatchdogSeverity::Critical,
                "channel depth exceeded watchdog threshold",
            ));
        }

        if let Some(task_statistics) = task_statistics {
            if task_statistics.failed > 0 {
                findings.push(finding(
                    WatchdogFindingKind::DeadLoop,
                    WatchdogSeverity::Warning,
                    "failed task count is non-zero",
                ));
            }
            if task_statistics.running as u64 >= self.config.max_task_growth {
                findings.push(finding(
                    WatchdogFindingKind::TaskLeak,
                    WatchdogSeverity::Critical,
                    "running task count exceeded watchdog threshold",
                ));
            }
        }

        if let Some(previous) = self.previous_metrics.read().clone() {
            let task_growth = metrics.task_count.saturating_sub(previous.task_count);
            if task_growth >= self.config.max_task_growth {
                findings.push(finding(
                    WatchdogFindingKind::TaskLeak,
                    WatchdogSeverity::Critical,
                    "task count growth exceeded watchdog threshold",
                ));
            }

            let memory_growth = metrics.memory_bytes.saturating_sub(previous.memory_bytes);
            if memory_growth >= self.config.max_memory_growth_bytes {
                findings.push(finding(
                    WatchdogFindingKind::MemoryGrowth,
                    WatchdogSeverity::Critical,
                    "memory growth exceeded watchdog threshold",
                ));
            }
        }

        *self.previous_metrics.write() = Some(metrics);
        findings
    }

    pub fn config(&self) -> &WatchdogConfig {
        &self.config
    }
}

fn finding(
    kind: WatchdogFindingKind,
    severity: WatchdogSeverity,
    message: &'static str,
) -> WatchdogFinding {
    WatchdogFinding {
        kind,
        severity,
        message: message.to_string(),
        observed_at_millis: now_millis(),
    }
}

fn now_millis() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}
