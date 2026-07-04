use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DomainEvent {
    ClientConnected { client_id: Uuid },
    ClientDisconnected { client_id: Uuid },
    TunnelCreated { tunnel_id: Uuid, client_id: Uuid },
    TunnelClosed { tunnel_id: Uuid, client_id: Uuid },
    DataRelayed { tunnel_id: Uuid, bytes: u64 },
    Error { message: String },
}
