//! Connection monitoring and health scoring.
//!
//! This module stores per-connection observations only. It does not open,
//! close, read, or write network sockets.

use crate::connection::ConnectionId;
use crate::core::TunnelId;
use crate::error::ConnectionLostError;
use crate::event::{EventPublisher, TunnelEvent};
use dashmap::DashMap;
use futures::future::BoxFuture;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Aggregated connection health exposed to UI and reconnect managers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionHealth {
    Healthy,
    Warning,
    Critical,
    Offline,
}

/// Serializable connection monitor snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionMonitorSnapshot {
    pub tunnel_id: TunnelId,
    pub connection_id: ConnectionId,
    pub latency_ms: Option<u64>,
    pub alive: bool,
    pub packet_loss: f64,
    pub last_ping_millis: Option<u64>,
    pub last_pong_millis: Option<u64>,
    pub average_rtt_ms: f64,
    pub health_score: u8,
    pub health: ConnectionHealth,
    pub ping_count: u64,
    pub pong_count: u64,
    pub lost_packets: u64,
    pub connection_started_at_millis: u64,
    pub connection_duration_ms: u64,
}

impl ConnectionMonitorSnapshot {
    fn new(tunnel_id: TunnelId, connection_id: ConnectionId) -> Self {
        Self {
            tunnel_id,
            connection_id,
            latency_ms: None,
            alive: true,
            packet_loss: 0.0,
            last_ping_millis: None,
            last_pong_millis: None,
            average_rtt_ms: 0.0,
            health_score: 100,
            health: ConnectionHealth::Healthy,
            ping_count: 0,
            pong_count: 0,
            lost_packets: 0,
            connection_started_at_millis: now_millis(),
            connection_duration_ms: 0,
        }
    }

    fn recompute(&mut self) {
        let expected_packets = self.ping_count.max(1);
        self.packet_loss = self.lost_packets as f64 / expected_packets as f64;

        let rtt_penalty = self
            .latency_ms
            .map(|latency| (latency / 20).min(40) as u8)
            .unwrap_or(0);
        let loss_penalty = (self.packet_loss * 50.0).round().min(50.0) as u8;
        let alive_penalty = if self.alive { 0 } else { 100 };

        self.health_score = 100u8
            .saturating_sub(rtt_penalty)
            .saturating_sub(loss_penalty)
            .saturating_sub(alive_penalty);
        self.health = match (self.alive, self.health_score) {
            (false, _) => ConnectionHealth::Offline,
            (_, 80..=100) => ConnectionHealth::Healthy,
            (_, 50..=79) => ConnectionHealth::Warning,
            _ => ConnectionHealth::Critical,
        };
        self.connection_duration_ms =
            now_millis().saturating_sub(self.connection_started_at_millis);
    }
}

/// Async connection monitor contract.
pub trait ConnectionMonitor: Send + Sync {
    fn register(
        &self,
        tunnel_id: TunnelId,
        connection_id: ConnectionId,
    ) -> BoxFuture<'_, ConnectionMonitorSnapshot>;

    fn record_ping(
        &self,
        connection_id: ConnectionId,
    ) -> BoxFuture<'_, Result<ConnectionMonitorSnapshot, ConnectionLostError>>;

    fn record_pong(
        &self,
        connection_id: ConnectionId,
        rtt: Duration,
    ) -> BoxFuture<'_, Result<ConnectionMonitorSnapshot, ConnectionLostError>>;

    fn mark_lost(
        &self,
        connection_id: ConnectionId,
    ) -> BoxFuture<'_, Result<ConnectionMonitorSnapshot, ConnectionLostError>>;

    fn mark_restored(
        &self,
        connection_id: ConnectionId,
    ) -> BoxFuture<'_, Result<ConnectionMonitorSnapshot, ConnectionLostError>>;
}

/// Lock-sharded monitor registry.
#[derive(Debug, Default)]
pub struct ConnectionMonitorManager {
    snapshots: DashMap<ConnectionId, ConnectionMonitorSnapshot>,
    events: Option<EventPublisher>,
}

impl ConnectionMonitorManager {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_events(events: EventPublisher) -> Self {
        Self {
            snapshots: DashMap::new(),
            events: Some(events),
        }
    }

    pub async fn register(
        &self,
        tunnel_id: TunnelId,
        connection_id: ConnectionId,
    ) -> ConnectionMonitorSnapshot {
        let snapshot = ConnectionMonitorSnapshot::new(tunnel_id, connection_id);
        self.snapshots.insert(connection_id, snapshot.clone());
        snapshot
    }

    pub async fn record_ping(
        &self,
        connection_id: ConnectionId,
    ) -> Result<ConnectionMonitorSnapshot, ConnectionLostError> {
        let mut snapshot = self
            .snapshots
            .get_mut(&connection_id)
            .ok_or(ConnectionLostError::MonitorNotFound { connection_id })?;
        snapshot.ping_count = snapshot.ping_count.saturating_add(1);
        snapshot.last_ping_millis = Some(now_millis());
        snapshot.recompute();
        Ok(snapshot.clone())
    }

    pub async fn record_pong(
        &self,
        connection_id: ConnectionId,
        rtt: Duration,
    ) -> Result<ConnectionMonitorSnapshot, ConnectionLostError> {
        let mut snapshot = self
            .snapshots
            .get_mut(&connection_id)
            .ok_or(ConnectionLostError::MonitorNotFound { connection_id })?;
        let rtt_ms = rtt.as_millis() as u64;
        snapshot.alive = true;
        snapshot.pong_count = snapshot.pong_count.saturating_add(1);
        snapshot.latency_ms = Some(rtt_ms);
        snapshot.average_rtt_ms = if snapshot.pong_count == 1 {
            rtt_ms as f64
        } else {
            ((snapshot.average_rtt_ms * (snapshot.pong_count - 1) as f64) + rtt_ms as f64)
                / snapshot.pong_count as f64
        };
        snapshot.last_pong_millis = Some(now_millis());
        snapshot.recompute();
        Ok(snapshot.clone())
    }

    pub async fn mark_lost(
        &self,
        connection_id: ConnectionId,
    ) -> Result<ConnectionMonitorSnapshot, ConnectionLostError> {
        let snapshot = {
            let mut snapshot = self
                .snapshots
                .get_mut(&connection_id)
                .ok_or(ConnectionLostError::MonitorNotFound { connection_id })?;
            snapshot.alive = false;
            snapshot.lost_packets = snapshot.lost_packets.saturating_add(1);
            snapshot.recompute();
            snapshot.clone()
        };

        self.publish(TunnelEvent::ConnectionLost {
            tunnel_id: snapshot.tunnel_id,
            connection_id: Some(connection_id),
        })
        .await;
        Ok(snapshot)
    }

    pub async fn mark_restored(
        &self,
        connection_id: ConnectionId,
    ) -> Result<ConnectionMonitorSnapshot, ConnectionLostError> {
        let snapshot = {
            let mut snapshot = self
                .snapshots
                .get_mut(&connection_id)
                .ok_or(ConnectionLostError::MonitorNotFound { connection_id })?;
            snapshot.alive = true;
            snapshot.recompute();
            snapshot.clone()
        };

        self.publish(TunnelEvent::ConnectionRestored {
            tunnel_id: snapshot.tunnel_id,
            connection_id: Some(connection_id),
        })
        .await;
        Ok(snapshot)
    }

    pub fn snapshot(
        &self,
        connection_id: ConnectionId,
    ) -> Result<ConnectionMonitorSnapshot, ConnectionLostError> {
        self.snapshots
            .get(&connection_id)
            .map(|entry| entry.value().clone())
            .ok_or(ConnectionLostError::MonitorNotFound { connection_id })
    }

    pub fn len(&self) -> usize {
        self.snapshots.len()
    }

    pub fn is_empty(&self) -> bool {
        self.snapshots.is_empty()
    }

    async fn publish(&self, event: TunnelEvent) {
        if let Some(publisher) = &self.events {
            let _ = publisher.send(event).await;
        }
    }
}

impl ConnectionMonitor for ConnectionMonitorManager {
    fn register(
        &self,
        tunnel_id: TunnelId,
        connection_id: ConnectionId,
    ) -> BoxFuture<'_, ConnectionMonitorSnapshot> {
        Box::pin(async move { ConnectionMonitorManager::register(self, tunnel_id, connection_id).await })
    }

    fn record_ping(
        &self,
        connection_id: ConnectionId,
    ) -> BoxFuture<'_, Result<ConnectionMonitorSnapshot, ConnectionLostError>> {
        Box::pin(async move { ConnectionMonitorManager::record_ping(self, connection_id).await })
    }

    fn record_pong(
        &self,
        connection_id: ConnectionId,
        rtt: Duration,
    ) -> BoxFuture<'_, Result<ConnectionMonitorSnapshot, ConnectionLostError>> {
        Box::pin(async move { ConnectionMonitorManager::record_pong(self, connection_id, rtt).await })
    }

    fn mark_lost(
        &self,
        connection_id: ConnectionId,
    ) -> BoxFuture<'_, Result<ConnectionMonitorSnapshot, ConnectionLostError>> {
        Box::pin(async move { ConnectionMonitorManager::mark_lost(self, connection_id).await })
    }

    fn mark_restored(
        &self,
        connection_id: ConnectionId,
    ) -> BoxFuture<'_, Result<ConnectionMonitorSnapshot, ConnectionLostError>> {
        Box::pin(async move { ConnectionMonitorManager::mark_restored(self, connection_id).await })
    }
}

fn now_millis() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}
