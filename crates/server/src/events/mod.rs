use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum ServerEvent {
    ClientConnected(Uuid),
    ClientDisconnected(Uuid),
    TunnelCreated(Uuid),
    TunnelClosed(Uuid),
    ConnectionEstablished(Uuid),
    ConnectionClosed(Uuid),
}

#[derive(Clone)]
pub struct EventBus {
    tx: broadcast::Sender<ServerEvent>,
}

impl EventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(1024);
        Self { tx }
    }

    pub fn publish(&self, event: ServerEvent) {
        let _ = self.tx.send(event);
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ServerEvent> {
        self.tx.subscribe()
    }
}
