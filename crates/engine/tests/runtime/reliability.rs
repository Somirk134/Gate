use futures::future::BoxFuture;
use gate_engine::core::TunnelId;
use gate_engine::health::HealthStatus;
use gate_engine::runtime::{
    ConnectionPolicy, RecoveryTrigger, RuntimeHealthCheck, RuntimeIdentity, RuntimeLifecycle,
    RuntimeReliabilityMetrics, RuntimeScheduler, RuntimeState, RuntimeStateMachine,
    RuntimeSupervisor, RuntimeTaskManager, RuntimeWatchdog, ShutdownHook, ShutdownResource,
    ShutdownResourceKind, SupervisorAction, TaskKind, WatchdogConfig, WatchdogFindingKind,
};
use gate_engine::{
    GracefulShutdownManager, RuntimeConnectionManager, RuntimeMetricsRegistry, RuntimeTraceContext,
};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::Duration;

#[test]
fn runtime_state_machine_uses_unified_lifecycle() {
    let state = RuntimeStateMachine::default();

    assert_eq!(state.current(), RuntimeState::Created);
    state.transition_to(RuntimeState::Initializing).unwrap();
    state.transition_to(RuntimeState::Starting).unwrap();
    state.transition_to(RuntimeState::Running).unwrap();
    state.transition_to(RuntimeState::Failed).unwrap();
    state.transition_to(RuntimeState::Restarting).unwrap();
    state.transition_to(RuntimeState::Starting).unwrap();
    state.transition_to(RuntimeState::Running).unwrap();
    state.transition_to(RuntimeState::Stopping).unwrap();
    state.transition_to(RuntimeState::Closed).unwrap();

    assert!(state.current().is_terminal());
    assert!(state.transition_to(RuntimeState::Starting).is_err());
}

#[tokio::test]
async fn supervisor_restarts_failed_runtime_through_recovery_flow() {
    let runtime = TestRuntime::new();
    runtime.start().await.unwrap();
    runtime.fail();

    let supervisor = RuntimeSupervisor::default();
    let identity = RuntimeIdentity::new(TunnelId::new(), "supervisor-test");
    let runtime_id = supervisor.register(identity, Arc::new(runtime.clone()));

    let action = supervisor
        .recover(
            runtime_id,
            RecoveryTrigger::RuntimeError,
            Some("test failure".into()),
        )
        .await
        .unwrap();

    assert_eq!(action, SupervisorAction::Restart);
    assert_eq!(runtime.state(), RuntimeState::Running);
    assert_eq!(supervisor.recovery().events().len(), 1);
    assert_eq!(supervisor.snapshot(runtime_id).unwrap().restart_count, 1);
}

#[test]
fn health_watchdog_metrics_and_tracing_are_unified() {
    let health = RuntimeHealthCheck::new();
    health.check_runtime_alive(RuntimeState::Running);
    health.check_listener_alive(true);
    health.check_tunnel_alive(true);
    health.check_tls_alive(true);
    health.check_connection_alive(true);
    health.check_heartbeat(true, 0);
    let report = health.report();
    assert_eq!(report.status, HealthStatus::Healthy);
    assert_eq!(report.signals.len(), 6);

    let watchdog = RuntimeWatchdog::new(WatchdogConfig {
        max_channel_depth: 1,
        ..WatchdogConfig::default()
    });
    let findings = watchdog.inspect(
        RuntimeReliabilityMetrics {
            channel_depth: 2,
            ..RuntimeReliabilityMetrics::default()
        },
        None,
        Some(Duration::from_secs(60)),
        Some(Duration::from_secs(20)),
    );
    assert!(findings
        .iter()
        .any(|finding| finding.kind == WatchdogFindingKind::ChannelBlocked));

    let runtime_id = Default::default();
    let tunnel_id = TunnelId::new();
    let trace = RuntimeTraceContext::new(runtime_id, tunnel_id);
    assert_eq!(trace.runtime_id, runtime_id);
    assert_eq!(trace.tunnel_id, tunnel_id);

    let registry = RuntimeMetricsRegistry::new();
    registry.upsert(
        runtime_id,
        RuntimeReliabilityMetrics::default().with_state(RuntimeState::Running),
    );
    assert_eq!(registry.aggregate().runtime_count, 1);
}

#[test]
fn connection_manager_enforces_limit_and_expires_idle() {
    let manager = RuntimeConnectionManager::new(ConnectionPolicy {
        max_connections: 1,
        idle_timeout: Duration::ZERO,
        ..ConnectionPolicy::default()
    });
    let remote = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 10001);
    let local = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 20001);

    let connection = manager
        .register(TunnelId::new(), remote, local)
        .expect("first connection should register");
    assert!(manager.register(TunnelId::new(), remote, local).is_err());

    manager.mark_connected(connection.connection_id).unwrap();
    let expired = manager.expire_timeouts();
    assert_eq!(expired.len(), 1);
    assert_eq!(manager.active_count(), 0);
}

#[tokio::test]
async fn graceful_shutdown_runs_registered_resource_hooks() {
    let manager = GracefulShutdownManager::new();
    let hook: ShutdownHook = Arc::new(|| Box::pin(async { Ok(()) }));
    manager.register(ShutdownResource::new(
        "listener",
        ShutdownResourceKind::Listener,
        hook,
    ));

    let report = manager.shutdown(Duration::from_secs(1)).await;

    assert!(report.success);
    assert_eq!(report.resources.len(), 1);
}

#[tokio::test]
async fn task_manager_marks_panicked_tasks_as_failed() {
    let scheduler = Arc::new(RuntimeScheduler::new(8));
    let manager = RuntimeTaskManager::new(scheduler);
    manager
        .spawn(TaskKind::Runtime, "panic-task", async {
            panic!("intentional reliability test panic");
        })
        .unwrap();

    tokio::time::sleep(Duration::from_millis(50)).await;

    assert_eq!(manager.statistics().failed, 1);
    assert_eq!(manager.failed_tasks().len(), 1);
}

#[derive(Clone)]
struct TestRuntime {
    state: Arc<RuntimeStateMachine>,
}

impl TestRuntime {
    fn new() -> Self {
        Self {
            state: Arc::new(RuntimeStateMachine::default()),
        }
    }

    fn fail(&self) {
        self.state.transition_to(RuntimeState::Failed).unwrap();
    }
}

impl RuntimeLifecycle for TestRuntime {
    fn state(&self) -> RuntimeState {
        self.state.current()
    }

    fn start(&self) -> BoxFuture<'static, Result<(), gate_engine::runtime::RuntimeError>> {
        let state = Arc::clone(&self.state);
        Box::pin(async move {
            match state.current() {
                RuntimeState::Created | RuntimeState::Stopped => {
                    state.transition_to(RuntimeState::Initializing)?;
                }
                RuntimeState::Restarting => {}
                RuntimeState::Running => return Ok(()),
                current => {
                    return Err(gate_engine::runtime::RuntimeError::InvalidStateTransition {
                        from: current,
                        to: RuntimeState::Starting,
                    });
                }
            }
            state.transition_to(RuntimeState::Starting)?;
            state.transition_to(RuntimeState::Running)?;
            Ok(())
        })
    }

    fn stop(&self) -> BoxFuture<'static, Result<(), gate_engine::runtime::RuntimeError>> {
        let state = Arc::clone(&self.state);
        Box::pin(async move {
            state.transition_to(RuntimeState::Stopping)?;
            state.transition_to(RuntimeState::Stopped)?;
            Ok(())
        })
    }

    fn restart(&self) -> BoxFuture<'static, Result<(), gate_engine::runtime::RuntimeError>> {
        let runtime = self.clone();
        Box::pin(async move {
            if runtime.state() == RuntimeState::Failed {
                runtime.state.transition_to(RuntimeState::Restarting)?;
            }
            runtime.start().await
        })
    }

    fn pause(&self) -> BoxFuture<'static, Result<(), gate_engine::runtime::RuntimeError>> {
        let state = Arc::clone(&self.state);
        Box::pin(async move {
            state.transition_to(RuntimeState::Paused)?;
            Ok(())
        })
    }

    fn resume(&self) -> BoxFuture<'static, Result<(), gate_engine::runtime::RuntimeError>> {
        let state = Arc::clone(&self.state);
        Box::pin(async move {
            state.transition_to(RuntimeState::Running)?;
            Ok(())
        })
    }

    fn shutdown(&self) -> BoxFuture<'static, Result<(), gate_engine::runtime::RuntimeError>> {
        let state = Arc::clone(&self.state);
        Box::pin(async move {
            state.transition_to(RuntimeState::Stopping)?;
            state.transition_to(RuntimeState::Closed)?;
            Ok(())
        })
    }
}
