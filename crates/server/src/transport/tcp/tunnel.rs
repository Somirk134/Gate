use bytes::Bytes;
use tokio::sync::mpsc;

pub struct TcpTunnel {
    pub tunnel_id: String,
    pub rx: mpsc::Receiver<Bytes>,
    pub tx: mpsc::Sender<Bytes>,
}
