use thiserror::Error;

pub type CommunicationResult<T> = Result<T, CommunicationError>;

#[derive(Debug, Error)]
pub enum CommunicationError {
    #[error("transport error: {0}")]
    Transport(#[from] TransportError),
    #[error("protocol error: {0}")]
    Protocol(#[from] gate_protocol::ProtocolError),
    #[error("request was canceled")]
    Canceled,
    #[error("internal communication error: {0}")]
    Internal(String),
}

#[derive(Debug, Error)]
pub enum TransportError {
    #[error("transport is unavailable: {0}")]
    Unavailable(String),
    #[error("transport connect failed: {0}")]
    Connect(String),
    #[error("transport disconnect failed: {0}")]
    Disconnect(String),
    #[error("transport read failed: {0}")]
    Read(String),
    #[error("transport write failed: {0}")]
    Write(String),
    #[error("transport reconnect failed: {0}")]
    Reconnect(String),
    #[error("transport is closed")]
    Closed,
}
