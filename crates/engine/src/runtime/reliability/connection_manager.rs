//! Runtime connection manager.

use crate::connection::ConnectionId;
use crate::core::TunnelId;
use crate::runtime::state::ConnectionState;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionPolicy {
    pub max_connections: usize,
    pub connection_timeout: Duration,
    pub idle_timeout: Duration,
    pub keep_alive: bool,
}

impl Default for ConnectionPolicy {
    fn default() -> Self {
        Self {
            max_connections: 4096,
            connection_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(300),
            keep_alive: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConnectionSnapshot {
    pub tunnel_id: TunnelId,
    pub connection_id: ConnectionId,
    pub remote_addr: SocketAddr,
    pub local_addr: SocketAddr,
    pub state: ConnectionState,
    pub created_at_millis: u64,
    pub last_activity_millis: u64,
    pub closed_at_millis: Option<u64>,
    pub keep_alive: bool,
}

impl RuntimeConnectionSnapshot {
    fn new(
        tunnel_id: TunnelId,
        connection_id: ConnectionId,
        remote_addr: SocketAddr,
        local_addr: SocketAddr,
        keep_alive: bool,
    ) -> Self {
        let now = now_millis();
        Self {
            tunnel_id,
            connection_id,
            remote_addr,
            local_addr,
            state: ConnectionState::Created,
            created_at_millis: now,
            last_activity_millis: now,
            closed_at_millis: None,
            keep_alive,
        }
    }
}

#[derive(Debug, Error)]
pub enum ConnectionManagerError {
    #[error("connection limit exceeded: max_connections={max_connections}")]
    LimitExceeded { max_connections: usize },

    #[error("connection was not found: {connection_id}")]
    NotFound { connection_id: ConnectionId },
}

#[derive(Debug)]
pub struct RuntimeConnectionManager {
    policy: ConnectionPolicy,
    connections: DashMap<ConnectionId, RuntimeConnectionSnapshot>,
}

impl Default for RuntimeConnectionManager {
    fn default() -> Self {
        Self::new(ConnectionPolicy::default())
    }
}

impl RuntimeConnectionManager {
    pub fn new(policy: ConnectionPolicy) -> Self {
        Self {
            policy,
            connections: DashMap::new(),
        }
    }

    pub fn register(
        &self,
        tunnel_id: TunnelId,
        remote_addr: SocketAddr,
        local_addr: SocketAddr,
    ) -> Result<RuntimeConnectionSnapshot, ConnectionManagerError> {
        if self.active_count() >= self.policy.max_connections {
            return Err(ConnectionManagerError::LimitExceeded {
                max_connections: self.policy.max_connections,
            });
        }

        let snapshot = RuntimeConnectionSnapshot::new(
            tunnel_id,
            ConnectionId::new(),
            remote_addr,
            local_addr,
            self.policy.keep_alive,
        );
        self.connections
            .insert(snapshot.connection_id, snapshot.clone());
        Ok(snapshot)
    }

    pub fn mark_connected(
        &self,
        connection_id: ConnectionId,
    ) -> Result<RuntimeConnectionSnapshot, ConnectionManagerError> {
        self.update(connection_id, |snapshot| {
            snapshot.state = ConnectionState::Connected;
            snapshot.last_activity_millis = now_millis();
        })
    }

    pub fn touch(
        &self,
        connection_id: ConnectionId,
    ) -> Result<RuntimeConnectionSnapshot, ConnectionManagerError> {
        self.update(connection_id, |snapshot| {
            snapshot.last_activity_millis = now_millis();
        })
    }

    pub fn close(
        &self,
        connection_id: ConnectionId,
    ) -> Result<RuntimeConnectionSnapshot, ConnectionManagerError> {
        let (_, mut snapshot) = self
            .connections
            .remove(&connection_id)
            .ok_or(ConnectionManagerError::NotFound { connection_id })?;
        snapshot.state = ConnectionState::Closed;
        snapshot.closed_at_millis = Some(now_millis());
        Ok(snapshot)
    }

    pub fn expire_idle(&self) -> Vec<RuntimeConnectionSnapshot> {
        self.expire_by(|snapshot, now, _connection_timeout_ms, idle_timeout_ms| {
            now.saturating_sub(snapshot.last_activity_millis) >= idle_timeout_ms
                && !matches!(
                    snapshot.state,
                    ConnectionState::Closing | ConnectionState::Closed | ConnectionState::Failed
                )
        })
    }

    pub fn expire_timeouts(&self) -> Vec<RuntimeConnectionSnapshot> {
        self.expire_by(|snapshot, now, connection_timeout_ms, idle_timeout_ms| {
            let connection_age = now.saturating_sub(snapshot.created_at_millis);
            let idle_age = now.saturating_sub(snapshot.last_activity_millis);
            let connection_timed_out = matches!(
                snapshot.state,
                ConnectionState::Created | ConnectionState::Connecting
            ) && connection_age >= connection_timeout_ms;
            let idle_timed_out = idle_age >= idle_timeout_ms
                && !matches!(
                    snapshot.state,
                    ConnectionState::Closing | ConnectionState::Closed | ConnectionState::Failed
                );
            connection_timed_out || idle_timed_out
        })
    }

    fn expire_by(
        &self,
        should_expire: impl Fn(&RuntimeConnectionSnapshot, u64, u64, u64) -> bool,
    ) -> Vec<RuntimeConnectionSnapshot> {
        let now = now_millis();
        let connection_timeout_ms = self
            .policy
            .connection_timeout
            .as_millis()
            .min(u128::from(u64::MAX)) as u64;
        let idle_timeout_ms = self
            .policy
            .idle_timeout
            .as_millis()
            .min(u128::from(u64::MAX)) as u64;
        let expired_ids = self
            .connections
            .iter()
            .filter(|entry| {
                should_expire(entry.value(), now, connection_timeout_ms, idle_timeout_ms)
            })
            .map(|entry| *entry.key())
            .collect::<Vec<_>>();

        expired_ids
            .into_iter()
            .filter_map(|connection_id| self.close(connection_id).ok())
            .collect()
    }

    pub fn snapshot(&self, connection_id: ConnectionId) -> Option<RuntimeConnectionSnapshot> {
        self.connections
            .get(&connection_id)
            .map(|entry| entry.value().clone())
    }

    pub fn snapshots(&self) -> Vec<RuntimeConnectionSnapshot> {
        self.connections
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn active_count(&self) -> usize {
        self.connections
            .iter()
            .filter(|entry| {
                !matches!(
                    entry.state,
                    ConnectionState::Closing | ConnectionState::Closed | ConnectionState::Failed
                )
            })
            .count()
    }

    pub fn policy(&self) -> &ConnectionPolicy {
        &self.policy
    }

    fn update(
        &self,
        connection_id: ConnectionId,
        update: impl FnOnce(&mut RuntimeConnectionSnapshot),
    ) -> Result<RuntimeConnectionSnapshot, ConnectionManagerError> {
        let mut entry = self
            .connections
            .get_mut(&connection_id)
            .ok_or(ConnectionManagerError::NotFound { connection_id })?;
        update(entry.value_mut());
        Ok(entry.value().clone())
    }
}

fn now_millis() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}
