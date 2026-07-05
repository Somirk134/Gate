//! Unified Tunnel Engine error taxonomy.

use crate::core::{EnginePhase, TunnelId};
use thiserror::Error;

/// Engine-level errors.
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("invalid engine state transition: {from:?} -> {to:?}")]
    InvalidStateTransition { from: EnginePhase, to: EnginePhase },

    #[error("engine component is unavailable: {component}")]
    ComponentUnavailable { component: &'static str },

    #[error("tunnel error: {0}")]
    Tunnel(#[from] TunnelError),

    #[error("connection error: {0}")]
    Connection(#[from] ConnectionError),

    #[error("forward error: {0}")]
    Forward(#[from] ForwardError),

    #[error("protocol error: {0}")]
    Protocol(#[from] ProtocolError),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

/// Tunnel lifecycle and registry errors.
#[derive(Debug, Error)]
pub enum TunnelError {
    #[error("tunnel was not found: {id}")]
    NotFound { id: TunnelId },

    #[error("tunnel is not ready: {id}")]
    NotReady { id: TunnelId },

    #[error("tunnel operation is not implemented yet: {operation}")]
    NotImplemented { operation: &'static str },
}

/// Connection allocation and state errors.
#[derive(Debug, Error)]
pub enum ConnectionError {
    #[error("connection was not found: {id}")]
    NotFound { id: crate::connection::ConnectionId },

    #[error("connection is closed: {id}")]
    Closed { id: crate::connection::ConnectionId },

    #[error("connection operation is not implemented yet: {operation}")]
    NotImplemented { operation: &'static str },
}

/// Forwarding pipeline errors.
#[derive(Debug, Error)]
pub enum ForwardError {
    #[error("forwarder is paused")]
    Paused,

    #[error("forwarder is closed")]
    Closed,

    #[error("forward operation is not implemented yet: {operation}")]
    NotImplemented { operation: &'static str },
}

/// Protocol boundary errors.
#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("unsupported protocol: {protocol}")]
    Unsupported { protocol: String },

    #[error("protocol configuration is invalid: {reason}")]
    InvalidConfig { reason: String },

    #[error("protocol operation is not implemented yet: {operation}")]
    NotImplemented { operation: &'static str },
}
