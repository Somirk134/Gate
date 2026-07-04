use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TunnelMessage {
    Connect {
        client_id: Uuid,
        token: String,
    },
    ConnectAck {
        success: bool,
        message: String,
    },
    CreateTunnel {
        local_port: u16,
        remote_port: u16,
        protocol: String,
    },
    TunnelCreated {
        tunnel_id: Uuid,
        remote_port: u16,
    },
    CloseTunnel {
        tunnel_id: Uuid,
    },
    Data {
        tunnel_id: Uuid,
        payload: Vec<u8>,
    },
    Heartbeat,
    Error {
        code: u32,
        message: String,
    },
}
