use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tunnel {
    pub id: Uuid,
    pub client_id: Uuid,
    pub local_port: u16,
    pub remote_port: u16,
    pub protocol: TunnelProtocol,
    pub status: TunnelStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TunnelProtocol {
    Tcp,
    Udp,
    Http,
    Https,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TunnelStatus {
    Active,
    Inactive,
    Closed,
}
