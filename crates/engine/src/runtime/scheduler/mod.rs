//! Runtime scheduler.

use crate::runtime::error::SchedulerError;
use crate::runtime::worker::{TaskId, TaskKind, TaskStatistics, TaskStatus, WorkerPool};
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

/// Unified task scheduler for runtime, forward, cleanup, retry, and monitor work.
#[derive(Debug, Clone)]
pub struct RuntimeScheduler {
    workers: Arc<WorkerPool>,
}

impl RuntimeScheduler {
    pub fn new(max_tasks: usize) -> Self {
        Self {
            workers: Arc::new(WorkerPool::new(max_tasks)),
        }
    }

    pub fn with_workers(workers: Arc<WorkerPool>) -> Self {
        Self { workers }
    }

    pub fn spawn_runtime<F>(
        &self,
        name: impl Into<String>,
        future: F,
    ) -> Result<TaskId, SchedulerError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.spawn(TaskKind::Runtime, name, future)
    }

    pub fn spawn_listener<F>(
        &self,
        name: impl Into<String>,
        future: F,
    ) -> Result<TaskId, SchedulerError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.spawn(TaskKind::Listener, name, future)
    }

    pub fn spawn_forward<F>(
        &self,
        name: impl Into<String>,
        future: F,
    ) -> Result<TaskId, SchedulerError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.spawn(TaskKind::Forward, name, future)
    }

    pub fn spawn_statistics<F>(
        &self,
        name: impl Into<String>,
        future: F,
    ) -> Result<TaskId, SchedulerError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.spawn(TaskKind::Statistics, name, future)
    }

    pub fn spawn_cleanup<F>(
        &self,
        name: impl Into<String>,
        future: F,
    ) -> Result<TaskId, SchedulerError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.spawn(TaskKind::Cleanup, name, future)
    }

    pub fn spawn_retry<F>(
        &self,
        name: impl Into<String>,
        future: F,
    ) -> Result<TaskId, SchedulerError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.spawn(TaskKind::Retry, name, future)
    }

    pub fn spawn_monitor<F>(
        &self,
        name: impl Into<String>,
        future: F,
    ) -> Result<TaskId, SchedulerError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.spawn(TaskKind::Monitor, name, future)
    }

    pub fn spawn_heartbeat<F>(
        &self,
        name: impl Into<String>,
        future: F,
    ) -> Result<TaskId, SchedulerError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.spawn(TaskKind::Heartbeat, name, future)
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
        self.workers.spawn(kind, name, future)
    }

    pub fn cancel(&self, id: TaskId) -> Result<(), SchedulerError> {
        self.workers.cancel(id)
    }

    pub async fn wait(&self, id: TaskId, timeout: Duration) -> Result<TaskStatus, SchedulerError> {
        self.workers.wait(id, timeout).await
    }

    pub fn shutdown(&self) {
        self.workers.abort_all();
    }

    pub async fn graceful_shutdown(&self, timeout: Duration) -> Result<(), SchedulerError> {
        self.workers.graceful_shutdown(timeout).await
    }

    pub fn workers(&self) -> Arc<WorkerPool> {
        Arc::clone(&self.workers)
    }

    pub fn statistics(&self) -> TaskStatistics {
        self.workers.statistics()
    }
}

impl Default for RuntimeScheduler {
    fn default() -> Self {
        Self::new(4096)
    }
}
