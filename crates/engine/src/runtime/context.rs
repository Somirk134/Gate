//! Shared runtime context.

use crate::runtime::buffer::BufferPool;
use crate::runtime::config::RuntimeConfig;
use crate::runtime::monitor::TrafficStatistics;
use crate::runtime::session::SessionManager;
use crate::runtime::state::RuntimeStateMachine;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::watch;

/// Shared dependencies and state passed to runtime components.
#[derive(Debug)]
pub struct RuntimeContext {
    pub config: RuntimeConfig,
    pub state: Arc<RuntimeStateMachine>,
    pub sessions: Arc<SessionManager>,
    pub traffic: Arc<TrafficStatistics>,
    pub buffers: Arc<BufferPool>,
    shutdown: watch::Sender<bool>,
    started_at: Instant,
}

impl RuntimeContext {
    pub fn new(config: RuntimeConfig) -> Self {
        let traffic = Arc::new(TrafficStatistics::new());
        let sessions = Arc::new(SessionManager::new(Arc::clone(&traffic)));
        let buffers = Arc::new(BufferPool::new(config.buffer.clone()));
        let (shutdown, _) = watch::channel(false);

        Self {
            config,
            state: Arc::new(RuntimeStateMachine::default()),
            sessions,
            traffic,
            buffers,
            shutdown,
            started_at: Instant::now(),
        }
    }

    pub fn subscribe_shutdown(&self) -> watch::Receiver<bool> {
        self.shutdown.subscribe()
    }

    pub fn request_shutdown(&self) {
        let _ = self.shutdown.send(true);
    }

    pub fn reset_shutdown(&self) {
        let _ = self.shutdown.send(false);
    }

    pub fn shutdown_requested(&self) -> bool {
        *self.shutdown.borrow()
    }

    pub fn uptime(&self) -> Duration {
        Instant::now().saturating_duration_since(self.started_at)
    }
}
