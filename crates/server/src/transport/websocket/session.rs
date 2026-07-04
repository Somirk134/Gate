use tokio::sync::mpsc;
use uuid::Uuid;

pub struct WsSession {
    pub client_id: Uuid,
    pub sender: mpsc::UnboundedSender<String>,
}
