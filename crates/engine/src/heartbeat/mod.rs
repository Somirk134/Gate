//! Event-driven heartbeat state machine.
//!
//! The heartbeat manager tracks ping/pong lifecycle and timeout decisions. It
//! intentionally does not perform network I/O; transport adapters are expected
//! to call [`HeartbeatManager::ping`] before sending a protocol ping and
//! [`HeartbeatManager::pong`] after receiving a protocol pong.

use crate::config::HeartbeatConfig;
use crate::core::TunnelId;
use crate::error::HeartbeatError;
use crate::event::{EventPublisher, TunnelEvent};
use dashmap::DashMap;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Heartbeat lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeartbeatState {
    Idle,
    Running,
    WaitingPong,
    Timeout,
    Retrying,
    Stopped,
}

impl HeartbeatState {
    fn as_static_str(self) -> &'static str {
        match self {
            Self::Idle => "Idle",
            Self::Running => "Running",
            Self::WaitingPong => "WaitingPong",
            Self::Timeout => "Timeout",
            Self::Retrying => "Retrying",
            Self::Stopped => "Stopped",
        }
    }
}

/// Heartbeat counters and RTT aggregates.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HeartbeatMetrics {
    pub ping_count: u64,
    pub pong_count: u64,
    pub timeout_count: u64,
    pub retry_count: u64,
    pub heartbeat_count: u64,
    pub total_rtt_ms: u64,
    pub last_rtt_ms: Option<u64>,
}

impl HeartbeatMetrics {
    pub fn average_rtt_ms(&self) -> f64 {
        if self.pong_count == 0 {
            return 0.0;
        }

        self.total_rtt_ms as f64 / self.pong_count as f64
    }
}

/// Serializable heartbeat view for monitoring and client APIs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeartbeatSnapshot {
    pub tunnel_id: TunnelId,
    pub state: HeartbeatState,
    pub sequence: u64,
    pub missed_heartbeat: u32,
    pub retry_attempt: u32,
    pub last_ping_millis: Option<u64>,
    pub last_pong_millis: Option<u64>,
    pub last_timeout_millis: Option<u64>,
    pub average_rtt_ms: f64,
    pub metrics: HeartbeatMetrics,
}

#[derive(Debug, Clone)]
struct HeartbeatRuntime {
    state: HeartbeatState,
    sequence: u64,
    missed_heartbeat: u32,
    retry_attempt: u32,
    last_ping_millis: Option<u64>,
    last_pong_millis: Option<u64>,
    last_timeout_millis: Option<u64>,
    last_ping_instant: Option<Instant>,
    last_timeout_instant: Option<Instant>,
    metrics: HeartbeatMetrics,
}

impl Default for HeartbeatRuntime {
    fn default() -> Self {
        Self {
            state: HeartbeatState::Idle,
            sequence: 0,
            missed_heartbeat: 0,
            retry_attempt: 0,
            last_ping_millis: None,
            last_pong_millis: None,
            last_timeout_millis: None,
            last_ping_instant: None,
            last_timeout_instant: None,
            metrics: HeartbeatMetrics::default(),
        }
    }
}

impl HeartbeatRuntime {
    fn snapshot(&self, tunnel_id: TunnelId) -> HeartbeatSnapshot {
        HeartbeatSnapshot {
            tunnel_id,
            state: self.state,
            sequence: self.sequence,
            missed_heartbeat: self.missed_heartbeat,
            retry_attempt: self.retry_attempt,
            last_ping_millis: self.last_ping_millis,
            last_pong_millis: self.last_pong_millis,
            last_timeout_millis: self.last_timeout_millis,
            average_rtt_ms: self.metrics.average_rtt_ms(),
            metrics: self.metrics.clone(),
        }
    }

    fn transition(&mut self, to: HeartbeatState) -> Result<(), HeartbeatError> {
        let from = self.state;
        let allowed = matches!(
            (from, to),
            (HeartbeatState::Idle, HeartbeatState::Running)
                | (HeartbeatState::Idle, HeartbeatState::Stopped)
                | (HeartbeatState::Running, HeartbeatState::Idle)
                | (HeartbeatState::Running, HeartbeatState::WaitingPong)
                | (HeartbeatState::Running, HeartbeatState::Stopped)
                | (HeartbeatState::WaitingPong, HeartbeatState::Idle)
                | (HeartbeatState::WaitingPong, HeartbeatState::Running)
                | (HeartbeatState::WaitingPong, HeartbeatState::Timeout)
                | (HeartbeatState::WaitingPong, HeartbeatState::Stopped)
                | (HeartbeatState::Timeout, HeartbeatState::Idle)
                | (HeartbeatState::Timeout, HeartbeatState::Running)
                | (HeartbeatState::Timeout, HeartbeatState::Retrying)
                | (HeartbeatState::Timeout, HeartbeatState::Stopped)
                | (HeartbeatState::Retrying, HeartbeatState::Idle)
                | (HeartbeatState::Retrying, HeartbeatState::Running)
                | (HeartbeatState::Retrying, HeartbeatState::WaitingPong)
                | (HeartbeatState::Retrying, HeartbeatState::Stopped)
                | (HeartbeatState::Stopped, HeartbeatState::Running)
                | (HeartbeatState::Stopped, HeartbeatState::Idle)
        );

        if !allowed && from != to {
            return Err(HeartbeatError::InvalidTransition {
                from: from.as_static_str(),
                to: to.as_static_str(),
            });
        }

        self.state = to;
        Ok(())
    }
}

/// Async heartbeat control contract.
pub trait Heartbeat: Send + Sync {
    fn start(
        &self,
        tunnel_id: TunnelId,
    ) -> BoxFuture<'_, Result<HeartbeatSnapshot, HeartbeatError>>;

    fn stop(&self, tunnel_id: TunnelId)
        -> BoxFuture<'_, Result<HeartbeatSnapshot, HeartbeatError>>;

    fn pause(
        &self,
        tunnel_id: TunnelId,
    ) -> BoxFuture<'_, Result<HeartbeatSnapshot, HeartbeatError>>;

    fn resume(
        &self,
        tunnel_id: TunnelId,
    ) -> BoxFuture<'_, Result<HeartbeatSnapshot, HeartbeatError>>;

    fn tick(&self, tunnel_id: TunnelId)
        -> BoxFuture<'_, Result<HeartbeatSnapshot, HeartbeatError>>;

    fn ping(&self, tunnel_id: TunnelId)
        -> BoxFuture<'_, Result<HeartbeatSnapshot, HeartbeatError>>;

    fn pong(
        &self,
        tunnel_id: TunnelId,
        sequence: u64,
    ) -> BoxFuture<'_, Result<HeartbeatSnapshot, HeartbeatError>>;

    fn timeout(
        &self,
        tunnel_id: TunnelId,
    ) -> BoxFuture<'_, Result<HeartbeatSnapshot, HeartbeatError>>;
}

/// In-memory heartbeat manager optimized for many independent connection
/// entries. DashMap keeps lock contention scoped to the tunnel shard.
#[derive(Debug)]
pub struct HeartbeatManager {
    config: HeartbeatConfig,
    states: DashMap<TunnelId, HeartbeatRuntime>,
    events: Option<EventPublisher>,
}

impl Default for HeartbeatManager {
    fn default() -> Self {
        Self {
            config: HeartbeatConfig::default(),
            states: DashMap::new(),
            events: None,
        }
    }
}

impl HeartbeatManager {
    pub fn new(config: HeartbeatConfig) -> Self {
        Self {
            config,
            states: DashMap::new(),
            events: None,
        }
    }

    pub fn with_events(config: HeartbeatConfig, events: EventPublisher) -> Self {
        Self {
            config,
            states: DashMap::new(),
            events: Some(events),
        }
    }

    pub async fn start(&self, tunnel_id: TunnelId) -> Result<HeartbeatSnapshot, HeartbeatError> {
        if let Some(mut runtime) = self.states.get_mut(&tunnel_id) {
            if matches!(
                runtime.state,
                HeartbeatState::Running | HeartbeatState::WaitingPong | HeartbeatState::Retrying
            ) {
                return Err(HeartbeatError::AlreadyRunning { tunnel_id });
            }

            runtime.transition(HeartbeatState::Running)?;
            runtime.missed_heartbeat = 0;
            runtime.retry_attempt = 0;
        } else {
            let mut runtime = HeartbeatRuntime::default();
            runtime.transition(HeartbeatState::Running)?;
            self.states.insert(tunnel_id, runtime);
        }

        self.publish(TunnelEvent::HeartbeatStarted { tunnel_id })
            .await;
        self.snapshot(tunnel_id)
    }

    pub async fn stop(&self, tunnel_id: TunnelId) -> Result<HeartbeatSnapshot, HeartbeatError> {
        let snapshot = {
            let mut runtime = self
                .states
                .get_mut(&tunnel_id)
                .ok_or(HeartbeatError::NotFound { tunnel_id })?;
            runtime.transition(HeartbeatState::Stopped)?;
            runtime.snapshot(tunnel_id)
        };

        self.publish(TunnelEvent::HeartbeatStopped { tunnel_id })
            .await;
        Ok(snapshot)
    }

    pub async fn pause(&self, tunnel_id: TunnelId) -> Result<HeartbeatSnapshot, HeartbeatError> {
        let mut runtime = self
            .states
            .get_mut(&tunnel_id)
            .ok_or(HeartbeatError::NotFound { tunnel_id })?;
        runtime.transition(HeartbeatState::Idle)?;
        Ok(runtime.snapshot(tunnel_id))
    }

    pub async fn resume(&self, tunnel_id: TunnelId) -> Result<HeartbeatSnapshot, HeartbeatError> {
        let mut runtime = self
            .states
            .get_mut(&tunnel_id)
            .ok_or(HeartbeatError::NotFound { tunnel_id })?;
        runtime.transition(HeartbeatState::Running)?;
        Ok(runtime.snapshot(tunnel_id))
    }

    pub async fn tick(&self, tunnel_id: TunnelId) -> Result<HeartbeatSnapshot, HeartbeatError> {
        let mut timeout_event = false;
        let snapshot = {
            let mut runtime = self
                .states
                .get_mut(&tunnel_id)
                .ok_or(HeartbeatError::NotFound { tunnel_id })?;

            match runtime.state {
                HeartbeatState::Running => {
                    Self::mark_ping(&mut runtime)?;
                }
                HeartbeatState::WaitingPong => {
                    let timed_out = runtime
                        .last_ping_instant
                        .map(|last_ping| last_ping.elapsed() >= self.config.timeout)
                        .unwrap_or(false);
                    let missed_too_many =
                        runtime.missed_heartbeat >= self.config.max_missed_heartbeat;

                    if timed_out || missed_too_many {
                        Self::mark_timeout(&mut runtime)?;
                        timeout_event = true;
                    }
                }
                HeartbeatState::Timeout => {
                    let can_retry = runtime.retry_attempt < self.config.retry_count;
                    let retry_ready = runtime
                        .last_timeout_instant
                        .map(|last_timeout| last_timeout.elapsed() >= self.config.retry_delay)
                        .unwrap_or(true);

                    if can_retry && retry_ready {
                        runtime.transition(HeartbeatState::Retrying)?;
                        runtime.retry_attempt += 1;
                        runtime.metrics.retry_count += 1;
                        Self::mark_ping(&mut runtime)?;
                    }
                }
                HeartbeatState::Retrying => {
                    let retry_ready = runtime
                        .last_ping_instant
                        .map(|last_ping| last_ping.elapsed() >= self.config.retry_delay)
                        .unwrap_or(true);

                    if retry_ready {
                        Self::mark_ping(&mut runtime)?;
                    }
                }
                HeartbeatState::Idle | HeartbeatState::Stopped => {}
            }

            runtime.snapshot(tunnel_id)
        };

        if timeout_event {
            self.publish(TunnelEvent::HeartbeatTimeout { tunnel_id })
                .await;
        }

        Ok(snapshot)
    }

    pub async fn ping(&self, tunnel_id: TunnelId) -> Result<HeartbeatSnapshot, HeartbeatError> {
        let mut runtime = self
            .states
            .get_mut(&tunnel_id)
            .ok_or(HeartbeatError::NotFound { tunnel_id })?;
        Self::mark_ping(&mut runtime)?;
        Ok(runtime.snapshot(tunnel_id))
    }

    pub async fn pong(
        &self,
        tunnel_id: TunnelId,
        _sequence: u64,
    ) -> Result<HeartbeatSnapshot, HeartbeatError> {
        let mut runtime = self
            .states
            .get_mut(&tunnel_id)
            .ok_or(HeartbeatError::NotFound { tunnel_id })?;

        if !matches!(
            runtime.state,
            HeartbeatState::WaitingPong | HeartbeatState::Retrying | HeartbeatState::Timeout
        ) {
            return Err(HeartbeatError::NotRunning { tunnel_id });
        }

        let rtt_ms = runtime
            .last_ping_instant
            .map(|last_ping| last_ping.elapsed().as_millis() as u64)
            .unwrap_or_default();

        runtime.transition(HeartbeatState::Running)?;
        runtime.missed_heartbeat = 0;
        runtime.retry_attempt = 0;
        runtime.last_pong_millis = Some(now_millis());
        runtime.metrics.pong_count += 1;
        runtime.metrics.last_rtt_ms = Some(rtt_ms);
        runtime.metrics.total_rtt_ms = runtime.metrics.total_rtt_ms.saturating_add(rtt_ms);
        Ok(runtime.snapshot(tunnel_id))
    }

    pub async fn timeout(&self, tunnel_id: TunnelId) -> Result<HeartbeatSnapshot, HeartbeatError> {
        let snapshot = {
            let mut runtime = self
                .states
                .get_mut(&tunnel_id)
                .ok_or(HeartbeatError::NotFound { tunnel_id })?;
            Self::mark_timeout(&mut runtime)?;
            runtime.snapshot(tunnel_id)
        };

        self.publish(TunnelEvent::HeartbeatTimeout { tunnel_id })
            .await;
        Ok(snapshot)
    }

    pub fn snapshot(&self, tunnel_id: TunnelId) -> Result<HeartbeatSnapshot, HeartbeatError> {
        self.states
            .get(&tunnel_id)
            .map(|runtime| runtime.snapshot(tunnel_id))
            .ok_or(HeartbeatError::NotFound { tunnel_id })
    }

    pub fn len(&self) -> usize {
        self.states.len()
    }

    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    pub fn config(&self) -> &HeartbeatConfig {
        &self.config
    }

    fn mark_ping(runtime: &mut HeartbeatRuntime) -> Result<(), HeartbeatError> {
        runtime.sequence = runtime.sequence.saturating_add(1);
        runtime.last_ping_millis = Some(now_millis());
        runtime.last_ping_instant = Some(Instant::now());
        runtime.metrics.ping_count = runtime.metrics.ping_count.saturating_add(1);
        runtime.metrics.heartbeat_count = runtime.metrics.heartbeat_count.saturating_add(1);

        runtime.transition(HeartbeatState::WaitingPong)?;
        Ok(())
    }

    fn mark_timeout(runtime: &mut HeartbeatRuntime) -> Result<(), HeartbeatError> {
        runtime.transition(HeartbeatState::Timeout)?;
        runtime.missed_heartbeat = runtime.missed_heartbeat.saturating_add(1);
        runtime.last_timeout_millis = Some(now_millis());
        runtime.last_timeout_instant = Some(Instant::now());
        runtime.metrics.timeout_count = runtime.metrics.timeout_count.saturating_add(1);
        Ok(())
    }

    async fn publish(&self, event: TunnelEvent) {
        if let Some(publisher) = &self.events {
            let _ = publisher.send(event).await;
        }
    }
}

impl Heartbeat for HeartbeatManager {
    fn start(
        &self,
        tunnel_id: TunnelId,
    ) -> BoxFuture<'_, Result<HeartbeatSnapshot, HeartbeatError>> {
        Box::pin(async move { HeartbeatManager::start(self, tunnel_id).await })
    }

    fn stop(
        &self,
        tunnel_id: TunnelId,
    ) -> BoxFuture<'_, Result<HeartbeatSnapshot, HeartbeatError>> {
        Box::pin(async move { HeartbeatManager::stop(self, tunnel_id).await })
    }

    fn pause(
        &self,
        tunnel_id: TunnelId,
    ) -> BoxFuture<'_, Result<HeartbeatSnapshot, HeartbeatError>> {
        Box::pin(async move { HeartbeatManager::pause(self, tunnel_id).await })
    }

    fn resume(
        &self,
        tunnel_id: TunnelId,
    ) -> BoxFuture<'_, Result<HeartbeatSnapshot, HeartbeatError>> {
        Box::pin(async move { HeartbeatManager::resume(self, tunnel_id).await })
    }

    fn tick(
        &self,
        tunnel_id: TunnelId,
    ) -> BoxFuture<'_, Result<HeartbeatSnapshot, HeartbeatError>> {
        Box::pin(async move { HeartbeatManager::tick(self, tunnel_id).await })
    }

    fn ping(
        &self,
        tunnel_id: TunnelId,
    ) -> BoxFuture<'_, Result<HeartbeatSnapshot, HeartbeatError>> {
        Box::pin(async move { HeartbeatManager::ping(self, tunnel_id).await })
    }

    fn pong(
        &self,
        tunnel_id: TunnelId,
        sequence: u64,
    ) -> BoxFuture<'_, Result<HeartbeatSnapshot, HeartbeatError>> {
        Box::pin(async move { HeartbeatManager::pong(self, tunnel_id, sequence).await })
    }

    fn timeout(
        &self,
        tunnel_id: TunnelId,
    ) -> BoxFuture<'_, Result<HeartbeatSnapshot, HeartbeatError>> {
        Box::pin(async move { HeartbeatManager::timeout(self, tunnel_id).await })
    }
}

fn now_millis() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}
