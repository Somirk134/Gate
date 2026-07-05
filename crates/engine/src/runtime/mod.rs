//! Runtime task management boundary.

use crate::config::RuntimeConfig;
use crate::error::EngineError;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::future::Future;
use tokio::task::JoinHandle;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(Uuid);

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for TaskId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TaskStatus {
    Created,
    Running,
    Cancelled,
    Finished,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub name: String,
    pub status: TaskStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RuntimeStatus {
    Created,
    Running,
    ShuttingDown,
    Stopped,
}

/// Central Tokio task manager.
pub struct RuntimeManager {
    config: RuntimeConfig,
    tasks: DashMap<TaskId, Task>,
    handles: DashMap<TaskId, JoinHandle<()>>,
}

impl RuntimeManager {
    pub fn new(config: RuntimeConfig) -> Self {
        Self {
            config,
            tasks: DashMap::new(),
            handles: DashMap::new(),
        }
    }

    pub fn spawn<F>(&self, name: impl Into<String>, future: F) -> Result<TaskId, EngineError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let id = TaskId::new();
        let task = Task {
            id,
            name: name.into(),
            status: TaskStatus::Running,
        };
        let handle = tokio::spawn(future);

        self.tasks.insert(id, task);
        self.handles.insert(id, handle);
        Ok(id)
    }

    pub fn cancel(&self, id: TaskId) -> Result<(), EngineError> {
        if let Some((_, handle)) = self.handles.remove(&id) {
            handle.abort();
        }

        if let Some(mut task) = self.tasks.get_mut(&id) {
            task.status = TaskStatus::Cancelled;
        }

        Ok(())
    }

    pub fn shutdown(&self) -> Result<(), EngineError> {
        let ids: Vec<TaskId> = self.handles.iter().map(|entry| *entry.key()).collect();
        for id in ids {
            self.cancel(id)?;
        }
        Ok(())
    }

    pub async fn graceful_shutdown(&self) -> Result<(), EngineError> {
        self.shutdown()
    }

    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }
}

impl Default for RuntimeManager {
    fn default() -> Self {
        Self::new(RuntimeConfig::default())
    }
}
