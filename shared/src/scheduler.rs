use crate::error::AppError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScheduledTaskKind {
    Heartbeat,
    Cleanup,
    Reconnect,
    Statistics,
    HealthCheck,
}

pub trait ScheduledTask: Send + Sync {
    fn kind(&self) -> ScheduledTaskKind;
}

pub trait TaskScheduler: Send + Sync {
    fn schedule(&self, task: Box<dyn ScheduledTask>) -> Result<(), AppError>;

    fn cancel(&self, kind: ScheduledTaskKind) -> Result<(), AppError>;
}
