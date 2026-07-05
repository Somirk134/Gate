//! Shared engine context.

use super::{EngineConfig, EngineState};
use crate::event::EventPublisher;
use crate::runtime::RuntimeManager;
use std::sync::Arc;

/// Shared dependencies and state passed to engine modules.
#[derive(Clone)]
pub struct EngineContext {
    pub config: EngineConfig,
    pub state: Arc<EngineState>,
    pub runtime: Arc<RuntimeManager>,
    pub events: EventPublisher,
}

impl EngineContext {
    pub fn new(
        config: EngineConfig,
        state: Arc<EngineState>,
        runtime: Arc<RuntimeManager>,
        events: EventPublisher,
    ) -> Self {
        Self {
            config,
            state,
            runtime,
            events,
        }
    }
}
