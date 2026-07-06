//! Runtime reliability control plane.
//!
//! These components are intentionally protocol-neutral. They supervise,
//! observe, recover, and shut down existing TCP/HTTP/HTTPS runtimes without
//! adding new tunnel types or data-plane behavior.

pub mod connection_manager;
pub mod graceful_shutdown;
pub mod health;
pub mod metrics;
pub mod recovery;
pub mod supervisor;
pub mod task_manager;
pub mod trace;
pub mod watchdog;

pub use connection_manager::{
    ConnectionManagerError, ConnectionPolicy, RuntimeConnectionManager, RuntimeConnectionSnapshot,
};
pub use graceful_shutdown::{
    GracefulShutdownManager, GracefulShutdownReport, ShutdownHook, ShutdownResource,
    ShutdownResourceKind, ShutdownResourceResult,
};
pub use health::{
    RuntimeHealthCheck, RuntimeHealthReport, RuntimeHealthSignal, RuntimeHealthTarget,
};
pub use metrics::{RuntimeMetricsRegistry, RuntimeReliabilityMetrics};
pub use recovery::{RecoveryDecision, RecoveryEvent, RecoveryTrigger, RuntimeRecoveryFlow};
pub use supervisor::{
    CircuitBreakerConfig, CircuitBreakerState, RestartPolicy, RuntimeIdentity, RuntimeSupervisor,
    SupervisorAction, SupervisorConfig, SupervisorSnapshot,
};
pub use task_manager::RuntimeTaskManager;
pub use trace::{RuntimeId, RuntimeTraceContext, TraceId};
pub use watchdog::{
    RuntimeWatchdog, WatchdogConfig, WatchdogFinding, WatchdogFindingKind, WatchdogSeverity,
};
