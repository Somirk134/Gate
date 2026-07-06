//! Tunnel Engine architecture for Gate.
//!
//! This crate defines the core engine boundaries, lifecycle, traits, events,
//! errors, and extension points required by future HTTP, TCP, HTTPS, UDP, and
//! P2P tunnel implementations.

pub mod config;
pub mod connection;
pub mod connection_monitor;
pub mod core;
pub mod error;
pub mod event;
pub mod forwarder;
pub mod health;
pub mod heartbeat;
pub mod listener;
pub mod mock;
pub mod pipeline;
pub mod reconnect;
pub mod repository;
pub mod router;
pub mod runtime;
pub mod session;
pub mod session_recovery;
pub mod statistics;
pub mod state_sync;
pub mod transport;

pub use config::{
    ForwardConfig, ForwardConfigBuilder, HealthConfig, HealthConfigBuilder, HeartbeatConfig,
    HeartbeatConfigBuilder, ProtocolConfig, ProtocolConfigBuilder, ProtocolKind,
    ReconnectConfig, ReconnectConfigBuilder, ReconnectStrategyConfig, RuntimeConfig,
    RuntimeConfigBuilder, SyncConfig, SyncConfigBuilder, TunnelConfig, TunnelConfigBuilder,
};
pub use core::{
    EngineBuilder, EngineConfig, EngineConfigBuilder, EngineContext, EngineLifecycle,
    EngineManager, EnginePhase, EngineState, Tunnel, TunnelEngine, TunnelId, TunnelStatus,
};
pub use connection_monitor::{
    ConnectionHealth, ConnectionMonitor, ConnectionMonitorManager, ConnectionMonitorSnapshot,
};
pub use error::{
    ConnectionError, ConnectionLostError, EngineError, ForwardError, HeartbeatError,
    ProtocolError, ReconnectError, RecoveryError, StateSyncError, TunnelError,
};
pub use health::{HealthCheckTarget, HealthChecker, HealthManager, HealthReport, HealthSignal, HealthStatus};
pub use heartbeat::{Heartbeat, HeartbeatManager, HeartbeatMetrics, HeartbeatSnapshot, HeartbeatState};
pub use reconnect::{
    CustomReconnectStrategy, ExponentialBackoffStrategy, FixedIntervalStrategy, ImmediateStrategy,
    LinearStrategy, Reconnect, ReconnectManager, ReconnectMode, ReconnectRequest,
    ReconnectSnapshot, ReconnectState, ReconnectStrategy, ReconnectStrategyKind,
};
pub use runtime::{
    BackoffStrategy, BufferConfig as RuntimeBufferConfig, ConnectorConfig, ForwardPipeline,
    ListenerConfig, RetryConfig, RuntimeBuilder, RuntimeLifecycle, RuntimeManager,
    RuntimeMetrics, RuntimeState, TcpConnector, TcpListenerService, TimeoutConfig,
    TrafficStatistics, TunnelRuntime, RuntimeConfig as TunnelRuntimeConfig,
};
pub use session_recovery::{
    Recovery, RecoveryContext, RecoveryResult, SessionRecoveryManager, SubscriptionSnapshot,
};
pub use state_sync::{
    StateSnapshot, StateSyncManager, StateSyncResult, StateSynchronizer, SyncTarget,
};
