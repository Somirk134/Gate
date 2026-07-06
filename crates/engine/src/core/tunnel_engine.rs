//! Tunnel Engine facade.

use super::{EngineBuilder, EngineContext, EngineLifecycle, EngineManager, EnginePhase};
use crate::error::EngineError;
use tracing::info;

/// Main Tunnel Engine entry point.
#[derive(Clone)]
pub struct TunnelEngine {
    context: EngineContext,
    manager: EngineManager,
}

impl TunnelEngine {
    pub fn builder() -> EngineBuilder {
        EngineBuilder::new()
    }

    pub fn new(context: EngineContext, manager: EngineManager) -> Self {
        Self { context, manager }
    }

    pub fn context(&self) -> &EngineContext {
        &self.context
    }

    pub fn manager(&self) -> &EngineManager {
        &self.manager
    }
}

impl EngineLifecycle for TunnelEngine {
    fn state(&self) -> EnginePhase {
        self.context.state.current()
    }

    fn initialize(&self) -> Result<(), EngineError> {
        self.context
            .state
            .transition_to(EnginePhase::Initializing)?;
        info!(target: "gate_engine", "tunnel engine initialized");
        self.context.state.transition_to(EnginePhase::Ready)?;
        Ok(())
    }

    fn start(&self) -> Result<(), EngineError> {
        self.context.state.transition_to(EnginePhase::Running)?;
        Ok(())
    }

    fn pause(&self) -> Result<(), EngineError> {
        self.context.state.transition_to(EnginePhase::Paused)?;
        Ok(())
    }

    fn stop(&self) -> Result<(), EngineError> {
        self.context.state.transition_to(EnginePhase::Stopping)?;
        self.context.state.transition_to(EnginePhase::Stopped)?;
        Ok(())
    }

    fn restart(&self) -> Result<(), EngineError> {
        self.context.state.transition_to(EnginePhase::Restarting)?;
        self.context
            .state
            .transition_to(EnginePhase::Initializing)?;
        self.context.state.transition_to(EnginePhase::Ready)?;
        self.context.state.transition_to(EnginePhase::Running)?;
        Ok(())
    }
}
