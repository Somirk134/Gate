//! Runtime state machines.

use crate::runtime::error::{ForwardError, RuntimeError};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Tunnel runtime lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuntimeState {
    Created,
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
    Restarting,
    ShuttingDown,
    Shutdown,
    Error,
}

/// Thread-safe runtime state holder.
#[derive(Debug)]
pub struct RuntimeStateMachine {
    state: RwLock<RuntimeState>,
}

impl Default for RuntimeStateMachine {
    fn default() -> Self {
        Self::new(RuntimeState::Created)
    }
}

impl RuntimeStateMachine {
    pub fn new(state: RuntimeState) -> Self {
        Self {
            state: RwLock::new(state),
        }
    }

    pub fn current(&self) -> RuntimeState {
        *self.state.read()
    }

    pub fn transition_to(&self, next: RuntimeState) -> Result<RuntimeState, RuntimeError> {
        let current = self.current();
        if current == next || is_valid_runtime_transition(current, next) {
            *self.state.write() = next;
            Ok(next)
        } else {
            Err(RuntimeError::InvalidStateTransition {
                from: current,
                to: next,
            })
        }
    }
}

fn is_valid_runtime_transition(from: RuntimeState, to: RuntimeState) -> bool {
    use RuntimeState::*;

    matches!(
        (from, to),
        (Created, Starting)
            | (Created, Stopping)
            | (Created, ShuttingDown)
            | (Starting, Running)
            | (Starting, Stopping)
            | (Starting, ShuttingDown)
            | (Starting, Error)
            | (Running, Paused)
            | (Running, Stopping)
            | (Running, Restarting)
            | (Running, ShuttingDown)
            | (Running, Error)
            | (Paused, Running)
            | (Paused, Stopping)
            | (Paused, Restarting)
            | (Paused, ShuttingDown)
            | (Paused, Error)
            | (Stopping, Stopped)
            | (Stopping, Error)
            | (Stopped, Starting)
            | (Stopped, ShuttingDown)
            | (Restarting, Stopping)
            | (Restarting, Starting)
            | (Restarting, ShuttingDown)
            | (Restarting, Error)
            | (ShuttingDown, Shutdown)
            | (ShuttingDown, Error)
            | (Error, Stopping)
            | (Error, ShuttingDown)
            | (Error, Shutdown)
    )
}

/// Per-session lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SessionState {
    Created,
    Connecting,
    Forwarding,
    Paused,
    Closing,
    Closed,
    Failed,
}

/// Per-connection lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConnectionState {
    Created,
    Connecting,
    Connected,
    Retrying,
    Reconnecting,
    Closing,
    Closed,
    Failed,
}

/// Forward pipeline lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ForwardState {
    Created,
    Starting,
    Running,
    Paused,
    Finishing,
    Finished,
    Shutdown,
    Failed,
}

/// Thread-safe forward state holder.
#[derive(Debug)]
pub struct ForwardStateMachine {
    state: RwLock<ForwardState>,
}

impl Default for ForwardStateMachine {
    fn default() -> Self {
        Self::new(ForwardState::Created)
    }
}

impl ForwardStateMachine {
    pub fn new(state: ForwardState) -> Self {
        Self {
            state: RwLock::new(state),
        }
    }

    pub fn current(&self) -> ForwardState {
        *self.state.read()
    }

    pub fn transition_to(&self, next: ForwardState) -> Result<ForwardState, ForwardError> {
        let current = self.current();
        if current == next || is_valid_forward_transition(current, next) {
            *self.state.write() = next;
            Ok(next)
        } else {
            Err(ForwardError::InvalidStateTransition {
                from: current,
                to: next,
            })
        }
    }
}

fn is_valid_forward_transition(from: ForwardState, to: ForwardState) -> bool {
    use ForwardState::*;

    matches!(
        (from, to),
        (Created, Starting)
            | (Finished, Starting)
            | (Failed, Starting)
            | (Shutdown, Starting)
            | (Starting, Running)
            | (Starting, Failed)
            | (Running, Paused)
            | (Running, Finishing)
            | (Running, Shutdown)
            | (Running, Failed)
            | (Paused, Running)
            | (Paused, Shutdown)
            | (Paused, Failed)
            | (Finishing, Finished)
            | (Shutdown, Finished)
            | (Failed, Finished)
    )
}
