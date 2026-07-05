use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerState {
    Starting,
    Initializing,
    Ready,
    Running,
    Stopping,
    Stopped,
    Restarting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimePhase {
    Boot,
    Initialize,
    Ready,
    Running,
    Shutdown,
    Restart,
    GracefulShutdown,
}

pub trait ApplicationRuntime: Send + Sync {
    fn state(&self) -> ServerState;

    fn transition(&self, phase: RuntimePhase) -> Result<ServerState, AppError>;
}

pub trait GracefulShutdown: Send + Sync {
    fn shutdown(&self) -> Result<(), AppError>;
}
