//! Session identity, lifecycle, and in-memory session manager.

use crate::connection::ConnectionId;
use crate::core::TunnelId;
use crate::runtime::monitor::TrafficStatistics;
use crate::runtime::state::SessionState;
use dashmap::DashMap;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;

pub use crate::session::SessionId;

/// Serializable session view for UI, logs, and diagnostics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSnapshot {
    pub id: SessionId,
    pub tunnel_id: TunnelId,
    pub connection_id: ConnectionId,
    pub created_at_millis: u64,
    pub closed_at_millis: Option<u64>,
    pub remote_addr: SocketAddr,
    pub local_addr: SocketAddr,
    pub upload_bytes: u64,
    pub download_bytes: u64,
    pub status: SessionState,
}

/// Runtime session with per-session traffic counters.
#[derive(Debug)]
pub struct Session {
    pub id: SessionId,
    pub tunnel_id: TunnelId,
    pub connection_id: ConnectionId,
    pub created_at_millis: u64,
    pub remote_addr: SocketAddr,
    pub local_addr: SocketAddr,
    traffic: Arc<TrafficStatistics>,
    status: RwLock<SessionState>,
    closed_at_millis: RwLock<Option<u64>>,
}

impl Session {
    pub fn new(
        tunnel_id: TunnelId,
        connection_id: ConnectionId,
        remote_addr: SocketAddr,
        local_addr: SocketAddr,
    ) -> Self {
        Self {
            id: SessionId::new(),
            tunnel_id,
            connection_id,
            created_at_millis: now_millis(),
            remote_addr,
            local_addr,
            traffic: Arc::new(TrafficStatistics::new()),
            status: RwLock::new(SessionState::Created),
            closed_at_millis: RwLock::new(None),
        }
    }

    pub fn status(&self) -> SessionState {
        *self.status.read()
    }

    pub fn set_status(&self, status: SessionState) {
        *self.status.write() = status;
    }

    pub fn mark_closed(&self, status: SessionState) {
        self.set_status(status);
        *self.closed_at_millis.write() = Some(now_millis());
    }

    pub fn traffic(&self) -> Arc<TrafficStatistics> {
        Arc::clone(&self.traffic)
    }

    pub fn snapshot(&self) -> SessionSnapshot {
        let traffic = self.traffic.snapshot();
        SessionSnapshot {
            id: self.id,
            tunnel_id: self.tunnel_id,
            connection_id: self.connection_id,
            created_at_millis: self.created_at_millis,
            closed_at_millis: *self.closed_at_millis.read(),
            remote_addr: self.remote_addr,
            local_addr: self.local_addr,
            upload_bytes: traffic.total_upload,
            download_bytes: traffic.total_download,
            status: self.status(),
        }
    }
}

/// Unified runtime session manager.
#[derive(Debug)]
pub struct SessionManager {
    sessions: DashMap<SessionId, Arc<Session>>,
    traffic: Arc<TrafficStatistics>,
}

impl SessionManager {
    pub fn new(traffic: Arc<TrafficStatistics>) -> Self {
        Self {
            sessions: DashMap::new(),
            traffic,
        }
    }

    pub fn create(
        &self,
        tunnel_id: TunnelId,
        connection_id: ConnectionId,
        remote_addr: SocketAddr,
        local_addr: SocketAddr,
    ) -> Arc<Session> {
        let session = Arc::new(Session::new(
            tunnel_id,
            connection_id,
            remote_addr,
            local_addr,
        ));
        self.sessions.insert(session.id, Arc::clone(&session));
        self.traffic.increment_session();
        session
    }

    pub fn get(&self, id: &SessionId) -> Option<Arc<Session>> {
        self.sessions.get(id).map(|entry| Arc::clone(entry.value()))
    }

    pub fn close(&self, id: &SessionId, status: SessionState) -> Option<Arc<Session>> {
        let session = self.sessions.remove(id).map(|(_, session)| session)?;
        session.mark_closed(status);
        self.traffic.decrement_session();
        Some(session)
    }

    pub fn close_all(&self, status: SessionState) {
        let ids: Vec<SessionId> = self.sessions.iter().map(|entry| *entry.key()).collect();
        for id in ids {
            let _ = self.close(&id, status);
        }
    }

    pub fn active_count(&self) -> usize {
        self.sessions.len()
    }

    pub fn snapshots(&self) -> Vec<SessionSnapshot> {
        self.sessions
            .iter()
            .map(|entry| entry.value().snapshot())
            .collect()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new(Arc::new(TrafficStatistics::new()))
    }
}

fn now_millis() -> u64 {
    chrono::Utc::now().timestamp_millis() as u64
}
