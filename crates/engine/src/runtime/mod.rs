//! TCP Tunnel Runtime.
//!
//! The runtime owns the data plane for V1 TCP tunnels: listener, connector,
//! session registry, bidirectional forwarding, statistics, scheduler, worker
//! tasks, and graceful shutdown coordination.

pub mod buffer;
pub mod config;
pub mod connector;
pub mod context;
pub mod error;
pub mod forward;
pub mod lifecycle;
pub mod listener;
pub mod manager;
pub mod mock;
pub mod monitor;
pub mod scheduler;
pub mod session;
pub mod state;
pub mod stream;
pub mod tunnel_runtime;
pub mod worker;

pub use buffer::{Buffer, BufferPool};
pub use config::{
    BackoffStrategy, BufferConfig, BufferConfigBuilder, ConnectorConfig,
    ConnectorConfigBuilder, ListenerConfig, ListenerConfigBuilder, RetryConfig,
    RetryConfigBuilder, RuntimeConfig, RuntimeConfigBuilder, TimeoutConfig,
    TimeoutConfigBuilder,
};
pub use connector::TcpConnector;
pub use context::RuntimeContext;
pub use error::{
    BufferError, ConnectorError, ForwardError, ListenerError, RuntimeError,
    SchedulerError,
};
pub use forward::{ForwardPipeline, ForwardResult};
pub use lifecycle::RuntimeLifecycle;
pub use listener::TcpListenerService;
pub use manager::RuntimeManager;
pub use mock::{MockForward, MockRuntime, MockSession, MockTraffic};
pub use monitor::{RuntimeMetrics, RuntimeMonitor, TrafficSnapshot, TrafficStatistics};
pub use scheduler::RuntimeScheduler;
pub use session::{Session, SessionId, SessionManager, SessionSnapshot};
pub use state::{
    ConnectionState, ForwardState, RuntimeState, RuntimeStateMachine, SessionState,
};
pub use stream::{
    InstrumentedStream, StreamContext, StreamReader, StreamRole, StreamStatistics,
    StreamWriter,
};
pub use tunnel_runtime::{RuntimeBuilder, TunnelRuntime};
pub use worker::{Task, TaskId, TaskKind, TaskStatus, WorkerPool};

/// Backwards-compatible alias for older engine code that used `RuntimeStatus`.
pub type RuntimeStatus = RuntimeState;
