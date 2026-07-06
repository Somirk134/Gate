//! Mock runtime objects for client-side debugging.

use crate::connection::ConnectionId;
use crate::core::TunnelId;
use crate::runtime::monitor::TrafficSnapshot;
use crate::runtime::session::SessionId;
use crate::runtime::state::{ForwardState, RuntimeState, SessionState};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/// Mock runtime for UI and client integration debugging.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockRuntime {
    pub tunnel_id: TunnelId,
    pub state: RuntimeState,
    pub traffic: MockTraffic,
}

impl Default for MockRuntime {
    fn default() -> Self {
        Self {
            tunnel_id: TunnelId::new(),
            state: RuntimeState::Created,
            traffic: MockTraffic::default(),
        }
    }
}

impl MockRuntime {
    pub fn start(&mut self) {
        self.state = RuntimeState::Running;
    }

    pub fn stop(&mut self) {
        self.state = RuntimeState::Stopped;
    }

    pub fn pause(&mut self) {
        self.state = RuntimeState::Paused;
    }
}

/// Mock session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockSession {
    pub id: SessionId,
    pub tunnel_id: TunnelId,
    pub connection_id: ConnectionId,
    pub remote_addr: SocketAddr,
    pub local_addr: SocketAddr,
    pub status: SessionState,
}

impl Default for MockSession {
    fn default() -> Self {
        Self {
            id: SessionId::new(),
            tunnel_id: TunnelId::new(),
            connection_id: ConnectionId::new(),
            remote_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 10000),
            local_addr: SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 20000),
            status: SessionState::Created,
        }
    }
}

/// Mock forward pipeline.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MockForward {
    pub state: ForwardState,
    pub upload_bytes: u64,
    pub download_bytes: u64,
}

impl Default for MockForward {
    fn default() -> Self {
        Self {
            state: ForwardState::Created,
            upload_bytes: 0,
            download_bytes: 0,
        }
    }
}

/// Mock traffic.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MockTraffic {
    pub snapshot: TrafficSnapshot,
}
