//! Runtime supervisor and restart policy.

use crate::core::TunnelId;
use crate::runtime::error::RuntimeError;
use crate::runtime::lifecycle::RuntimeLifecycle;
use crate::runtime::reliability::{
    RecoveryDecision, RecoveryTrigger, RuntimeId, RuntimeRecoveryFlow,
};
use crate::runtime::state::RuntimeState;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RestartPolicy {
    Never,
    OnFailure,
    Always,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub open_timeout: Duration,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            open_timeout: Duration::from_secs(60),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorConfig {
    pub restart_policy: RestartPolicy,
    pub restart_delay: Duration,
    pub max_retry: u32,
    pub circuit_breaker: CircuitBreakerConfig,
}

impl Default for SupervisorConfig {
    fn default() -> Self {
        Self {
            restart_policy: RestartPolicy::OnFailure,
            restart_delay: Duration::from_secs(1),
            max_retry: 3,
            circuit_breaker: CircuitBreakerConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeIdentity {
    pub runtime_id: RuntimeId,
    pub tunnel_id: TunnelId,
    pub name: String,
}

impl RuntimeIdentity {
    pub fn new(tunnel_id: TunnelId, name: impl Into<String>) -> Self {
        Self {
            runtime_id: RuntimeId::new(),
            tunnel_id,
            name: name.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupervisorAction {
    None,
    Start,
    Restart,
    Stop,
    Escalate,
    CircuitOpen,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupervisorSnapshot {
    pub identity: RuntimeIdentity,
    pub state: RuntimeState,
    pub restart_count: u32,
    pub failure_count: u32,
    pub circuit_state: CircuitBreakerState,
    pub last_failure_millis: Option<u64>,
    pub last_action: SupervisorAction,
}

struct SupervisedRuntime {
    identity: RuntimeIdentity,
    runtime: Arc<dyn RuntimeLifecycle>,
    snapshot: parking_lot::RwLock<SupervisorSnapshot>,
}

impl std::fmt::Debug for SupervisedRuntime {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("SupervisedRuntime")
            .field("identity", &self.identity)
            .finish_non_exhaustive()
    }
}

#[derive(Debug)]
pub struct RuntimeSupervisor {
    config: SupervisorConfig,
    recovery: Arc<RuntimeRecoveryFlow>,
    runtimes: DashMap<RuntimeId, Arc<SupervisedRuntime>>,
}

impl Default for RuntimeSupervisor {
    fn default() -> Self {
        Self::new(SupervisorConfig::default())
    }
}

impl RuntimeSupervisor {
    pub fn new(config: SupervisorConfig) -> Self {
        Self {
            config,
            recovery: Arc::new(RuntimeRecoveryFlow::new()),
            runtimes: DashMap::new(),
        }
    }

    pub fn with_recovery(config: SupervisorConfig, recovery: Arc<RuntimeRecoveryFlow>) -> Self {
        Self {
            config,
            recovery,
            runtimes: DashMap::new(),
        }
    }

    pub fn register(
        &self,
        identity: RuntimeIdentity,
        runtime: Arc<dyn RuntimeLifecycle>,
    ) -> RuntimeId {
        let snapshot = SupervisorSnapshot {
            identity: identity.clone(),
            state: runtime.state(),
            restart_count: 0,
            failure_count: 0,
            circuit_state: CircuitBreakerState::Closed,
            last_failure_millis: None,
            last_action: SupervisorAction::None,
        };
        let runtime_id = identity.runtime_id;
        self.runtimes.insert(
            runtime_id,
            Arc::new(SupervisedRuntime {
                identity,
                runtime,
                snapshot: parking_lot::RwLock::new(snapshot),
            }),
        );
        runtime_id
    }

    pub fn unregister(&self, runtime_id: RuntimeId) -> Option<SupervisorSnapshot> {
        self.runtimes
            .remove(&runtime_id)
            .map(|(_, runtime)| runtime.snapshot.read().clone())
    }

    pub async fn start(&self, runtime_id: RuntimeId) -> Result<SupervisorAction, RuntimeError> {
        let runtime = self.get(runtime_id)?;
        runtime.runtime.start().await?;
        self.update_snapshot(&runtime, SupervisorAction::Start, |snapshot| {
            snapshot.state = runtime.runtime.state();
        });
        Ok(SupervisorAction::Start)
    }

    pub async fn stop(&self, runtime_id: RuntimeId) -> Result<SupervisorAction, RuntimeError> {
        let runtime = self.get(runtime_id)?;
        runtime.runtime.stop().await?;
        self.update_snapshot(&runtime, SupervisorAction::Stop, |snapshot| {
            snapshot.state = runtime.runtime.state();
        });
        Ok(SupervisorAction::Stop)
    }

    pub async fn recover(
        &self,
        runtime_id: RuntimeId,
        trigger: RecoveryTrigger,
        message: Option<String>,
    ) -> Result<SupervisorAction, RuntimeError> {
        let runtime = self.get(runtime_id)?;
        let snapshot = runtime.snapshot.read().clone();
        let event = self.recovery.recover(
            runtime_id,
            trigger,
            runtime.runtime.state(),
            snapshot.restart_count,
            self.config.max_retry,
            message,
        );

        match event.decision {
            RecoveryDecision::Ignore => Ok(SupervisorAction::None),
            RecoveryDecision::Stop => self.stop(runtime_id).await,
            RecoveryDecision::Escalate => {
                self.open_circuit(&runtime);
                Ok(SupervisorAction::Escalate)
            }
            RecoveryDecision::Restart => self.restart(runtime).await,
        }
    }

    pub async fn supervise_once(
        &self,
        runtime_id: RuntimeId,
    ) -> Result<SupervisorAction, RuntimeError> {
        let runtime = self.get(runtime_id)?;
        let state = runtime.runtime.state();
        self.update_snapshot(&runtime, SupervisorAction::None, |snapshot| {
            snapshot.state = state;
        });

        if state == RuntimeState::Failed {
            return self
                .recover(
                    runtime_id,
                    RecoveryTrigger::RuntimeError,
                    Some("runtime failed".into()),
                )
                .await;
        }

        if self.config.restart_policy == RestartPolicy::Always && state == RuntimeState::Stopped {
            return self.restart(runtime).await;
        }

        Ok(SupervisorAction::None)
    }

    pub fn snapshot(&self, runtime_id: RuntimeId) -> Option<SupervisorSnapshot> {
        self.runtimes
            .get(&runtime_id)
            .map(|entry| entry.snapshot.read().clone())
    }

    pub fn snapshots(&self) -> Vec<SupervisorSnapshot> {
        self.runtimes
            .iter()
            .map(|entry| entry.snapshot.read().clone())
            .collect()
    }

    pub fn recovery(&self) -> Arc<RuntimeRecoveryFlow> {
        Arc::clone(&self.recovery)
    }

    fn get(&self, runtime_id: RuntimeId) -> Result<Arc<SupervisedRuntime>, RuntimeError> {
        self.runtimes
            .get(&runtime_id)
            .map(|entry| Arc::clone(entry.value()))
            .ok_or_else(|| RuntimeError::InvalidConfig {
                reason: format!("supervised runtime not found: {runtime_id}"),
            })
    }

    async fn restart(
        &self,
        runtime: Arc<SupervisedRuntime>,
    ) -> Result<SupervisorAction, RuntimeError> {
        if self.config.restart_policy == RestartPolicy::Never {
            return Ok(SupervisorAction::None);
        }

        if self.circuit_open(&runtime) {
            self.update_snapshot(&runtime, SupervisorAction::CircuitOpen, |_| {});
            return Ok(SupervisorAction::CircuitOpen);
        }

        sleep(self.config.restart_delay).await;
        runtime.runtime.restart().await?;
        self.update_snapshot(&runtime, SupervisorAction::Restart, |snapshot| {
            snapshot.state = runtime.runtime.state();
            snapshot.restart_count = snapshot.restart_count.saturating_add(1);
        });
        Ok(SupervisorAction::Restart)
    }

    fn circuit_open(&self, runtime: &SupervisedRuntime) -> bool {
        let snapshot = runtime.snapshot.read();
        if snapshot.circuit_state == CircuitBreakerState::Open {
            let Some(last_failure_millis) = snapshot.last_failure_millis else {
                return true;
            };
            let elapsed = now_millis().saturating_sub(last_failure_millis);
            let open_timeout_millis = self
                .config
                .circuit_breaker
                .open_timeout
                .as_millis()
                .min(u128::from(u64::MAX)) as u64;
            return elapsed < open_timeout_millis;
        }
        false
    }

    fn open_circuit(&self, runtime: &SupervisedRuntime) {
        self.update_snapshot(runtime, SupervisorAction::CircuitOpen, |snapshot| {
            snapshot.circuit_state = CircuitBreakerState::Open;
            snapshot.failure_count = snapshot.failure_count.saturating_add(1);
            snapshot.last_failure_millis = Some(now_millis());
        });
    }

    fn update_snapshot(
        &self,
        runtime: &SupervisedRuntime,
        action: SupervisorAction,
        update: impl FnOnce(&mut SupervisorSnapshot),
    ) {
        let mut snapshot = runtime.snapshot.write();
        if runtime.runtime.state() == RuntimeState::Failed {
            snapshot.failure_count = snapshot.failure_count.saturating_add(1);
            snapshot.last_failure_millis = Some(now_millis());
            if snapshot.failure_count >= self.config.circuit_breaker.failure_threshold {
                snapshot.circuit_state = CircuitBreakerState::Open;
            }
        } else if snapshot.circuit_state == CircuitBreakerState::Open {
            snapshot.circuit_state = CircuitBreakerState::HalfOpen;
        }
        snapshot.last_action = action;
        update(&mut snapshot);
    }
}

fn now_millis() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}
