//! Versioned state synchronization snapshots.
//!
//! State synchronization prepares and records state snapshots. It does not
//! transmit snapshots to a server and does not persist them to a database.

use crate::config::SyncConfig;
use crate::core::TunnelId;
use crate::error::StateSyncError;
use crate::event::{EventPublisher, TunnelEvent};
use dashmap::DashMap;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Synchronizable state target.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SyncTarget {
    TunnelState,
    ProjectState,
    ServerState,
    Configuration,
    Statistics,
    LogCursor,
}

impl SyncTarget {
    pub fn as_static_str(self) -> &'static str {
        match self {
            Self::TunnelState => "TunnelState",
            Self::ProjectState => "ProjectState",
            Self::ServerState => "ServerState",
            Self::Configuration => "Configuration",
            Self::Statistics => "Statistics",
            Self::LogCursor => "LogCursor",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct StateKey {
    tunnel_id: Option<TunnelId>,
    target: SyncTarget,
}

/// State snapshot prepared for future transport synchronization.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSnapshot {
    pub tunnel_id: Option<TunnelId>,
    pub target: SyncTarget,
    pub version: u64,
    pub payload: Value,
    pub synchronized_at_millis: u64,
}

/// Synchronization result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateSyncResult {
    pub tunnel_id: Option<TunnelId>,
    pub target: SyncTarget,
    pub version: u64,
    pub synchronized: bool,
    pub synchronized_at_millis: u64,
}

/// Async synchronization contract.
pub trait StateSynchronizer: Send + Sync {
    fn synchronize(
        &self,
        tunnel_id: Option<TunnelId>,
        target: SyncTarget,
        payload: Value,
    ) -> BoxFuture<'_, Result<StateSyncResult, StateSyncError>>;

    fn snapshot(
        &self,
        tunnel_id: Option<TunnelId>,
        target: SyncTarget,
    ) -> BoxFuture<'_, Result<StateSnapshot, StateSyncError>>;
}

/// In-memory state sync manager.
#[derive(Debug)]
pub struct StateSyncManager {
    config: SyncConfig,
    snapshots: DashMap<StateKey, StateSnapshot>,
    events: Option<EventPublisher>,
}

impl Default for StateSyncManager {
    fn default() -> Self {
        Self::new(SyncConfig::default())
    }
}

impl StateSyncManager {
    pub fn new(config: SyncConfig) -> Self {
        Self {
            config,
            snapshots: DashMap::new(),
            events: None,
        }
    }

    pub fn with_events(config: SyncConfig, events: EventPublisher) -> Self {
        Self {
            config,
            snapshots: DashMap::new(),
            events: Some(events),
        }
    }

    pub async fn synchronize(
        &self,
        tunnel_id: Option<TunnelId>,
        target: SyncTarget,
        payload: Value,
    ) -> Result<StateSyncResult, StateSyncError> {
        if !self.target_enabled(target) {
            return Err(StateSyncError::Failed {
                reason: format!("sync target disabled: {}", target.as_static_str()),
            });
        }

        let key = StateKey { tunnel_id, target };
        let version = self
            .snapshots
            .get(&key)
            .map(|snapshot| snapshot.version.saturating_add(1))
            .unwrap_or(1);
        let synchronized_at_millis = now_millis();

        self.snapshots.insert(
            key,
            StateSnapshot {
                tunnel_id,
                target,
                version,
                payload,
                synchronized_at_millis,
            },
        );

        self.publish(TunnelEvent::StateSynchronized {
            tunnel_id,
            target: target.as_static_str().to_string(),
            version,
        })
        .await;

        Ok(StateSyncResult {
            tunnel_id,
            target,
            version,
            synchronized: true,
            synchronized_at_millis,
        })
    }

    pub async fn snapshot(
        &self,
        tunnel_id: Option<TunnelId>,
        target: SyncTarget,
    ) -> Result<StateSnapshot, StateSyncError> {
        let key = StateKey { tunnel_id, target };
        self.snapshots
            .get(&key)
            .map(|snapshot| snapshot.value().clone())
            .ok_or(StateSyncError::NotFound {
                target: target.as_static_str(),
            })
    }

    pub async fn synchronize_many(
        &self,
        snapshots: Vec<(Option<TunnelId>, SyncTarget, Value)>,
    ) -> Result<Vec<StateSyncResult>, StateSyncError> {
        let mut results = Vec::with_capacity(snapshots.len().min(self.config.batch_size));

        for (index, (tunnel_id, target, payload)) in snapshots.into_iter().enumerate() {
            if index >= self.config.batch_size {
                break;
            }

            results.push(self.synchronize(tunnel_id, target, payload).await?);
        }

        Ok(results)
    }

    pub fn config(&self) -> &SyncConfig {
        &self.config
    }

    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    fn target_enabled(&self, target: SyncTarget) -> bool {
        match target {
            SyncTarget::TunnelState => self.config.include_tunnel_state,
            SyncTarget::ProjectState => self.config.include_project_state,
            SyncTarget::ServerState => self.config.include_server_state,
            SyncTarget::Configuration => self.config.include_configuration,
            SyncTarget::Statistics => self.config.include_statistics,
            SyncTarget::LogCursor => self.config.include_log_cursor,
        }
    }

    async fn publish(&self, event: TunnelEvent) {
        if let Some(publisher) = &self.events {
            let _ = publisher.send(event).await;
        }
    }
}

impl StateSynchronizer for StateSyncManager {
    fn synchronize(
        &self,
        tunnel_id: Option<TunnelId>,
        target: SyncTarget,
        payload: Value,
    ) -> BoxFuture<'_, Result<StateSyncResult, StateSyncError>> {
        Box::pin(
            async move { StateSyncManager::synchronize(self, tunnel_id, target, payload).await },
        )
    }

    fn snapshot(
        &self,
        tunnel_id: Option<TunnelId>,
        target: SyncTarget,
    ) -> BoxFuture<'_, Result<StateSnapshot, StateSyncError>> {
        Box::pin(async move { StateSyncManager::snapshot(self, tunnel_id, target).await })
    }
}

fn now_millis() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}
