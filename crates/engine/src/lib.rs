//! Tunnel Engine architecture for Gate.
//!
//! This crate defines the core engine boundaries, lifecycle, traits, events,
//! errors, and extension points required by future HTTP, TCP, HTTPS, UDP, and
//! P2P tunnel implementations.

pub mod config;
pub mod connection;
pub mod core;
pub mod error;
pub mod event;
pub mod forwarder;
pub mod health;
pub mod heartbeat;
pub mod listener;
pub mod pipeline;
pub mod repository;
pub mod router;
pub mod runtime;
pub mod session;
pub mod statistics;
pub mod transport;

pub use config::{
    ForwardConfig, ForwardConfigBuilder, HeartbeatConfig, HeartbeatConfigBuilder, ProtocolConfig,
    ProtocolConfigBuilder, ProtocolKind, RuntimeConfig, RuntimeConfigBuilder, TunnelConfig,
    TunnelConfigBuilder,
};
pub use core::{
    EngineBuilder, EngineConfig, EngineConfigBuilder, EngineContext, EngineLifecycle,
    EngineManager, EnginePhase, EngineState, Tunnel, TunnelEngine, TunnelId, TunnelStatus,
};
pub use error::{ConnectionError, EngineError, ForwardError, ProtocolError, TunnelError};
