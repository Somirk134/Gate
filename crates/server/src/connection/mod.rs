use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct ConnectionManager {
    connections: Arc<DashMap<Uuid, ConnectionHandle>>,
}

struct ConnectionHandle {
    client_id: Uuid,
    sender: mpsc::UnboundedSender<ConnectionMessage>,
}

pub enum ConnectionMessage {
    Data(Vec<u8>),
    Close,
}

impl ConnectionManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(DashMap::new()),
        }
    }

    pub fn register(&self, client_id: Uuid, sender: mpsc::UnboundedSender<ConnectionMessage>) {
        self.connections.insert(
            client_id,
            ConnectionHandle { client_id, sender },
        );
    }

    pub fn unregister(&self, client_id: &Uuid) {
        self.connections.remove(client_id);
    }

    pub fn send(&self, client_id: &Uuid, msg: ConnectionMessage) -> bool {
        if let Some(handle) = self.connections.get(client_id) {
            handle.sender.send(msg).is_ok()
        } else {
            false
        }
    }

    pub fn active_count(&self) -> usize {
        self.connections.len()
    }
}
