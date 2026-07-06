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
pub mod http;
pub mod https;
pub mod lifecycle;
pub mod listener;
pub mod manager;
pub mod mock;
pub mod monitor;
pub mod reliability;
pub mod scheduler;
pub mod session;
pub mod state;
pub mod stream;
pub mod tunnel_runtime;
pub mod worker;

pub use buffer::{Buffer, BufferPool};
pub use config::{
    BackoffStrategy, BufferConfig, BufferConfigBuilder, ConnectorConfig, ConnectorConfigBuilder,
    ListenerConfig, ListenerConfigBuilder, RetryConfig, RetryConfigBuilder, RuntimeConfig,
    RuntimeConfigBuilder, TimeoutConfig, TimeoutConfigBuilder,
};
pub use connector::TcpConnector;
pub use context::RuntimeContext;
pub use error::{
    BufferError, ConnectorError, ForwardError, HttpsError, ListenerError, RuntimeError,
    SchedulerError,
};
pub use forward::{ForwardPipeline, ForwardResult};
pub use http::{
    HttpHeaderConfig, HttpHeaderRule, HttpHostResolver, HttpRequestLog, HttpRouteConfig,
    HttpRouteMetrics, HttpRuntimeBuilder, HttpRuntimeConfig, HttpRuntimeConfigBuilder,
    HttpRuntimeMetrics, HttpTlsLog, HttpTunnel, HttpTunnelRuntime,
};
pub use https::{
    CertificateSnapshot, HttpsCertificateProvider, HttpsRuntimeBuilder, HttpsRuntimeConfig,
    HttpsRuntimeMetrics, HttpsTlsConfig, HttpsTunnel, HttpsTunnelRuntime, TlsProtocolVersion,
};
pub use lifecycle::RuntimeLifecycle;
pub use listener::TcpListenerService;
pub use manager::RuntimeManager;
pub use mock::{MockForward, MockRuntime, MockSession, MockTraffic};
pub use monitor::{RuntimeMetrics, RuntimeMonitor, TrafficSnapshot, TrafficStatistics};
pub use reliability::{
    CircuitBreakerConfig, CircuitBreakerState, ConnectionManagerError, ConnectionPolicy,
    GracefulShutdownManager, GracefulShutdownReport, RecoveryDecision, RecoveryEvent,
    RecoveryTrigger, RestartPolicy, RuntimeConnectionManager, RuntimeConnectionSnapshot,
    RuntimeHealthCheck, RuntimeHealthReport, RuntimeHealthSignal, RuntimeHealthTarget, RuntimeId,
    RuntimeIdentity, RuntimeMetricsRegistry, RuntimeRecoveryFlow, RuntimeReliabilityMetrics,
    RuntimeSupervisor, RuntimeTaskManager, RuntimeTraceContext, RuntimeWatchdog, ShutdownHook,
    ShutdownResource, ShutdownResourceKind, ShutdownResourceResult, SupervisorAction,
    SupervisorConfig, SupervisorSnapshot, TraceId, WatchdogConfig, WatchdogFinding,
    WatchdogFindingKind, WatchdogSeverity,
};
pub use scheduler::RuntimeScheduler;
pub use session::{Session, SessionId, SessionManager, SessionSnapshot};
pub use state::{ConnectionState, ForwardState, RuntimeState, RuntimeStateMachine, SessionState};
pub use stream::{
    InstrumentedStream, StreamContext, StreamReader, StreamRole, StreamStatistics, StreamWriter,
};
pub use tunnel_runtime::{RuntimeBuilder, TunnelRuntime};
pub use worker::{Task, TaskId, TaskKind, TaskStatistics, TaskStatus, WorkerPool};

/// Backwards-compatible alias for older engine code that used `RuntimeStatus`.
pub type RuntimeStatus = RuntimeState;
