//! Unified recovery flow for runtime failures.

use crate::runtime::reliability::{RuntimeId, RuntimeTraceContext};
use crate::runtime::state::RuntimeState;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecoveryTrigger {
    ListenerPanic,
    TaskPanic,
    TlsError,
    RuntimeError,
    HealthCheckFailed,
    WatchdogFinding,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RecoveryDecision {
    Ignore,
    Restart,
    Stop,
    Escalate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryEvent {
    pub runtime_id: RuntimeId,
    pub trigger: RecoveryTrigger,
    pub state: RuntimeState,
    pub retry_count: u32,
    pub decision: RecoveryDecision,
    pub message: Option<String>,
    pub trace: Option<RuntimeTraceContext>,
    pub created_at_millis: u64,
}

#[derive(Debug, Default)]
pub struct RuntimeRecoveryFlow {
    events: Mutex<Vec<RecoveryEvent>>,
}

impl RuntimeRecoveryFlow {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn recover(
        &self,
        runtime_id: RuntimeId,
        trigger: RecoveryTrigger,
        state: RuntimeState,
        retry_count: u32,
        max_retry: u32,
        message: Option<String>,
    ) -> RecoveryEvent {
        let decision = Self::decide(trigger, state, retry_count, max_retry);
        let event = RecoveryEvent {
            runtime_id,
            trigger,
            state,
            retry_count,
            decision,
            message,
            trace: None,
            created_at_millis: now_millis(),
        };
        self.events.lock().push(event.clone());
        event
    }

    pub fn record(&self, event: RecoveryEvent) {
        self.events.lock().push(event);
    }

    pub fn events(&self) -> Vec<RecoveryEvent> {
        self.events.lock().clone()
    }

    pub fn decide(
        trigger: RecoveryTrigger,
        state: RuntimeState,
        retry_count: u32,
        max_retry: u32,
    ) -> RecoveryDecision {
        if state == RuntimeState::Closed {
            return RecoveryDecision::Ignore;
        }

        if retry_count >= max_retry {
            return RecoveryDecision::Escalate;
        }

        match trigger {
            RecoveryTrigger::ListenerPanic
            | RecoveryTrigger::TaskPanic
            | RecoveryTrigger::TlsError
            | RecoveryTrigger::RuntimeError
            | RecoveryTrigger::HealthCheckFailed
            | RecoveryTrigger::WatchdogFinding => RecoveryDecision::Restart,
        }
    }
}

fn now_millis() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}
