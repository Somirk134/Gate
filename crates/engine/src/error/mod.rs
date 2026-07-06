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

    #[error("heartbeat error: {0}")]
    Heartbeat(#[from] HeartbeatError),

    #[error("reconnect error: {0}")]
    Reconnect(#[from] ReconnectError),

    #[error("recovery error: {0}")]
    Recovery(#[from] RecoveryError),

    #[error("connection lost: {0}")]
    ConnectionLost(#[from] ConnectionLostError),

    #[error("state sync error: {0}")]
    StateSync(#[from] StateSyncError),

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

/// Heartbeat state machine errors.
#[derive(Debug, Error)]
pub enum HeartbeatError {
    #[error("heartbeat was not found for tunnel: {tunnel_id}")]
    NotFound { tunnel_id: TunnelId },

    #[error("heartbeat is already running for tunnel: {tunnel_id}")]
    AlreadyRunning { tunnel_id: TunnelId },

    #[error("heartbeat is not running for tunnel: {tunnel_id}")]
    NotRunning { tunnel_id: TunnelId },

    #[error("invalid heartbeat state transition: {from} -> {to}")]
    InvalidTransition { from: &'static str, to: &'static str },

    #[error("heartbeat timed out for tunnel: {tunnel_id}")]
    Timeout { tunnel_id: TunnelId },
}

/// Reconnect queue and scheduler errors.
#[derive(Debug, Error)]
pub enum ReconnectError {
    #[error("reconnect entry was not found for tunnel: {tunnel_id}")]
    NotFound { tunnel_id: TunnelId },

    #[error("reconnect queue is full")]
    QueueFull,

    #[error("reconnect attempts exceeded for tunnel: {tunnel_id}")]
    AttemptsExceeded { tunnel_id: TunnelId },

    #[error("reconnect strategy rejected attempt {attempt}")]
    StrategyRejected { attempt: u32 },

    #[error("reconnect scheduler is stopped")]
    SchedulerStopped,
}

/// Session and tunnel recovery errors.
#[derive(Debug, Error)]
pub enum RecoveryError {
    #[error("recovery context was not found for tunnel: {tunnel_id}")]
    ContextNotFound { tunnel_id: TunnelId },

    #[error("session recovery was not found for tunnel: {tunnel_id}")]
    SessionNotFound { tunnel_id: TunnelId },

    #[error("tunnel recovery was not found for tunnel: {tunnel_id}")]
    TunnelNotFound { tunnel_id: TunnelId },

    #[error("recovery operation is unsupported: {operation}")]
    Unsupported { operation: &'static str },

    #[error("recovery failed: {reason}")]
    Failed { reason: String },
}

/// Connection loss and monitor errors.
#[derive(Debug, Error)]
pub enum ConnectionLostError {
    #[error("connection was lost: {connection_id}")]
    Lost {
        connection_id: crate::connection::ConnectionId,
    },

    #[error("connection is offline: {connection_id}")]
    Offline {
        connection_id: crate::connection::ConnectionId,
    },

    #[error("connection monitor was not found: {connection_id}")]
    MonitorNotFound {
        connection_id: crate::connection::ConnectionId,
    },
}

/// State synchronization errors.
#[derive(Debug, Error)]
pub enum StateSyncError {
    #[error("state snapshot was not found: {target}")]
    NotFound { target: &'static str },

    #[error("state version conflict for target {target}: local={local}, remote={remote}")]
    Conflict {
        target: &'static str,
        local: u64,
        remote: u64,
    },

    #[error("state synchronization failed: {reason}")]
    Failed { reason: String },
}
