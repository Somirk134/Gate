//! Engine state and lifecycle phase storage.

use crate::error::EngineError;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Unified engine lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EnginePhase {
    Created,
    Initializing,
    Ready,
    Running,
    Paused,
    Stopping,
    Stopped,
    Restarting,
    Error,
}

/// Thread-safe holder for the current engine state.
#[derive(Debug)]
pub struct EngineState {
    phase: RwLock<EnginePhase>,
}

impl Default for EngineState {
    fn default() -> Self {
        Self::new(EnginePhase::Created)
    }
}

impl EngineState {
    pub fn new(phase: EnginePhase) -> Self {
        Self {
            phase: RwLock::new(phase),
        }
    }

    pub fn current(&self) -> EnginePhase {
        *self.phase.read()
    }

    pub fn transition_to(&self, next: EnginePhase) -> Result<EnginePhase, EngineError> {
        let current = self.current();
        if is_valid_transition(current, next) {
            *self.phase.write() = next;
            Ok(next)
        } else {
            Err(EngineError::InvalidStateTransition {
                from: current,
                to: next,
            })
        }
    }
}

fn is_valid_transition(from: EnginePhase, to: EnginePhase) -> bool {
    use EnginePhase::*;

    matches!(
        (from, to),
        (Created, Initializing)
            | (Initializing, Ready)
            | (Initializing, Error)
            | (Ready, Running)
            | (Ready, Stopping)
            | (Running, Paused)
            | (Running, Stopping)
            | (Running, Restarting)
            | (Running, Error)
            | (Paused, Running)
            | (Paused, Stopping)
            | (Stopping, Stopped)
            | (Restarting, Initializing)
            | (Restarting, Error)
            | (Stopped, Initializing)
            | (Error, Stopping)
            | (Error, Restarting)
            | (Error, Stopped)
    )
}
