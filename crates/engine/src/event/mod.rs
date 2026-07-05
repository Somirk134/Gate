//! Engine event contracts.

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
    HeartbeatTimeout { tunnel_id: TunnelId },
    TunnelError { tunnel_id: TunnelId, message: String },
}

pub type EventPublisher = mpsc::Sender<TunnelEvent>;
pub type EventSubscriber = mpsc::Receiver<TunnelEvent>;

pub fn event_channel(buffer: usize) -> (EventPublisher, EventSubscriber) {
    mpsc::channel(buffer)
}
