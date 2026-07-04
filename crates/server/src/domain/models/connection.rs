use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
    pub id: Uuid,
    pub client_id: Uuid,
    pub tunnel_id: Option<Uuid>,
    pub remote_addr: String,
    pub connected_at: DateTime<Utc>,
    pub disconnected_at: Option<DateTime<Utc>>,
    pub bytes_received: u64,
    pub bytes_sent: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    pub total_connections: u64,
    pub active_connections: u64,
    pub total_bytes_received: u64,
    pub total_bytes_sent: u64,
}
