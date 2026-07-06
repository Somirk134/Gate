//! Unified graceful shutdown orchestration.

use futures::future::BoxFuture;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ShutdownResourceKind {
    Listener,
    Task,
    Connection,
    Channel,
    Timer,
    File,
    Tls,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShutdownResourceResult {
    pub name: String,
    pub kind: ShutdownResourceKind,
    pub success: bool,
    pub elapsed_millis: u64,
    pub message: Option<String>,
}

pub type ShutdownHook = Arc<dyn Fn() -> BoxFuture<'static, Result<(), String>> + Send + Sync>;

#[derive(Clone)]
pub struct ShutdownResource {
    name: String,
    kind: ShutdownResourceKind,
    hook: ShutdownHook,
}

impl fmt::Debug for ShutdownResource {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ShutdownResource")
            .field("name", &self.name)
            .field("kind", &self.kind)
            .finish_non_exhaustive()
    }
}

impl ShutdownResource {
    pub fn new(name: impl Into<String>, kind: ShutdownResourceKind, hook: ShutdownHook) -> Self {
        Self {
            name: name.into(),
            kind,
            hook,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn kind(&self) -> ShutdownResourceKind {
        self.kind
    }

    async fn shutdown(&self, shutdown_timeout: Duration) -> ShutdownResourceResult {
        let started = Instant::now();
        let outcome = timeout(shutdown_timeout, (self.hook)()).await;
        let elapsed_millis = started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64;

        match outcome {
            Ok(Ok(())) => ShutdownResourceResult {
                name: self.name.clone(),
                kind: self.kind,
                success: true,
                elapsed_millis,
                message: None,
            },
            Ok(Err(message)) => ShutdownResourceResult {
                name: self.name.clone(),
                kind: self.kind,
                success: false,
                elapsed_millis,
                message: Some(message),
            },
            Err(_) => ShutdownResourceResult {
                name: self.name.clone(),
                kind: self.kind,
                success: false,
                elapsed_millis,
                message: Some(format!("shutdown timed out after {shutdown_timeout:?}")),
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GracefulShutdownReport {
    pub success: bool,
    pub elapsed_millis: u64,
    pub resources: Vec<ShutdownResourceResult>,
}

#[derive(Debug, Default)]
pub struct GracefulShutdownManager {
    resources: RwLock<Vec<ShutdownResource>>,
}

impl GracefulShutdownManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&self, resource: ShutdownResource) {
        self.resources.write().push(resource);
    }

    pub fn registered_count(&self) -> usize {
        self.resources.read().len()
    }

    pub async fn shutdown(&self, shutdown_timeout: Duration) -> GracefulShutdownReport {
        let started = Instant::now();
        let resources = self.resources.read().clone();
        let mut results = Vec::with_capacity(resources.len());

        for resource in resources {
            results.push(resource.shutdown(shutdown_timeout).await);
        }

        GracefulShutdownReport {
            success: results.iter().all(|result| result.success),
            elapsed_millis: started.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
            resources: results,
        }
    }
}
