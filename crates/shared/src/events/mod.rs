use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SharedEvent {
    ClientConnected { client_id: Uuid, timestamp: i64 },
    ClientDisconnected { client_id: Uuid, timestamp: i64 },
    TunnelOpened { tunnel_id: Uuid, client_id: Uuid, remote_port: u16 },
    TunnelClosed { tunnel_id: Uuid, client_id: Uuid },
    DataTransferred { tunnel_id: Uuid, bytes: u64, direction: DataDirection },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataDirection {
    Inbound,
    Outbound,
}
