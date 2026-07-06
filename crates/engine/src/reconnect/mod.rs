//! Reconnect queue, strategy, and scheduler boundaries.
//!
//! The reconnect manager does not perform network dial attempts. It prepares
//! retry work, computes delay, publishes lifecycle events, and records the
//! outcome reported by a transport adapter.

use crate::config::{ReconnectConfig, ReconnectStrategyConfig};
use crate::connection::ConnectionId;
use crate::core::TunnelId;
use crate::error::ReconnectError;
use crate::event::{EventPublisher, TunnelEvent};
use dashmap::DashMap;
use futures::future::BoxFuture;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Duration;

/// Reconnect trigger mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReconnectMode {
    Auto,
    Manual,
}

/// Reconnect lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReconnectState {
    Idle,
    Queued,
    Scheduling,
    Reconnecting,
    Succeeded,
    Failed,
    Cancelled,
}

/// Serializable strategy kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReconnectStrategyKind {
    Immediate,
    Linear,
    ExponentialBackoff,
    FixedInterval,
    Custom,
}

/// Strategy interface used by the scheduler.
pub trait ReconnectStrategy: Send + Sync {
    fn kind(&self) -> ReconnectStrategyKind;

    fn next_delay(&self, attempt: u32) -> Option<Duration>;
}

/// Retry immediately.
#[derive(Debug, Clone, Default)]
pub struct ImmediateStrategy;

impl ReconnectStrategy for ImmediateStrategy {
    fn kind(&self) -> ReconnectStrategyKind {
        ReconnectStrategyKind::Immediate
    }

    fn next_delay(&self, attempt: u32) -> Option<Duration> {
        (attempt > 0).then_some(Duration::ZERO)
    }
}

/// Grow delay linearly by attempt.
#[derive(Debug, Clone)]
pub struct LinearStrategy {
    delay: Duration,
}

impl LinearStrategy {
    pub fn new(delay: Duration) -> Self {
        Self { delay }
    }
}

impl ReconnectStrategy for LinearStrategy {
    fn kind(&self) -> ReconnectStrategyKind {
        ReconnectStrategyKind::Linear
    }

    fn next_delay(&self, attempt: u32) -> Option<Duration> {
        (attempt > 0).then(|| self.delay.saturating_mul(attempt))
    }
}

/// Exponential backoff strategy with cap.
#[derive(Debug, Clone)]
pub struct ExponentialBackoffStrategy {
    base_delay: Duration,
    max_delay: Duration,
    factor: f64,
}

impl ExponentialBackoffStrategy {
    pub fn new(base_delay: Duration, max_delay: Duration, factor: f64) -> Self {
        Self {
            base_delay,
            max_delay,
            factor,
        }
    }
}

impl ReconnectStrategy for ExponentialBackoffStrategy {
    fn kind(&self) -> ReconnectStrategyKind {
        ReconnectStrategyKind::ExponentialBackoff
    }

    fn next_delay(&self, attempt: u32) -> Option<Duration> {
        if attempt == 0 {
            return None;
        }

        let exponent = attempt.saturating_sub(1) as i32;
        let delay_ms = self.base_delay.as_millis() as f64 * self.factor.powi(exponent);
        Some(Duration::from_millis(
            (delay_ms as u64).min(self.max_delay.as_millis() as u64),
        ))
    }
}

/// Fixed interval retry strategy.
#[derive(Debug, Clone)]
pub struct FixedIntervalStrategy {
    interval: Duration,
}

impl FixedIntervalStrategy {
    pub fn new(interval: Duration) -> Self {
        Self { interval }
    }
}

impl ReconnectStrategy for FixedIntervalStrategy {
    fn kind(&self) -> ReconnectStrategyKind {
        ReconnectStrategyKind::FixedInterval
    }

    fn next_delay(&self, attempt: u32) -> Option<Duration> {
        (attempt > 0).then_some(self.interval)
    }
}

/// Custom retry sequence.
#[derive(Debug, Clone)]
pub struct CustomReconnectStrategy {
    delays: Vec<Duration>,
}

impl CustomReconnectStrategy {
    pub fn new(delays: Vec<Duration>) -> Self {
        Self { delays }
    }
}

impl ReconnectStrategy for CustomReconnectStrategy {
    fn kind(&self) -> ReconnectStrategyKind {
        ReconnectStrategyKind::Custom
    }

    fn next_delay(&self, attempt: u32) -> Option<Duration> {
        self.delays.get(attempt.saturating_sub(1) as usize).copied()
    }
}

/// Unit of reconnect work produced by the scheduler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectRequest {
    pub tunnel_id: TunnelId,
    pub connection_id: Option<ConnectionId>,
    pub mode: ReconnectMode,
    pub attempt: u32,
    pub reason: Option<String>,
    pub created_at_millis: u64,
    pub scheduled_delay_ms: u64,
    pub next_retry_at_millis: u64,
}

/// Reconnect state snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReconnectSnapshot {
    pub tunnel_id: TunnelId,
    pub connection_id: Option<ConnectionId>,
    pub state: ReconnectState,
    pub mode: ReconnectMode,
    pub strategy: ReconnectStrategyKind,
    pub attempt: u32,
    pub max_attempts: u32,
    pub reconnect_count: u64,
    pub failed_count: u64,
    pub last_error: Option<String>,
    pub next_retry_at_millis: Option<u64>,
    pub updated_at_millis: u64,
}

impl ReconnectSnapshot {
    fn new(
        tunnel_id: TunnelId,
        connection_id: Option<ConnectionId>,
        mode: ReconnectMode,
        strategy: ReconnectStrategyKind,
        max_attempts: u32,
    ) -> Self {
        Self {
            tunnel_id,
            connection_id,
            state: ReconnectState::Queued,
            mode,
            strategy,
            attempt: 0,
            max_attempts,
            reconnect_count: 0,
            failed_count: 0,
            last_error: None,
            next_retry_at_millis: None,
            updated_at_millis: now_millis(),
        }
    }
}

/// Async reconnect contract.
pub trait Reconnect: Send + Sync {
    fn auto_reconnect(
        &self,
        tunnel_id: TunnelId,
        connection_id: Option<ConnectionId>,
        reason: String,
    ) -> BoxFuture<'_, Result<ReconnectSnapshot, ReconnectError>>;

    fn manual_reconnect(
        &self,
        tunnel_id: TunnelId,
        connection_id: Option<ConnectionId>,
        reason: String,
    ) -> BoxFuture<'_, Result<ReconnectSnapshot, ReconnectError>>;

    fn schedule_next(&self) -> BoxFuture<'_, Result<Option<ReconnectRequest>, ReconnectError>>;
}

/// Reconnect manager with bounded queue and pluggable strategy.
pub struct ReconnectManager {
    config: ReconnectConfig,
    queue: RwLock<VecDeque<ReconnectRequest>>,
    snapshots: DashMap<TunnelId, ReconnectSnapshot>,
    strategy: Arc<dyn ReconnectStrategy>,
    events: Option<EventPublisher>,
}

impl Default for ReconnectManager {
    fn default() -> Self {
        Self::new(ReconnectConfig::default())
    }
}

impl ReconnectManager {
    pub fn new(config: ReconnectConfig) -> Self {
        let strategy = strategy_from_config(&config.strategy);
        Self {
            config,
            queue: RwLock::new(VecDeque::new()),
            snapshots: DashMap::new(),
            strategy,
            events: None,
        }
    }

    pub fn with_events(config: ReconnectConfig, events: EventPublisher) -> Self {
        let strategy = strategy_from_config(&config.strategy);
        Self {
            config,
            queue: RwLock::new(VecDeque::new()),
            snapshots: DashMap::new(),
            strategy,
            events: Some(events),
        }
    }

    pub fn with_strategy(config: ReconnectConfig, strategy: Arc<dyn ReconnectStrategy>) -> Self {
        Self {
            config,
            queue: RwLock::new(VecDeque::new()),
            snapshots: DashMap::new(),
            strategy,
            events: None,
        }
    }

    pub async fn auto_reconnect(
        &self,
        tunnel_id: TunnelId,
        connection_id: Option<ConnectionId>,
        reason: impl Into<String>,
    ) -> Result<ReconnectSnapshot, ReconnectError> {
        if !self.config.auto_reconnect {
            return Err(ReconnectError::SchedulerStopped);
        }

        self.enqueue(
            tunnel_id,
            connection_id,
            ReconnectMode::Auto,
            Some(reason.into()),
        )
    }

    pub async fn manual_reconnect(
        &self,
        tunnel_id: TunnelId,
        connection_id: Option<ConnectionId>,
        reason: impl Into<String>,
    ) -> Result<ReconnectSnapshot, ReconnectError> {
        self.enqueue(
            tunnel_id,
            connection_id,
            ReconnectMode::Manual,
            Some(reason.into()),
        )
    }

    pub async fn schedule_next(&self) -> Result<Option<ReconnectRequest>, ReconnectError> {
        let mut request = match self.queue.write().pop_front() {
            Some(request) => request,
            None => return Ok(None),
        };

        let attempt = request.attempt.saturating_add(1);
        if attempt > self.config.max_attempts {
            self.mark_failed(request.tunnel_id, "maximum reconnect attempts exceeded")
                .await?;
            return Err(ReconnectError::AttemptsExceeded {
                tunnel_id: request.tunnel_id,
            });
        }

        let delay = self
            .strategy
            .next_delay(attempt)
            .ok_or(ReconnectError::StrategyRejected { attempt })?;
        let delay_ms = delay.as_millis() as u64;
        let next_retry_at = now_millis().saturating_add(delay_ms);

        request.attempt = attempt;
        request.scheduled_delay_ms = delay_ms;
        request.next_retry_at_millis = next_retry_at;

        if let Some(mut snapshot) = self.snapshots.get_mut(&request.tunnel_id) {
            snapshot.state = ReconnectState::Reconnecting;
            snapshot.attempt = attempt;
            snapshot.reconnect_count = snapshot.reconnect_count.saturating_add(1);
            snapshot.next_retry_at_millis = Some(next_retry_at);
            snapshot.updated_at_millis = now_millis();
        }

        self.publish(TunnelEvent::ReconnectStarted {
            tunnel_id: request.tunnel_id,
            attempt,
        })
        .await;

        Ok(Some(request))
    }

    pub async fn mark_succeeded(
        &self,
        tunnel_id: TunnelId,
    ) -> Result<ReconnectSnapshot, ReconnectError> {
        let snapshot = {
            let mut snapshot = self
                .snapshots
                .get_mut(&tunnel_id)
                .ok_or(ReconnectError::NotFound { tunnel_id })?;
            snapshot.state = ReconnectState::Succeeded;
            snapshot.last_error = None;
            snapshot.next_retry_at_millis = None;
            snapshot.updated_at_millis = now_millis();
            snapshot.clone()
        };

        self.publish(TunnelEvent::ReconnectSucceeded {
            tunnel_id,
            attempt: snapshot.attempt,
        })
        .await;
        Ok(snapshot)
    }

    pub async fn mark_failed(
        &self,
        tunnel_id: TunnelId,
        reason: impl Into<String>,
    ) -> Result<ReconnectSnapshot, ReconnectError> {
        let reason = reason.into();
        let snapshot = {
            let mut snapshot = self
                .snapshots
                .get_mut(&tunnel_id)
                .ok_or(ReconnectError::NotFound { tunnel_id })?;
            snapshot.state = ReconnectState::Failed;
            snapshot.failed_count = snapshot.failed_count.saturating_add(1);
            snapshot.last_error = Some(reason.clone());
            snapshot.next_retry_at_millis = None;
            snapshot.updated_at_millis = now_millis();
            snapshot.clone()
        };

        self.publish(TunnelEvent::ReconnectFailed {
            tunnel_id,
            attempt: snapshot.attempt,
            reason,
        })
        .await;
        Ok(snapshot)
    }

    pub async fn cancel(&self, tunnel_id: TunnelId) -> Result<ReconnectSnapshot, ReconnectError> {
        self.queue
            .write()
            .retain(|item| item.tunnel_id != tunnel_id);
        let mut snapshot = self
            .snapshots
            .get_mut(&tunnel_id)
            .ok_or(ReconnectError::NotFound { tunnel_id })?;
        snapshot.state = ReconnectState::Cancelled;
        snapshot.updated_at_millis = now_millis();
        Ok(snapshot.clone())
    }

    pub fn snapshot(&self, tunnel_id: TunnelId) -> Result<ReconnectSnapshot, ReconnectError> {
        self.snapshots
            .get(&tunnel_id)
            .map(|entry| entry.value().clone())
            .ok_or(ReconnectError::NotFound { tunnel_id })
    }

    pub fn queue_len(&self) -> usize {
        self.queue.read().len()
    }

    pub fn config(&self) -> &ReconnectConfig {
        &self.config
    }

    pub fn strategy_kind(&self) -> ReconnectStrategyKind {
        self.strategy.kind()
    }

    fn enqueue(
        &self,
        tunnel_id: TunnelId,
        connection_id: Option<ConnectionId>,
        mode: ReconnectMode,
        reason: Option<String>,
    ) -> Result<ReconnectSnapshot, ReconnectError> {
        let existing = self
            .snapshots
            .get(&tunnel_id)
            .map(|entry| entry.value().clone());
        let current_attempt = existing
            .as_ref()
            .map(|snapshot| snapshot.attempt)
            .unwrap_or_default();
        let mut queue = self.queue.write();
        if queue.len() >= self.config.queue_capacity {
            return Err(ReconnectError::QueueFull);
        }

        let now = now_millis();
        queue.push_back(ReconnectRequest {
            tunnel_id,
            connection_id,
            mode,
            attempt: current_attempt,
            reason: reason.clone(),
            created_at_millis: now,
            scheduled_delay_ms: 0,
            next_retry_at_millis: now,
        });
        drop(queue);

        let snapshot = if let Some(mut snapshot) = existing {
            snapshot.connection_id = connection_id;
            snapshot.state = ReconnectState::Queued;
            snapshot.mode = mode;
            snapshot.strategy = self.strategy.kind();
            snapshot.max_attempts = self.config.max_attempts;
            snapshot.last_error = reason;
            snapshot.next_retry_at_millis = Some(now);
            snapshot.updated_at_millis = now;
            snapshot
        } else {
            let mut snapshot = ReconnectSnapshot::new(
                tunnel_id,
                connection_id,
                mode,
                self.strategy.kind(),
                self.config.max_attempts,
            );
            snapshot.last_error = reason;
            snapshot.next_retry_at_millis = Some(now);
            snapshot
        };
        self.snapshots.insert(tunnel_id, snapshot.clone());
        Ok(snapshot)
    }

    async fn publish(&self, event: TunnelEvent) {
        if let Some(publisher) = &self.events {
            let _ = publisher.send(event).await;
        }
    }
}

impl Reconnect for ReconnectManager {
    fn auto_reconnect(
        &self,
        tunnel_id: TunnelId,
        connection_id: Option<ConnectionId>,
        reason: String,
    ) -> BoxFuture<'_, Result<ReconnectSnapshot, ReconnectError>> {
        Box::pin(async move {
            ReconnectManager::auto_reconnect(self, tunnel_id, connection_id, reason).await
        })
    }

    fn manual_reconnect(
        &self,
        tunnel_id: TunnelId,
        connection_id: Option<ConnectionId>,
        reason: String,
    ) -> BoxFuture<'_, Result<ReconnectSnapshot, ReconnectError>> {
        Box::pin(async move {
            ReconnectManager::manual_reconnect(self, tunnel_id, connection_id, reason).await
        })
    }

    fn schedule_next(&self) -> BoxFuture<'_, Result<Option<ReconnectRequest>, ReconnectError>> {
        Box::pin(async move { ReconnectManager::schedule_next(self).await })
    }
}

fn strategy_from_config(config: &ReconnectStrategyConfig) -> Arc<dyn ReconnectStrategy> {
    match config {
        ReconnectStrategyConfig::Immediate => Arc::new(ImmediateStrategy),
        ReconnectStrategyConfig::Linear { delay } => Arc::new(LinearStrategy::new(*delay)),
        ReconnectStrategyConfig::ExponentialBackoff {
            base_delay,
            max_delay,
            factor,
        } => Arc::new(ExponentialBackoffStrategy::new(
            *base_delay,
            *max_delay,
            *factor,
        )),
        ReconnectStrategyConfig::FixedInterval { interval } => {
            Arc::new(FixedIntervalStrategy::new(*interval))
        }
        ReconnectStrategyConfig::Custom { delays } => {
            Arc::new(CustomReconnectStrategy::new(delays.clone()))
        }
    }
}

fn now_millis() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}
