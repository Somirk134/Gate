//! Runtime manager.

use crate::core::TunnelId;
use crate::error::EngineError;
use crate::runtime::config::RuntimeConfig;
use crate::runtime::error::RuntimeError;
use crate::runtime::http::{HttpRuntimeConfig, HttpTunnelRuntime};
use crate::runtime::https::{HttpsRuntimeConfig, HttpsTunnelRuntime};
use crate::runtime::scheduler::RuntimeScheduler;
use crate::runtime::tunnel_runtime::TunnelRuntime;
use crate::runtime::worker::TaskId;
use anyhow::anyhow;
use dashmap::DashMap;
use std::future::Future;
use std::sync::Arc;

/// Manages tunnel runtimes and keeps backwards-compatible task APIs.
#[derive(Debug)]
pub struct RuntimeManager {
    config: RuntimeConfig,
    scheduler: Arc<RuntimeScheduler>,
    runtimes: DashMap<TunnelId, Arc<TunnelRuntime>>,
    http_runtimes: DashMap<TunnelId, Arc<HttpTunnelRuntime>>,
    https_runtimes: DashMap<TunnelId, Arc<HttpsTunnelRuntime>>,
}

impl RuntimeManager {
    pub fn new<C>(config: C) -> Self
    where
        C: Into<RuntimeConfig>,
    {
        let config = config.into();
        Self {
            scheduler: Arc::new(RuntimeScheduler::new(config.max_tasks)),
            config,
            runtimes: DashMap::new(),
            http_runtimes: DashMap::new(),
            https_runtimes: DashMap::new(),
        }
    }

    pub fn create_runtime(&self, config: RuntimeConfig) -> Arc<TunnelRuntime> {
        let runtime = Arc::new(TunnelRuntime::new(config));
        self.runtimes
            .insert(runtime.tunnel_id(), Arc::clone(&runtime));
        runtime
    }

    pub fn register_runtime(&self, runtime: TunnelRuntime) -> Arc<TunnelRuntime> {
        let runtime = Arc::new(runtime);
        self.runtimes
            .insert(runtime.tunnel_id(), Arc::clone(&runtime));
        runtime
    }

    pub fn get_runtime(&self, tunnel_id: &TunnelId) -> Option<Arc<TunnelRuntime>> {
        self.runtimes
            .get(tunnel_id)
            .map(|entry| Arc::clone(entry.value()))
    }

    pub fn remove_runtime(&self, tunnel_id: &TunnelId) -> Option<Arc<TunnelRuntime>> {
        self.runtimes.remove(tunnel_id).map(|(_, runtime)| runtime)
    }

    pub fn create_http_runtime(&self, config: HttpRuntimeConfig) -> Arc<HttpTunnelRuntime> {
        let runtime = Arc::new(HttpTunnelRuntime::new(config));
        self.http_runtimes
            .insert(runtime.tunnel_id(), Arc::clone(&runtime));
        runtime
    }

    pub fn register_http_runtime(&self, runtime: HttpTunnelRuntime) -> Arc<HttpTunnelRuntime> {
        let runtime = Arc::new(runtime);
        self.http_runtimes
            .insert(runtime.tunnel_id(), Arc::clone(&runtime));
        runtime
    }

    pub fn get_http_runtime(&self, tunnel_id: &TunnelId) -> Option<Arc<HttpTunnelRuntime>> {
        self.http_runtimes
            .get(tunnel_id)
            .map(|entry| Arc::clone(entry.value()))
    }

    pub fn remove_http_runtime(&self, tunnel_id: &TunnelId) -> Option<Arc<HttpTunnelRuntime>> {
        self.http_runtimes
            .remove(tunnel_id)
            .map(|(_, runtime)| runtime)
    }

    pub fn create_https_runtime(&self, config: HttpsRuntimeConfig) -> Arc<HttpsTunnelRuntime> {
        let runtime = Arc::new(HttpsTunnelRuntime::new(config));
        self.https_runtimes
            .insert(runtime.tunnel_id(), Arc::clone(&runtime));
        runtime
    }

    pub fn register_https_runtime(&self, runtime: HttpsTunnelRuntime) -> Arc<HttpsTunnelRuntime> {
        let runtime = Arc::new(runtime);
        self.https_runtimes
            .insert(runtime.tunnel_id(), Arc::clone(&runtime));
        runtime
    }

    pub fn get_https_runtime(&self, tunnel_id: &TunnelId) -> Option<Arc<HttpsTunnelRuntime>> {
        self.https_runtimes
            .get(tunnel_id)
            .map(|entry| Arc::clone(entry.value()))
    }

    pub fn remove_https_runtime(&self, tunnel_id: &TunnelId) -> Option<Arc<HttpsTunnelRuntime>> {
        self.https_runtimes
            .remove(tunnel_id)
            .map(|(_, runtime)| runtime)
    }

    pub async fn start_runtime(
        &self,
        config: RuntimeConfig,
    ) -> Result<Arc<TunnelRuntime>, RuntimeError> {
        let runtime = self.create_runtime(config);
        runtime.start().await?;
        Ok(runtime)
    }

    pub async fn stop_runtime(&self, tunnel_id: &TunnelId) -> Result<(), RuntimeError> {
        if let Some(runtime) = self.get_runtime(tunnel_id) {
            runtime.stop().await?;
        }
        Ok(())
    }

    pub async fn start_http_runtime(
        &self,
        config: HttpRuntimeConfig,
    ) -> Result<Arc<HttpTunnelRuntime>, RuntimeError> {
        let runtime = self.create_http_runtime(config);
        runtime.start().await?;
        Ok(runtime)
    }

    pub async fn stop_http_runtime(&self, tunnel_id: &TunnelId) -> Result<(), RuntimeError> {
        if let Some(runtime) = self.get_http_runtime(tunnel_id) {
            runtime.stop().await?;
        }
        Ok(())
    }

    pub async fn start_https_runtime(
        &self,
        config: HttpsRuntimeConfig,
    ) -> Result<Arc<HttpsTunnelRuntime>, RuntimeError> {
        let runtime = self.create_https_runtime(config);
        runtime.start().await?;
        Ok(runtime)
    }

    pub async fn stop_https_runtime(&self, tunnel_id: &TunnelId) -> Result<(), RuntimeError> {
        if let Some(runtime) = self.get_https_runtime(tunnel_id) {
            runtime.stop().await?;
        }
        Ok(())
    }

    pub fn spawn<F>(&self, name: impl Into<String>, future: F) -> Result<TaskId, EngineError>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.scheduler
            .spawn_runtime(name, future)
            .map_err(|error| EngineError::Internal(anyhow!(error)))
    }

    pub fn cancel(&self, id: TaskId) -> Result<(), EngineError> {
        self.scheduler
            .cancel(id)
            .map_err(|error| EngineError::Internal(anyhow!(error)))
    }

    pub fn shutdown(&self) -> Result<(), EngineError> {
        for runtime in self.runtimes.iter() {
            runtime.context().request_shutdown();
            runtime.scheduler().shutdown();
        }
        for runtime in self.http_runtimes.iter() {
            runtime.context().request_shutdown();
            runtime.scheduler().shutdown();
        }
        for runtime in self.https_runtimes.iter() {
            runtime.context().request_shutdown();
            runtime.scheduler().shutdown();
        }
        self.scheduler.shutdown();
        Ok(())
    }

    pub async fn graceful_shutdown(&self) -> Result<(), EngineError> {
        let runtimes: Vec<Arc<TunnelRuntime>> = self
            .runtimes
            .iter()
            .map(|entry| Arc::clone(entry.value()))
            .collect();

        for runtime in runtimes {
            runtime
                .shutdown()
                .await
                .map_err(|error| EngineError::Internal(anyhow!(error)))?;
        }

        let http_runtimes: Vec<Arc<HttpTunnelRuntime>> = self
            .http_runtimes
            .iter()
            .map(|entry| Arc::clone(entry.value()))
            .collect();

        for runtime in http_runtimes {
            runtime
                .shutdown()
                .await
                .map_err(|error| EngineError::Internal(anyhow!(error)))?;
        }

        let https_runtimes: Vec<Arc<HttpsTunnelRuntime>> = self
            .https_runtimes
            .iter()
            .map(|entry| Arc::clone(entry.value()))
            .collect();

        for runtime in https_runtimes {
            runtime
                .shutdown()
                .await
                .map_err(|error| EngineError::Internal(anyhow!(error)))?;
        }

        self.scheduler
            .graceful_shutdown(self.config.timeout.shutdown_timeout)
            .await
            .map_err(|error| EngineError::Internal(anyhow!(error)))
    }

    pub fn config(&self) -> &RuntimeConfig {
        &self.config
    }

    pub fn scheduler(&self) -> Arc<RuntimeScheduler> {
        Arc::clone(&self.scheduler)
    }
}

impl Default for RuntimeManager {
    fn default() -> Self {
        Self::new(RuntimeConfig::default())
    }
}
