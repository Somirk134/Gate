//! Engine lifecycle contract.

use super::EnginePhase;
use crate::error::EngineError;

/// Lifecycle operations supported by the Tunnel Engine.
pub trait EngineLifecycle: Send + Sync {
    fn state(&self) -> EnginePhase;

    fn initialize(&self) -> Result<(), EngineError>;

    fn start(&self) -> Result<(), EngineError>;

    fn pause(&self) -> Result<(), EngineError>;

    fn stop(&self) -> Result<(), EngineError>;

    fn restart(&self) -> Result<(), EngineError>;
}
