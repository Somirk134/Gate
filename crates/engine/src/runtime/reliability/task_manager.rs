//! Runtime task manager.

use crate::runtime::error::SchedulerError;
use crate::runtime::scheduler::RuntimeScheduler;
use crate::runtime::worker::{Task, TaskId, TaskKind, TaskStatistics, TaskStatus};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

/// Thin reliability facade over the runtime scheduler.
#[derive(Debug, Clone)]
pub struct RuntimeTaskManager {
    scheduler: Arc<RuntimeScheduler>,
}

impl RuntimeTaskManager {
    pub fn new(scheduler: Arc<RuntimeScheduler>) -> Self {
        Self { scheduler }
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
        self.scheduler.spawn(kind, name, future)
    }

    pub fn cancel(&self, id: TaskId) -> Result<(), SchedulerError> {
        self.scheduler.cancel(id)
    }

    pub async fn wait(&self, id: TaskId, timeout: Duration) -> Result<TaskStatus, SchedulerError> {
        self.scheduler.wait(id, timeout).await
    }

    pub async fn wait_all(&self, timeout: Duration) -> Result<(), SchedulerError> {
        self.scheduler.graceful_shutdown(timeout).await
    }

    pub fn tasks(&self) -> Vec<Task> {
        self.scheduler.workers().tasks()
    }

    pub fn failed_tasks(&self) -> Vec<Task> {
        self.tasks()
            .into_iter()
            .filter(|task| task.status == TaskStatus::Failed)
            .collect()
    }

    pub fn statistics(&self) -> TaskStatistics {
        self.scheduler.statistics()
    }

    pub fn scheduler(&self) -> Arc<RuntimeScheduler> {
        Arc::clone(&self.scheduler)
    }
}
