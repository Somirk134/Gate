//! Runtime error taxonomy.

use crate::runtime::state::{ConnectionState, ForwardState, RuntimeState};
use crate::runtime::worker::TaskId;
use std::net::SocketAddr;
use std::time::Duration;
use thiserror::Error;

/// Top-level runtime error.
#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("invalid runtime state transition: {from:?} -> {to:?}")]
    InvalidStateTransition {
        from: RuntimeState,
        to: RuntimeState,
    },

    #[error("runtime configuration is invalid: {reason}")]
    InvalidConfig { reason: String },

    #[error("runtime is paused")]
    Paused,

    #[error("runtime shutdown was requested")]
    Shutdown,

    #[error("listener error: {0}")]
    Listener(#[from] ListenerError),

    #[error("connector error: {0}")]
    Connector(#[from] ConnectorError),

    #[error("forward error: {0}")]
    Forward(#[from] ForwardError),

    #[error("buffer error: {0}")]
    Buffer(#[from] BufferError),

    #[error("scheduler error: {0}")]
    Scheduler(#[from] SchedulerError),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

/// TCP listener errors.
#[derive(Debug, Error)]
pub enum ListenerError {
    #[error("failed to bind TCP listener at {addr}: {source}")]
    Bind {
        addr: SocketAddr,
        #[source]
        source: std::io::Error,
    },

    #[error("failed to accept TCP connection at {addr}: {source}")]
    Accept {
        addr: SocketAddr,
        #[source]
        source: std::io::Error,
    },

    #[error("listener lifecycle failed: {reason}")]
    Lifecycle { reason: String },

    #[error("listener scheduling failed: {0}")]
    Schedule(#[from] SchedulerError),
}

/// TCP connector errors.
#[derive(Debug, Error)]
pub enum ConnectorError {
    #[error("failed to connect to {addr}: {source}")]
    Connect {
        addr: SocketAddr,
        #[source]
        source: std::io::Error,
    },

    #[error("connect timeout after {timeout:?} for {addr}")]
    ConnectTimeout {
        addr: SocketAddr,
        timeout: Duration,
    },

    #[error("connect retry attempts exhausted for {addr} after {attempts} attempts")]
    RetryExhausted { addr: SocketAddr, attempts: u32 },

    #[error("invalid connector state transition: {from:?} -> {to:?}")]
    InvalidStateTransition {
        from: ConnectionState,
        to: ConnectionState,
    },
}

/// Forwarding pipeline errors.
#[derive(Debug, Error)]
pub enum ForwardError {
    #[error("forwarding I/O failed: {source}")]
    Io {
        #[source]
        source: std::io::Error,
    },

    #[error("forward idle timeout after {timeout:?}")]
    IdleTimeout { timeout: Duration },

    #[error("forward shutdown was requested")]
    Shutdown,

    #[error("forwarding is paused")]
    Paused,

    #[error("invalid forward state transition: {from:?} -> {to:?}")]
    InvalidStateTransition { from: ForwardState, to: ForwardState },
}

/// Buffer pool errors.
#[derive(Debug, Error)]
pub enum BufferError {
    #[error("requested buffer size {requested} exceeds limit {limit}")]
    SizeLimitExceeded { requested: usize, limit: usize },
}

/// Scheduler and worker-pool errors.
#[derive(Debug, Error)]
pub enum SchedulerError {
    #[error("task limit exceeded: max_tasks={max_tasks}")]
    TaskLimitExceeded { max_tasks: usize },

    #[error("task was not found: {id}")]
    TaskNotFound { id: TaskId },

    #[error("task join failed: {name}: {source}")]
    Join {
        name: String,
        #[source]
        source: tokio::task::JoinError,
    },

    #[error("task shutdown timed out after {timeout:?}: {name}")]
    ShutdownTimeout { name: String, timeout: Duration },
}
