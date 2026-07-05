//! Tunnel Engine builder.

use super::{EngineConfig, EngineContext, EngineManager, EngineState, TunnelEngine};
use crate::event::event_channel;
use crate::runtime::RuntimeManager;
use std::sync::Arc;

/// Builder for [`TunnelEngine`].
#[derive(Default)]
pub struct EngineBuilder {
    config: EngineConfig,
    manager: EngineManager,
}

impl EngineBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn config(mut self, config: EngineConfig) -> Self {
        self.config = config;
        self
    }

    pub fn manager(mut self, manager: EngineManager) -> Self {
        self.manager = manager;
        self
    }

    pub fn build(self) -> TunnelEngine {
        let (events, _subscriber) = event_channel(1024);
        let state = Arc::new(EngineState::default());
        let runtime = Arc::new(RuntimeManager::new(self.config.runtime.clone()));
        let context = EngineContext::new(self.config, state, runtime, events);

        TunnelEngine::new(context, self.manager)
    }
}
