//! TCP forwarding pipeline.

use crate::runtime::buffer::BufferPool;
use crate::runtime::config::TimeoutConfig;
use crate::runtime::error::ForwardError;
use crate::runtime::monitor::TrafficStatistics;
use crate::runtime::session::Session;
use crate::runtime::state::{ForwardState, ForwardStateMachine, SessionState};
use crate::runtime::stream::{InstrumentedStream, StreamRole, StreamStatistics};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::copy_bidirectional;
use tokio::net::TcpStream;
use tokio::sync::watch;
use tokio::time::sleep;
use tracing::{debug, info};

/// Result of one bidirectional forwarding session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardResult {
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub duration: Duration,
}

#[derive(Debug)]
struct ForwardPipelineInner {
    state: RwLock<ForwardState>,
    statistics: Arc<TrafficStatistics>,
    buffers: Arc<BufferPool>,
    timeout: TimeoutConfig,
}

/// V1 TCP bidirectional forwarding pipeline.
#[derive(Debug, Clone)]
pub struct ForwardPipeline {
    inner: Arc<ForwardPipelineInner>,
}

impl ForwardPipeline {
    pub fn new(
        statistics: Arc<TrafficStatistics>,
        buffers: Arc<BufferPool>,
        timeout: TimeoutConfig,
    ) -> Self {
        Self {
            inner: Arc::new(ForwardPipelineInner {
                state: RwLock::new(ForwardState::Created),
                statistics,
                buffers,
                timeout,
            }),
        }
    }

    pub fn state(&self) -> ForwardState {
        *self.inner.state.read()
    }

    pub fn buffers(&self) -> Arc<BufferPool> {
        Arc::clone(&self.inner.buffers)
    }

    pub async fn forward(
        &self,
        session: Arc<Session>,
        client: TcpStream,
        target: TcpStream,
        mut shutdown: watch::Receiver<bool>,
    ) -> Result<ForwardResult, ForwardError> {
        let state = ForwardStateMachine::default();
        let _ = state.transition_to(ForwardState::Starting);
        self.set_state(ForwardState::Starting);
        let _ = state.transition_to(ForwardState::Running);
        self.set_state(ForwardState::Running);
        session.set_status(SessionState::Forwarding);

        let started = Instant::now();
        let client_statistics = Arc::new(StreamStatistics::new());
        let target_statistics = Arc::new(StreamStatistics::new());
        let session_traffic = session.traffic();

        let mut client = InstrumentedStream::new(
            client,
            StreamRole::Client,
            Arc::clone(&client_statistics),
            Arc::clone(&self.inner.statistics),
            Arc::clone(&session_traffic),
        );
        let mut target = InstrumentedStream::new(
            target,
            StreamRole::Target,
            Arc::clone(&target_statistics),
            Arc::clone(&self.inner.statistics),
            session_traffic,
        );

        info!(
            target: "gate_runtime",
            session_id = %session.id,
            tunnel_id = %session.tunnel_id,
            "Forward Started"
        );

        let idle_timeout = self.inner.timeout.idle_timeout;
        let idle = wait_for_idle(
            Arc::clone(&client_statistics),
            Arc::clone(&target_statistics),
            idle_timeout,
        );
        tokio::pin!(idle);

        let copy = copy_bidirectional(&mut client, &mut target);
        tokio::pin!(copy);

        let result = tokio::select! {
            result = &mut copy => match result {
                Ok((upload_bytes, download_bytes)) => {
                    let _ = state.transition_to(ForwardState::Finishing);
                    self.set_state(ForwardState::Finishing);
                    Ok(ForwardResult {
                        upload_bytes,
                        download_bytes,
                        duration: Instant::now().saturating_duration_since(started),
                    })
                }
                Err(source) => {
                    self.inner.statistics.increment_error();
                    let _ = state.transition_to(ForwardState::Failed);
                    self.set_state(ForwardState::Failed);
                    Err(ForwardError::Io { source })
                }
            },
            _ = shutdown_notified(&mut shutdown) => {
                let _ = state.transition_to(ForwardState::Shutdown);
                self.set_state(ForwardState::Shutdown);
                Err(ForwardError::Shutdown)
            },
            _ = &mut idle => {
                self.inner.statistics.increment_error();
                let _ = state.transition_to(ForwardState::Failed);
                self.set_state(ForwardState::Failed);
                Err(ForwardError::IdleTimeout { timeout: idle_timeout })
            }
        };

        match &result {
            Ok(result) => {
                let _ = state.transition_to(ForwardState::Finished);
                self.set_state(ForwardState::Finished);
                info!(
                    target: "gate_runtime",
                    session_id = %session.id,
                    tunnel_id = %session.tunnel_id,
                    upload_bytes = result.upload_bytes,
                    download_bytes = result.download_bytes,
                    "Forward Finished"
                );
            }
            Err(error) => {
                debug!(
                    target: "gate_runtime",
                    session_id = %session.id,
                    tunnel_id = %session.tunnel_id,
                    error = %error,
                    "Forward Finished"
                );
            }
        }

        result
    }

    fn set_state(&self, state: ForwardState) {
        *self.inner.state.write() = state;
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

async fn wait_for_idle(
    client_statistics: Arc<StreamStatistics>,
    target_statistics: Arc<StreamStatistics>,
    timeout: Duration,
) {
    if timeout.is_zero() {
        std::future::pending::<()>().await;
        return;
    }

    let check_every = idle_check_interval(timeout);
    loop {
        sleep(check_every).await;
        let last_activity = client_statistics
            .last_activity_millis()
            .max(target_statistics.last_activity_millis());
        let elapsed = now_millis().saturating_sub(last_activity);
        if elapsed >= timeout.as_millis().min(u64::MAX as u128) as u64 {
            return;
        }
    }
}

fn idle_check_interval(timeout: Duration) -> Duration {
    let half = Duration::from_millis((timeout.as_millis() / 2).min(u64::MAX as u128) as u64);
    half.min(Duration::from_secs(1)).max(Duration::from_millis(50))
}

fn now_millis() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}
