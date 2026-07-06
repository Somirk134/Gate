//! Runtime state machines.

use crate::runtime::error::{ForwardError, RuntimeError};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};

/// Tunnel runtime lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RuntimeState {
    Created,
    Initializing,
    Starting,
    Running,
    Paused,
    Stopping,
    Stopped,
    Restarting,
    Failed,
    Closed,
}

impl RuntimeState {
    pub fn is_active(self) -> bool {
        matches!(
            self,
            Self::Initializing | Self::Starting | Self::Running | Self::Paused | Self::Restarting
        )
    }

    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Closed)
    }

    pub fn can_restart(self) -> bool {
        matches!(
            self,
            Self::Running | Self::Paused | Self::Stopped | Self::Failed
        )
    }
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
        (Created, Initializing)
            | (Created, Starting)
            | (Created, Stopping)
            | (Created, Failed)
            | (Created, Closed)
            | (Initializing, Starting)
            | (Initializing, Stopping)
            | (Initializing, Failed)
            | (Initializing, Closed)
            | (Starting, Running)
            | (Starting, Stopping)
            | (Starting, Failed)
            | (Starting, Closed)
            | (Running, Paused)
            | (Running, Stopping)
            | (Running, Restarting)
            | (Running, Failed)
            | (Running, Closed)
            | (Paused, Running)
            | (Paused, Stopping)
            | (Paused, Restarting)
            | (Paused, Failed)
            | (Paused, Closed)
            | (Stopping, Stopped)
            | (Stopping, Failed)
            | (Stopping, Closed)
            | (Stopped, Starting)
            | (Stopped, Initializing)
            | (Stopped, Closed)
            | (Restarting, Stopping)
            | (Restarting, Starting)
            | (Restarting, Initializing)
            | (Restarting, Failed)
            | (Restarting, Closed)
            | (Failed, Restarting)
            | (Failed, Stopping)
            | (Failed, Closed)
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
