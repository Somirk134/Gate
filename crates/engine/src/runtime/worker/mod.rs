//! Worker pool for runtime Tokio tasks.

use crate::runtime::error::SchedulerError;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tokio::task::JoinHandle;
use tokio::time::timeout;
use uuid::Uuid;

/// Stable task identifier.
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

/// Runtime task kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskKind {
    Runtime,
    Listener,
    Forward,
    Statistics,
    Cleanup,
    Retry,
    Monitor,
    Heartbeat,
}

/// Runtime task status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TaskStatus {
    Created,
    Running,
    Cancelled,
    Finished,
    Failed,
}

/// Runtime task metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: TaskId,
    pub name: String,
    pub kind: TaskKind,
    pub status: TaskStatus,
    pub created_at_millis: u64,
    pub finished_at_millis: Option<u64>,
}

#[derive(Debug)]
struct WorkerPoolInner {
    max_tasks: usize,
    tasks: Arc<DashMap<TaskId, Task>>,
    handles: Arc<DashMap<TaskId, JoinHandle<()>>>,
}

/// Tokio worker task registry.
#[derive(Debug, Clone)]
pub struct WorkerPool {
    inner: Arc<WorkerPoolInner>,
}

impl WorkerPool {
    pub fn new(max_tasks: usize) -> Self {
        Self {
            inner: Arc::new(WorkerPoolInner {
                max_tasks,
                tasks: Arc::new(DashMap::new()),
                handles: Arc::new(DashMap::new()),
            }),
        }
    }

    pub fn spawn<F>(
        &self,
        kind: TaskKind,
        name: impl Into<String>,
        future: F,
    ) -> Result<TaskId, SchedulerError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.reap_completed_handles();

        if self.running_count() >= self.inner.max_tasks {
            return Err(SchedulerError::TaskLimitExceeded {
                max_tasks: self.inner.max_tasks,
            });
        }

        let id = TaskId::new();
        let name = name.into();
        let tasks = Arc::clone(&self.inner.tasks);
        let task = Task {
            id,
            name,
            kind,
            status: TaskStatus::Running,
            created_at_millis: now_millis(),
            finished_at_millis: None,
        };
        tasks.insert(id, task);

        let handles = Arc::clone(&self.inner.handles);
        let handle = tokio::spawn(async move {
            future.await;
            if let Some(mut task) = tasks.get_mut(&id) {
                if task.status == TaskStatus::Running {
                    task.status = TaskStatus::Finished;
                }
                task.finished_at_millis = Some(now_millis());
            }
            handles.remove(&id);
        });

        self.inner.handles.insert(id, handle);
        Ok(id)
    }

    pub fn cancel(&self, id: TaskId) -> Result<(), SchedulerError> {
        if let Some((_, handle)) = self.inner.handles.remove(&id) {
            handle.abort();
        }

        if let Some(mut task) = self.inner.tasks.get_mut(&id) {
            task.status = TaskStatus::Cancelled;
            task.finished_at_millis = Some(now_millis());
            return Ok(());
        }

        Err(SchedulerError::TaskNotFound { id })
    }

    pub fn abort_all(&self) {
        let ids: Vec<TaskId> = self.inner.handles.iter().map(|entry| *entry.key()).collect();
        for id in ids {
            let _ = self.cancel(id);
        }
    }

    pub async fn graceful_shutdown(&self, shutdown_timeout: Duration) -> Result<(), SchedulerError> {
        let ids: Vec<TaskId> = self.inner.handles.iter().map(|entry| *entry.key()).collect();

        for id in ids {
            let Some((_, mut handle)) = self.inner.handles.remove(&id) else {
                continue;
            };
            let name = self
                .inner
                .tasks
                .get(&id)
                .map(|task| task.name.clone())
                .unwrap_or_else(|| id.to_string());

            match timeout(shutdown_timeout, &mut handle).await {
                Ok(join_result) => match join_result {
                    Ok(()) => self.mark_status(id, TaskStatus::Finished),
                    Err(source) if source.is_cancelled() => self.mark_status(id, TaskStatus::Cancelled),
                    Err(source) => {
                        self.mark_status(id, TaskStatus::Failed);
                        return Err(SchedulerError::Join { name, source });
                    }
                },
                Err(_) => {
                    handle.abort();
                    let _ = handle.await;
                    self.mark_status(id, TaskStatus::Cancelled);
                    return Err(SchedulerError::ShutdownTimeout {
                        name,
                        timeout: shutdown_timeout,
                    });
                }
            }
        }

        Ok(())
    }

    pub fn get(&self, id: &TaskId) -> Option<Task> {
        self.inner.tasks.get(id).map(|entry| entry.value().clone())
    }

    pub fn tasks(&self) -> Vec<Task> {
        self.inner
            .tasks
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn running_count(&self) -> usize {
        self.inner
            .tasks
            .iter()
            .filter(|entry| entry.status == TaskStatus::Running)
            .count()
    }

    fn mark_status(&self, id: TaskId, status: TaskStatus) {
        if let Some(mut task) = self.inner.tasks.get_mut(&id) {
            task.status = status;
            task.finished_at_millis = Some(now_millis());
        }
    }

    fn reap_completed_handles(&self) {
        let ids: Vec<TaskId> = self
            .inner
            .handles
            .iter()
            .filter(|entry| entry.value().is_finished())
            .map(|entry| *entry.key())
            .collect();

        for id in ids {
            let status = self.inner.tasks.get(&id).map(|task| task.status);
            if matches!(
                status,
                Some(TaskStatus::Finished | TaskStatus::Cancelled | TaskStatus::Failed)
            ) {
                self.inner.handles.remove(&id);
            }
        }
    }
}

impl Default for WorkerPool {
    fn default() -> Self {
        Self::new(4096)
    }
}

fn now_millis() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}
