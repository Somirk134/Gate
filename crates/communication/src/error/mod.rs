//! Error taxonomy for the communication layer.

use thiserror::Error;

use crate::{connection::ConnectionId, timeout::TimeoutKind};

pub type CommunicationResult<T> = Result<T, CommunicationError>;

#[derive(Debug, Error)]
pub enum CommunicationError {
    #[error("transport error: {0}")]
    Transport(#[from] TransportError),
    #[error("timeout error: {0}")]
    Timeout(#[from] TimeoutError),
    #[error("connection error: {0}")]
    Connection(#[from] ConnectionError),
    #[error("dispatcher error: {0}")]
    Dispatcher(#[from] DispatcherError),
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

#[derive(Debug, Error)]
pub enum TimeoutError {
    #[error("{kind:?} timeout expired after {timeout_ms} ms")]
    Expired {
        kind: TimeoutKind,
        timeout_ms: u64,
    },
}

#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("connection not found: {id}")]
    NotFound { id: ConnectionId },
    #[error("invalid connection state transition: {from:?} -> {to:?}")]
    InvalidTransition {
        from: crate::connection::ConnectionState,
        to: crate::connection::ConnectionState,
    },
    #[error("connection failed: {0}")]
    Failed(String),
}

#[derive(Debug, Error)]
pub enum DispatcherError {
    #[error("handler unavailable for command: {command}")]
    HandlerUnavailable { command: String },
    #[error("unsupported message type: {message_type:?}")]
    UnsupportedMessageType {
        message_type: gate_protocol::MessageType,
    },
    #[error("request not found: {request_id}")]
    RequestNotFound { request_id: uuid::Uuid },
    #[error("dispatch failed: {0}")]
    Failed(String),
}
