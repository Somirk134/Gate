//! Session and tunnel recovery orchestration.
//!
//! Recovery restores runtime metadata only: session identity, tunnel identity,
//! statistics, context attributes, and subscription cursors. Business data and
//! database state are deliberately outside this module.

use crate::connection::ConnectionId;
use crate::core::TunnelId;
use crate::error::RecoveryError;
use crate::event::{EventPublisher, TunnelEvent};
use crate::session::SessionId;
use crate::statistics::StabilityMetrics;
use dashmap::DashMap;
use futures::future::BoxFuture;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::time::Instant;

/// Recoverable subscription cursor metadata.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubscriptionSnapshot {
    pub name: String,
    pub cursor: Option<String>,
}

/// Recoverable runtime context.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryContext {
    pub tunnel_id: TunnelId,
    pub session_id: Option<SessionId>,
    pub connection_id: Option<ConnectionId>,
    pub statistics: Option<StabilityMetrics>,
    pub subscriptions: Vec<SubscriptionSnapshot>,
    pub attributes: BTreeMap<String, String>,
    pub captured_at_millis: u64,
}

impl RecoveryContext {
    pub fn new(tunnel_id: TunnelId) -> Self {
        Self {
            tunnel_id,
            session_id: None,
            connection_id: None,
            statistics: None,
            subscriptions: Vec::new(),
            attributes: BTreeMap::new(),
            captured_at_millis: now_millis(),
        }
    }

    pub fn session_id(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn connection_id(mut self, connection_id: ConnectionId) -> Self {
        self.connection_id = Some(connection_id);
        self
    }

    pub fn statistics(mut self, statistics: StabilityMetrics) -> Self {
        self.statistics = Some(statistics);
        self
    }

    pub fn subscription(mut self, name: impl Into<String>, cursor: Option<String>) -> Self {
        self.subscriptions.push(SubscriptionSnapshot {
            name: name.into(),
            cursor,
        });
        self
    }

    pub fn attribute(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.attributes.insert(key.into(), value.into());
        self
    }
}

/// Recovery outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecoveryResult {
    pub tunnel_id: TunnelId,
    pub recovered_session: bool,
    pub recovered_tunnel: bool,
    pub recovered_statistics: bool,
    pub recovered_context: bool,
    pub recovered_subscription: bool,
    pub recovery_time_ms: u64,
    pub warnings: Vec<String>,
}

impl RecoveryResult {
    fn empty(tunnel_id: TunnelId, started: Instant) -> Self {
        Self {
            tunnel_id,
            recovered_session: false,
            recovered_tunnel: false,
            recovered_statistics: false,
            recovered_context: false,
            recovered_subscription: false,
            recovery_time_ms: started.elapsed().as_millis() as u64,
            warnings: Vec::new(),
        }
    }
}

/// Recovery counters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RecoveryMetrics {
    pub recovery_count: u64,
    pub failed_count: u64,
    pub last_recovery_time_ms: u64,
}

/// Async recovery contract.
pub trait Recovery: Send + Sync {
    fn recover_session(
        &self,
        tunnel_id: TunnelId,
    ) -> BoxFuture<'_, Result<RecoveryResult, RecoveryError>>;

    fn recover_tunnel(
        &self,
        tunnel_id: TunnelId,
    ) -> BoxFuture<'_, Result<RecoveryResult, RecoveryError>>;

    fn recover_all(
        &self,
        tunnel_id: TunnelId,
    ) -> BoxFuture<'_, Result<RecoveryResult, RecoveryError>>;
}

/// In-memory recovery manager.
#[derive(Debug, Default)]
pub struct SessionRecoveryManager {
    contexts: DashMap<TunnelId, RecoveryContext>,
    metrics: RwLock<RecoveryMetrics>,
    events: Option<EventPublisher>,
}

impl SessionRecoveryManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_events(events: EventPublisher) -> Self {
        Self {
            contexts: DashMap::new(),
            metrics: RwLock::new(RecoveryMetrics::default()),
            events: Some(events),
        }
    }

    pub async fn capture(&self, context: RecoveryContext) {
        self.contexts.insert(context.tunnel_id, context);
    }

    pub async fn recover_session(
        &self,
        tunnel_id: TunnelId,
    ) -> Result<RecoveryResult, RecoveryError> {
        let started = Instant::now();
        let context = self.context(tunnel_id)?;
        if context.session_id.is_none() {
            self.metrics.write().failed_count += 1;
            return Err(RecoveryError::SessionNotFound { tunnel_id });
        }

        let mut result = RecoveryResult::empty(tunnel_id, started);
        result.recovered_session = true;
        result.recovered_context = true;
        self.finish(result).await
    }

    pub async fn recover_tunnel(
        &self,
        tunnel_id: TunnelId,
    ) -> Result<RecoveryResult, RecoveryError> {
        let started = Instant::now();
        let context = self.context(tunnel_id)?;
        let mut result = RecoveryResult::empty(tunnel_id, started);
        result.recovered_tunnel = true;
        result.recovered_context = !context.attributes.is_empty();
        self.finish(result).await
    }

    pub async fn recover_statistics(
        &self,
        tunnel_id: TunnelId,
    ) -> Result<RecoveryResult, RecoveryError> {
        let started = Instant::now();
        let context = self.context(tunnel_id)?;
        let mut result = RecoveryResult::empty(tunnel_id, started);
        result.recovered_statistics = context.statistics.is_some();
        result.recovered_context = true;
        self.finish(result).await
    }

    pub async fn recover_context(
        &self,
        tunnel_id: TunnelId,
    ) -> Result<RecoveryResult, RecoveryError> {
        let started = Instant::now();
        let _context = self.context(tunnel_id)?;
        let mut result = RecoveryResult::empty(tunnel_id, started);
        result.recovered_context = true;
        self.finish(result).await
    }

    pub async fn recover_subscription(
        &self,
        tunnel_id: TunnelId,
    ) -> Result<RecoveryResult, RecoveryError> {
        let started = Instant::now();
        let context = self.context(tunnel_id)?;
        let mut result = RecoveryResult::empty(tunnel_id, started);
        result.recovered_subscription = !context.subscriptions.is_empty();
        result.recovered_context = true;
        self.finish(result).await
    }

    pub async fn recover_all(&self, tunnel_id: TunnelId) -> Result<RecoveryResult, RecoveryError> {
        let started = Instant::now();
        let context = self.context(tunnel_id)?;
        let mut result = RecoveryResult::empty(tunnel_id, started);
        result.recovered_session = context.session_id.is_some();
        result.recovered_tunnel = true;
        result.recovered_statistics = context.statistics.is_some();
        result.recovered_context = true;
        result.recovered_subscription = !context.subscriptions.is_empty();

        if context.session_id.is_none() {
            result
                .warnings
                .push("session id missing; skipped session restore".to_string());
        }

        self.finish(result).await
    }

    pub fn context(&self, tunnel_id: TunnelId) -> Result<RecoveryContext, RecoveryError> {
        self.contexts
            .get(&tunnel_id)
            .map(|entry| entry.value().clone())
            .ok_or(RecoveryError::ContextNotFound { tunnel_id })
    }

    pub fn metrics(&self) -> RecoveryMetrics {
        self.metrics.read().clone()
    }

    async fn finish(&self, mut result: RecoveryResult) -> Result<RecoveryResult, RecoveryError> {
        result.recovery_time_ms = result.recovery_time_ms.max(1);
        {
            let mut metrics = self.metrics.write();
            metrics.recovery_count = metrics.recovery_count.saturating_add(1);
            metrics.last_recovery_time_ms = result.recovery_time_ms;
        }

        self.publish(TunnelEvent::SessionRecovered {
            tunnel_id: result.tunnel_id,
            recovery_time_ms: result.recovery_time_ms,
        })
        .await;
        Ok(result)
    }

    async fn publish(&self, event: TunnelEvent) {
        if let Some(publisher) = &self.events {
            let _ = publisher.send(event).await;
        }
    }
}

impl Recovery for SessionRecoveryManager {
    fn recover_session(
        &self,
        tunnel_id: TunnelId,
    ) -> BoxFuture<'_, Result<RecoveryResult, RecoveryError>> {
        Box::pin(async move { SessionRecoveryManager::recover_session(self, tunnel_id).await })
    }

    fn recover_tunnel(
        &self,
        tunnel_id: TunnelId,
    ) -> BoxFuture<'_, Result<RecoveryResult, RecoveryError>> {
        Box::pin(async move { SessionRecoveryManager::recover_tunnel(self, tunnel_id).await })
    }

    fn recover_all(
        &self,
        tunnel_id: TunnelId,
    ) -> BoxFuture<'_, Result<RecoveryResult, RecoveryError>> {
        Box::pin(async move { SessionRecoveryManager::recover_all(self, tunnel_id).await })
    }
}

fn now_millis() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}
