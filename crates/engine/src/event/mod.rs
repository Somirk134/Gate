//! Engine event contracts.

use crate::connection::ConnectionId;
use crate::core::TunnelId;
use crate::statistics::TunnelStatistics;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

/// Unified Tunnel Engine event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TunnelEvent {
    TunnelStarted { tunnel_id: TunnelId },
    TunnelStopped { tunnel_id: TunnelId },
    ConnectionCreated {
        tunnel_id: TunnelId,
        connection_id: crate::connection::ConnectionId,
    },
    ConnectionClosed {
        tunnel_id: TunnelId,
        connection_id: crate::connection::ConnectionId,
    },
    TrafficUpdated {
        tunnel_id: TunnelId,
        statistics: TunnelStatistics,
    },
    HeartbeatStarted {
        tunnel_id: TunnelId,
    },
    HeartbeatStopped {
        tunnel_id: TunnelId,
    },
    HeartbeatTimeout { tunnel_id: TunnelId },
    ReconnectStarted {
        tunnel_id: TunnelId,
        attempt: u32,
    },
    ReconnectSucceeded {
        tunnel_id: TunnelId,
        attempt: u32,
    },
    ReconnectFailed {
        tunnel_id: TunnelId,
        attempt: u32,
        reason: String,
    },
    SessionRecovered {
        tunnel_id: TunnelId,
        recovery_time_ms: u64,
    },
    ConnectionLost {
        tunnel_id: TunnelId,
        connection_id: Option<ConnectionId>,
    },
    ConnectionRestored {
        tunnel_id: TunnelId,
        connection_id: Option<ConnectionId>,
    },
    StateSynchronized {
        tunnel_id: Option<TunnelId>,
        target: String,
        version: u64,
    },
    TunnelError { tunnel_id: TunnelId, message: String },
}

pub type EventPublisher = mpsc::Sender<TunnelEvent>;
pub type EventSubscriber = mpsc::Receiver<TunnelEvent>;

pub fn event_channel(buffer: usize) -> (EventPublisher, EventSubscriber) {
    mpsc::channel(buffer)
}
