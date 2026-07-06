//! Unified runtime health checks.

use crate::health::HealthStatus;
use crate::runtime::state::RuntimeState;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};

/// Health dimensions required by runtime reliability.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuntimeHealthTarget {
    RuntimeAlive,
    ListenerAlive,
    TunnelAlive,
    TlsAlive,
    ConnectionAlive,
    Heartbeat,
}

/// One health signal with a normalized score.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeHealthSignal {
    pub target: RuntimeHealthTarget,
    pub status: HealthStatus,
    pub score: u8,
    pub message: Option<String>,
    pub checked_at_millis: u64,
}

impl RuntimeHealthSignal {
    pub fn new(
        target: RuntimeHealthTarget,
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

    pub fn healthy(target: RuntimeHealthTarget) -> Self {
        Self::new(target, HealthStatus::Healthy, 100, None)
    }

    pub fn offline(target: RuntimeHealthTarget, message: impl Into<String>) -> Self {
        Self::new(target, HealthStatus::Offline, 0, Some(message.into()))
    }
}

/// Aggregated health report for one runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeHealthReport {
    pub status: HealthStatus,
    pub score: u8,
    pub checked_at_millis: u64,
    pub signals: Vec<RuntimeHealthSignal>,
}

/// In-memory health check aggregator.
#[derive(Debug, Default)]
pub struct RuntimeHealthCheck {
    signals: DashMap<RuntimeHealthTarget, RuntimeHealthSignal>,
}

impl RuntimeHealthCheck {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn record(&self, signal: RuntimeHealthSignal) -> RuntimeHealthReport {
        self.signals.insert(signal.target, signal);
        self.report()
    }

    pub fn check_runtime_alive(&self, state: RuntimeState) -> RuntimeHealthSignal {
        let signal = match state {
            RuntimeState::Running | RuntimeState::Paused => {
                RuntimeHealthSignal::healthy(RuntimeHealthTarget::RuntimeAlive)
            }
            RuntimeState::Initializing | RuntimeState::Starting | RuntimeState::Restarting => {
                RuntimeHealthSignal::new(
                    RuntimeHealthTarget::RuntimeAlive,
                    HealthStatus::Warning,
                    80,
                    Some(format!("runtime is {state:?}")),
                )
            }
            RuntimeState::Failed => RuntimeHealthSignal::new(
                RuntimeHealthTarget::RuntimeAlive,
                HealthStatus::Critical,
                10,
                Some("runtime failed".to_string()),
            ),
            RuntimeState::Created | RuntimeState::Stopping | RuntimeState::Stopped => {
                RuntimeHealthSignal::new(
                    RuntimeHealthTarget::RuntimeAlive,
                    HealthStatus::Warning,
                    60,
                    Some(format!("runtime is {state:?}")),
                )
            }
            RuntimeState::Closed => {
                RuntimeHealthSignal::offline(RuntimeHealthTarget::RuntimeAlive, "runtime closed")
            }
        };
        self.record(signal.clone());
        signal
    }

    pub fn check_listener_alive(&self, alive: bool) -> RuntimeHealthSignal {
        self.check_alive(
            RuntimeHealthTarget::ListenerAlive,
            alive,
            "listener is not alive",
        )
    }

    pub fn check_tunnel_alive(&self, alive: bool) -> RuntimeHealthSignal {
        self.check_alive(
            RuntimeHealthTarget::TunnelAlive,
            alive,
            "tunnel is not alive",
        )
    }

    pub fn check_tls_alive(&self, alive: bool) -> RuntimeHealthSignal {
        self.check_alive(RuntimeHealthTarget::TlsAlive, alive, "TLS is not alive")
    }

    pub fn check_connection_alive(&self, alive: bool) -> RuntimeHealthSignal {
        self.check_alive(
            RuntimeHealthTarget::ConnectionAlive,
            alive,
            "connection is not alive",
        )
    }

    pub fn check_heartbeat(&self, alive: bool, missed: u32) -> RuntimeHealthSignal {
        let signal = if alive && missed == 0 {
            RuntimeHealthSignal::healthy(RuntimeHealthTarget::Heartbeat)
        } else if alive {
            RuntimeHealthSignal::new(
                RuntimeHealthTarget::Heartbeat,
                HealthStatus::Warning,
                75,
                Some(format!("heartbeat missed {missed} tick(s)")),
            )
        } else {
            RuntimeHealthSignal::offline(RuntimeHealthTarget::Heartbeat, "heartbeat lost")
        };
        self.record(signal.clone());
        signal
    }

    pub fn report(&self) -> RuntimeHealthReport {
        let signals = self
            .signals
            .iter()
            .map(|entry| entry.value().clone())
            .collect::<Vec<_>>();

        if signals.is_empty() {
            return RuntimeHealthReport {
                status: HealthStatus::Warning,
                score: 80,
                checked_at_millis: now_millis(),
                signals,
            };
        }

        let score = (signals
            .iter()
            .map(|signal| u64::from(signal.score))
            .sum::<u64>()
            / signals.len() as u64) as u8;
        let status = if signals
            .iter()
            .any(|signal| signal.status == HealthStatus::Offline)
        {
            HealthStatus::Offline
        } else if signals
            .iter()
            .any(|signal| signal.status == HealthStatus::Critical)
        {
            HealthStatus::Critical
        } else if signals
            .iter()
            .any(|signal| signal.status == HealthStatus::Warning)
        {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        };

        RuntimeHealthReport {
            status,
            score,
            checked_at_millis: now_millis(),
            signals,
        }
    }

    fn check_alive(
        &self,
        target: RuntimeHealthTarget,
        alive: bool,
        offline_message: &'static str,
    ) -> RuntimeHealthSignal {
        let signal = if alive {
            RuntimeHealthSignal::healthy(target)
        } else {
            RuntimeHealthSignal::offline(target, offline_message)
        };
        self.record(signal.clone());
        signal
    }
}

fn now_millis() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}
