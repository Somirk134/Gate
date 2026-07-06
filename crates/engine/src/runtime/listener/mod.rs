//! TCP listener service.

use crate::connection::ConnectionId;
use crate::listener::{ListenerId, ListenerStatus};
use crate::runtime::config::ListenerConfig;
use crate::runtime::connector::TcpConnector;
use crate::runtime::context::RuntimeContext;
use crate::runtime::error::{ForwardError, ListenerError, RuntimeError};
use crate::runtime::forward::ForwardPipeline;
use crate::runtime::scheduler::RuntimeScheduler;
use crate::runtime::state::{RuntimeState, SessionState};
use crate::runtime::worker::TaskId;
use parking_lot::RwLock;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener as TokioTcpListener, TcpStream};
use tokio::sync::watch;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

#[derive(Debug)]
struct TcpListenerServiceInner {
    id: ListenerId,
    config: ListenerConfig,
    context: Arc<RuntimeContext>,
    scheduler: Arc<RuntimeScheduler>,
    connector: TcpConnector,
    forward: ForwardPipeline,
    status: RwLock<ListenerStatus>,
    task_id: RwLock<Option<TaskId>>,
    bound_addr: RwLock<Option<SocketAddr>>,
}

/// TCP listener service responsible for accept and session creation.
#[derive(Debug, Clone)]
pub struct TcpListenerService {
    inner: Arc<TcpListenerServiceInner>,
}

impl TcpListenerService {
    pub fn new(
        config: ListenerConfig,
        context: Arc<RuntimeContext>,
        scheduler: Arc<RuntimeScheduler>,
        connector: TcpConnector,
        forward: ForwardPipeline,
    ) -> Self {
        Self {
            inner: Arc::new(TcpListenerServiceInner {
                id: ListenerId::new(),
                config,
                context,
                scheduler,
                connector,
                forward,
                status: RwLock::new(ListenerStatus::Created),
                task_id: RwLock::new(None),
                bound_addr: RwLock::new(None),
            }),
        }
    }

    pub fn id(&self) -> ListenerId {
        self.inner.id
    }

    pub fn status(&self) -> ListenerStatus {
        *self.inner.status.read()
    }

    pub fn bound_addr(&self) -> Option<SocketAddr> {
        *self.inner.bound_addr.read()
    }

    pub async fn start(&self) -> Result<(), ListenerError> {
        if self.inner.task_id.read().is_some() {
            return Ok(());
        }

        let listen_addr = self.inner.config.listen_addr;
        let listener = TokioTcpListener::bind(listen_addr)
            .await
            .map_err(|source| ListenerError::Bind {
                addr: listen_addr,
                source,
            })?;
        let bound_addr = listener
            .local_addr()
            .map_err(|source| ListenerError::Bind {
                addr: listen_addr,
                source,
            })?;

        *self.inner.bound_addr.write() = Some(bound_addr);
        *self.inner.status.write() = ListenerStatus::Listening;

        let service = self.clone();
        let task_id = self.inner.scheduler.spawn_listener(
            format!("tcp-listener:{bound_addr}"),
            async move {
                service.accept_loop(listener, bound_addr).await;
            },
        )?;
        *self.inner.task_id.write() = Some(task_id);

        info!(
            target: "gate_runtime",
            listener_id = %self.id(),
            addr = %bound_addr,
            "Runtime Start"
        );

        Ok(())
    }

    pub async fn stop(&self) -> Result<(), ListenerError> {
        if let Some(task_id) = self.inner.task_id.write().take() {
            let _ = self.inner.scheduler.cancel(task_id);
        }
        *self.inner.status.write() = ListenerStatus::Stopped;
        Ok(())
    }

    async fn accept_loop(self, listener: TokioTcpListener, bound_addr: SocketAddr) {
        let mut shutdown = self.inner.context.subscribe_shutdown();

        loop {
            if *shutdown.borrow() {
                break;
            }

            if self.inner.context.state.current() == RuntimeState::Paused {
                sleep(self.inner.config.accept_backoff).await;
                continue;
            }

            tokio::select! {
                _ = shutdown_notified(&mut shutdown) => break,
                accepted = listener.accept() => match accepted {
                    Ok((stream, remote_addr)) => {
                        if let Err(error) = self.handle_client(stream, remote_addr).await {
                            self.inner.context.traffic.increment_error();
                            error!(
                                target: "gate_runtime",
                                addr = %bound_addr,
                                error = %error,
                                "listener client handling failed"
                            );
                        }
                    }
                    Err(source) => {
                        self.inner.context.traffic.increment_error();
                        warn!(
                            target: "gate_runtime",
                            addr = %bound_addr,
                            error = %source,
                            "listener accept failed"
                        );
                        sleep(self.inner.config.accept_backoff).await;
                    }
                }
            }
        }

        *self.inner.status.write() = ListenerStatus::Stopped;
        debug!(
            target: "gate_runtime",
            listener_id = %self.id(),
            addr = %bound_addr,
            "listener stopped"
        );
    }

    async fn handle_client(
        &self,
        client: TcpStream,
        remote_addr: SocketAddr,
    ) -> Result<(), RuntimeError> {
        if self.inner.context.state.current() == RuntimeState::Paused {
            return Err(RuntimeError::Paused);
        }

        let max_sessions = self
            .inner
            .context
            .config
            .max_sessions
            .min(self.inner.config.max_connections);
        if self.inner.context.sessions.active_count() >= max_sessions {
            return Err(RuntimeError::InvalidConfig {
                reason: format!("max session limit reached: {max_sessions}"),
            });
        }

        if self.inner.config.tcp_nodelay {
            client.set_nodelay(true).map_err(|source| {
                RuntimeError::Listener(ListenerError::Lifecycle {
                    reason: format!("failed to set TCP_NODELAY: {source}"),
                })
            })?;
        }

        let local_addr = client.local_addr().unwrap_or(self.inner.config.listen_addr);
        let session = self.inner.context.sessions.create(
            self.inner.config.tunnel_id,
            ConnectionId::new(),
            remote_addr,
            local_addr,
        );

        info!(
            target: "gate_runtime",
            session_id = %session.id,
            tunnel_id = %session.tunnel_id,
            remote_addr = %remote_addr,
            local_addr = %local_addr,
            "Session Create"
        );

        let connector = self.inner.connector.clone();
        let forward = self.inner.forward.clone();
        let sessions = Arc::clone(&self.inner.context.sessions);
        let traffic = Arc::clone(&self.inner.context.traffic);
        let shutdown = self.inner.context.subscribe_shutdown();
        let session_id = session.id;

        self.inner
            .scheduler
            .spawn_forward(format!("tcp-forward:{session_id}"), async move {
                session.set_status(SessionState::Connecting);
                let result = async {
                    let target = connector.connect().await?;
                    forward
                        .forward(Arc::clone(&session), client, target, shutdown)
                        .await?;
                    Ok::<(), RuntimeError>(())
                }
                .await;

                match result {
                    Ok(()) => {
                        session.set_status(SessionState::Closing);
                        let _ = sessions.close(&session_id, SessionState::Closed);
                        info!(
                            target: "gate_runtime",
                            session_id = %session_id,
                            "Session Close"
                        );
                    }
                    Err(RuntimeError::Forward(ForwardError::Shutdown)) => {
                        session.set_status(SessionState::Closing);
                        let _ = sessions.close(&session_id, SessionState::Closed);
                        info!(
                            target: "gate_runtime",
                            session_id = %session_id,
                            "Session Close"
                        );
                    }
                    Err(error) => {
                        traffic.increment_error();
                        let _ = sessions.close(&session_id, SessionState::Failed);
                        error!(
                            target: "gate_runtime",
                            session_id = %session_id,
                            error = %error,
                            "Session Close"
                        );
                    }
                }
            })?;

        Ok(())
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
