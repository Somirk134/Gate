use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientInfo {
    pub id: Uuid,
    pub name: String,
    pub version: String,
    pub connected_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TunnelInfo {
    pub id: Uuid,
    pub client_id: Uuid,
    pub local_port: u16,
    pub remote_port: u16,
    pub protocol: String,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionInfo {
    pub id: Uuid,
    pub client_id: Uuid,
    pub tunnel_id: Option<Uuid>,
    pub remote_addr: String,
    pub connected_at: String,
}
