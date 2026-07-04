use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct TunnelManager {
    tunnels: Arc<DashMap<Uuid, TunnelHandle>>,
}

struct TunnelHandle {
    tunnel_id: Uuid,
    client_id: Uuid,
    local_port: u16,
    remote_port: u16,
    sender: mpsc::UnboundedSender<TunnelMessage>,
}

pub enum TunnelMessage {
    Relay(Vec<u8>),
    Close,
}

impl TunnelManager {
    pub fn new() -> Self {
        Self {
            tunnels: Arc::new(DashMap::new()),
        }
    }

    pub fn create(&self, tunnel_id: Uuid, client_id: Uuid, local_port: u16, remote_port: u16) {
        let (_tx, _rx) = mpsc::unbounded_channel();
        self.tunnels.insert(
            tunnel_id,
            TunnelHandle {
                tunnel_id,
                client_id,
                local_port,
                remote_port,
                sender: _tx,
            },
        );
    }

    pub fn close(&self, tunnel_id: &Uuid) {
        if let Some((_, handle)) = self.tunnels.remove(tunnel_id) {
            let _ = handle.sender.send(TunnelMessage::Close);
        }
    }

    pub fn relay(&self, tunnel_id: &Uuid, data: Vec<u8>) -> bool {
        if let Some(handle) = self.tunnels.get(tunnel_id) {
            handle.sender.send(TunnelMessage::Relay(data)).is_ok()
        } else {
            false
        }
    }

    pub fn active_count(&self) -> usize {
        self.tunnels.len()
    }
}
