//! TunnelRuntime assembly and builder.

use crate::core::TunnelId;
use crate::runtime::config::{
    BufferConfig, ConnectorConfig, ListenerConfig, RetryConfig, RuntimeConfig, TimeoutConfig,
};
use crate::runtime::connector::TcpConnector;
use crate::runtime::context::RuntimeContext;
use crate::runtime::error::RuntimeError;
use crate::runtime::forward::ForwardPipeline;
use crate::runtime::lifecycle::RuntimeLifecycle;
use crate::runtime::listener::TcpListenerService;
use crate::runtime::monitor::{RuntimeMetrics, RuntimeMonitor};
use crate::runtime::scheduler::RuntimeScheduler;
use crate::runtime::state::{RuntimeState, SessionState};
use futures::future::BoxFuture;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::watch;
use tokio::time::sleep;
use tracing::info;

#[derive(Debug)]
struct TunnelRuntimeInner {
    context: Arc<RuntimeContext>,
    scheduler: Arc<RuntimeScheduler>,
    connector: TcpConnector,
    forward: ForwardPipeline,
    listener: TcpListenerService,
    monitor: Arc<RuntimeMonitor>,
}

/// TCP tunnel runtime entry point.
#[derive(Debug, Clone)]
pub struct TunnelRuntime {
    inner: Arc<TunnelRuntimeInner>,
}

impl TunnelRuntime {
    pub fn builder() -> RuntimeBuilder {
        RuntimeBuilder::default()
    }

    pub fn new(config: RuntimeConfig) -> Self {
        let context = Arc::new(RuntimeContext::new(config.clone()));
        let scheduler = Arc::new(RuntimeScheduler::new(config.max_tasks));
        let connector = TcpConnector::new(
            config.connector.clone(),
            config.retry.clone(),
            config.timeout.clone(),
            Arc::clone(&context.traffic),
        );
        let forward = ForwardPipeline::new(
            Arc::clone(&context.traffic),
            Arc::clone(&context.buffers),
            config.timeout.clone(),
        );
        let listener = TcpListenerService::new(
            config.listener.clone(),
            Arc::clone(&context),
            Arc::clone(&scheduler),
            connector.clone(),
            forward.clone(),
        );
        let monitor = Arc::new(RuntimeMonitor::new(
            Arc::clone(&context.traffic),
            Arc::clone(&context.sessions),
        ));

        Self {
            inner: Arc::new(TunnelRuntimeInner {
                context,
                scheduler,
                connector,
                forward,
                listener,
                monitor,
            }),
        }
    }

    pub fn context(&self) -> Arc<RuntimeContext> {
        Arc::clone(&self.inner.context)
    }

    pub fn scheduler(&self) -> Arc<RuntimeScheduler> {
        Arc::clone(&self.inner.scheduler)
    }

    pub fn connector(&self) -> TcpConnector {
        self.inner.connector.clone()
    }

    pub fn forward(&self) -> ForwardPipeline {
        self.inner.forward.clone()
    }

    pub fn listener(&self) -> TcpListenerService {
        self.inner.listener.clone()
    }

    pub fn monitor(&self) -> Arc<RuntimeMonitor> {
        Arc::clone(&self.inner.monitor)
    }

    pub fn metrics(&self) -> RuntimeMetrics {
        self.inner.monitor.snapshot()
    }

    pub fn bound_addr(&self) -> Option<SocketAddr> {
        self.inner.listener.bound_addr()
    }

    pub fn tunnel_id(&self) -> TunnelId {
        self.inner.context.config.listener.tunnel_id
    }

    pub async fn start(&self) -> Result<(), RuntimeError> {
        match self.inner.context.state.current() {
            RuntimeState::Initializing | RuntimeState::Starting | RuntimeState::Running => {
                return Ok(())
            }
            RuntimeState::Paused => return self.resume().await,
            RuntimeState::Closed => {
                return Err(RuntimeError::InvalidStateTransition {
                    from: self.inner.context.state.current(),
                    to: RuntimeState::Starting,
                });
            }
            RuntimeState::Failed => {
                self.inner
                    .context
                    .state
                    .transition_to(RuntimeState::Restarting)?;
            }
            _ => {}
        }

        self.inner
            .context
            .state
            .transition_to(RuntimeState::Initializing)?;
        self.inner.context.reset_shutdown();
        let result = async {
            self.inner
                .context
                .state
                .transition_to(RuntimeState::Starting)?;
            self.inner.listener.start().await?;
            self.start_monitor_task()?;
            self.start_cleanup_task()?;
            self.inner
                .context
                .state
                .transition_to(RuntimeState::Running)?;
            Ok::<(), RuntimeError>(())
        }
        .await;

        if let Err(error) = result {
            let _ = self.inner.context.state.transition_to(RuntimeState::Failed);
            return Err(error);
        }

        info!(
            target: "gate_runtime",
            tunnel_id = %self.tunnel_id(),
            name = %self.inner.context.config.name,
            "Runtime Start"
        );
        Ok(())
    }

    pub async fn stop(&self) -> Result<(), RuntimeError> {
        match self.inner.context.state.current() {
            RuntimeState::Stopped => return Ok(()),
            RuntimeState::Closed => return Ok(()),
            _ => {}
        }

        self.inner
            .context
            .state
            .transition_to(RuntimeState::Stopping)?;
        self.inner.context.request_shutdown();
        self.inner.listener.stop().await?;
        self.inner
            .scheduler
            .graceful_shutdown(self.inner.context.config.timeout.shutdown_timeout)
            .await?;
        self.inner.context.sessions.close_all(SessionState::Closed);
        self.inner
            .context
            .state
            .transition_to(RuntimeState::Stopped)?;
        Ok(())
    }

    pub async fn restart(&self) -> Result<(), RuntimeError> {
        match self.inner.context.state.current() {
            RuntimeState::Running | RuntimeState::Paused => {
                self.inner
                    .context
                    .state
                    .transition_to(RuntimeState::Restarting)?;
                self.stop().await?;
            }
            RuntimeState::Created | RuntimeState::Stopped => {}
            RuntimeState::Failed => {
                self.inner
                    .context
                    .state
                    .transition_to(RuntimeState::Restarting)?;
            }
            RuntimeState::Closed => {
                return Err(RuntimeError::InvalidStateTransition {
                    from: RuntimeState::Closed,
                    to: RuntimeState::Starting,
                });
            }
            _ => {}
        }

        self.start().await
    }

    pub async fn pause(&self) -> Result<(), RuntimeError> {
        if self.inner.context.state.current() == RuntimeState::Paused {
            return Ok(());
        }

        self.inner
            .context
            .state
            .transition_to(RuntimeState::Paused)?;
        info!(
            target: "gate_runtime",
            tunnel_id = %self.tunnel_id(),
            "runtime paused"
        );
        Ok(())
    }

    pub async fn resume(&self) -> Result<(), RuntimeError> {
        if self.inner.context.state.current() == RuntimeState::Running {
            return Ok(());
        }

        self.inner
            .context
            .state
            .transition_to(RuntimeState::Running)?;
        info!(
            target: "gate_runtime",
            tunnel_id = %self.tunnel_id(),
            "runtime resumed"
        );
        Ok(())
    }

    pub async fn shutdown(&self) -> Result<(), RuntimeError> {
        if self.inner.context.state.current() == RuntimeState::Closed {
            return Ok(());
        }

        self.inner
            .context
            .state
            .transition_to(RuntimeState::Stopping)?;
        self.inner.context.request_shutdown();
        self.inner.listener.stop().await?;
        self.inner
            .scheduler
            .graceful_shutdown(self.inner.context.config.timeout.shutdown_timeout)
            .await?;
        self.inner.context.sessions.close_all(SessionState::Closed);
        self.inner
            .context
            .state
            .transition_to(RuntimeState::Closed)?;
        Ok(())
    }

    fn start_monitor_task(&self) -> Result<(), RuntimeError> {
        let context = Arc::clone(&self.inner.context);
        let monitor = Arc::clone(&self.inner.monitor);
        let interval = context
            .config
            .monitor_interval
            .max(Duration::from_millis(100));
        let tunnel_id = self.tunnel_id();

        self.inner
            .scheduler
            .spawn_monitor(format!("runtime-monitor:{tunnel_id}"), async move {
                let mut shutdown = context.subscribe_shutdown();
                loop {
                    tokio::select! {
                        _ = shutdown_notified(&mut shutdown) => break,
                        _ = sleep(interval) => {
                            let _ = monitor.snapshot();
                        }
                    }
                }
            })?;

        Ok(())
    }

    fn start_cleanup_task(&self) -> Result<(), RuntimeError> {
        let context = Arc::clone(&self.inner.context);
        let interval = context
            .config
            .cleanup_interval
            .max(Duration::from_millis(100));
        let tunnel_id = self.tunnel_id();

        self.inner
            .scheduler
            .spawn_cleanup(format!("runtime-cleanup:{tunnel_id}"), async move {
                let mut shutdown = context.subscribe_shutdown();
                loop {
                    tokio::select! {
                        _ = shutdown_notified(&mut shutdown) => break,
                        _ = sleep(interval) => {
                            let _ = context.traffic.drain_current();
                        }
                    }
                }
            })?;

        Ok(())
    }
}

impl RuntimeLifecycle for TunnelRuntime {
    fn state(&self) -> RuntimeState {
        self.inner.context.state.current()
    }

    fn start(&self) -> BoxFuture<'static, Result<(), RuntimeError>> {
        let runtime = self.clone();
        Box::pin(async move { TunnelRuntime::start(&runtime).await })
    }

    fn stop(&self) -> BoxFuture<'static, Result<(), RuntimeError>> {
        let runtime = self.clone();
        Box::pin(async move { TunnelRuntime::stop(&runtime).await })
    }

    fn restart(&self) -> BoxFuture<'static, Result<(), RuntimeError>> {
        let runtime = self.clone();
        Box::pin(async move { TunnelRuntime::restart(&runtime).await })
    }

    fn pause(&self) -> BoxFuture<'static, Result<(), RuntimeError>> {
        let runtime = self.clone();
        Box::pin(async move { TunnelRuntime::pause(&runtime).await })
    }

    fn resume(&self) -> BoxFuture<'static, Result<(), RuntimeError>> {
        let runtime = self.clone();
        Box::pin(async move { TunnelRuntime::resume(&runtime).await })
    }

    fn shutdown(&self) -> BoxFuture<'static, Result<(), RuntimeError>> {
        let runtime = self.clone();
        Box::pin(async move { TunnelRuntime::shutdown(&runtime).await })
    }
}

/// Builder for [`TunnelRuntime`].
#[derive(Debug, Clone)]
pub struct RuntimeBuilder {
    config: RuntimeConfig,
}

impl Default for RuntimeBuilder {
    fn default() -> Self {
        Self {
            config: RuntimeConfig::default(),
        }
    }
}

impl RuntimeBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn config(mut self, config: RuntimeConfig) -> Self {
        self.config = config;
        self
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    pub fn tunnel_id(mut self, tunnel_id: TunnelId) -> Self {
        self.config.listener.tunnel_id = tunnel_id;
        self
    }

    pub fn listen_addr(mut self, listen_addr: SocketAddr) -> Self {
        self.config.listener.listen_addr = listen_addr;
        self
    }

    pub fn target_addr(mut self, target_addr: SocketAddr) -> Self {
        self.config.connector.target_addr = target_addr;
        self
    }

    pub fn listener(mut self, listener: ListenerConfig) -> Self {
        self.config.listener = listener;
        self
    }

    pub fn connector(mut self, connector: ConnectorConfig) -> Self {
        self.config.connector = connector;
        self
    }

    pub fn buffer(mut self, buffer: BufferConfig) -> Self {
        self.config.buffer = buffer;
        self
    }

    pub fn retry(mut self, retry: RetryConfig) -> Self {
        self.config.retry = retry;
        self
    }

    pub fn timeout(mut self, timeout: TimeoutConfig) -> Self {
        self.config.timeout = timeout;
        self
    }

    pub fn build(self) -> TunnelRuntime {
        TunnelRuntime::new(self.config)
    }
}

async fn shutdown_notified(shutdown: &mut watch::Receiver<bool>) {
    if *shutdown.borrow() {
        return;
    }

    while shutdown.changed().await.is_ok() {
        if *shutdown.borrow() {
            return;
        }
    }
}
