# Best Practice

## Naming

- Rust manager: `HeartbeatManager`, `ReconnectManager`, `SessionRecoveryManager`, `ConnectionMonitorManager`, `StateSyncManager`, `HealthManager`.
- Rust trait: `Heartbeat`, `Reconnect`, `Recovery`, `ConnectionMonitor`, `StateSynchronizer`, `HealthChecker`.
- Rust config: `HeartbeatConfig`, `ReconnectConfig`, `HealthConfig`, `SyncConfig`.
- Rust error: `HeartbeatError`, `ReconnectError`, `RecoveryError`, `ConnectionLostError`, `StateSyncError`.
- TypeScript service: `HeartbeatService`, `ReconnectService`, `ConnectionMonitorService`, `SessionRecoveryService`, `HealthService`, `StateSyncService`.
- TypeScript state strings use snake case: `waiting_pong`, `exponential_backoff`, `tunnel_state`.

## Operational Guidance

- Heartbeat timeout should trigger reconnect, not business recovery directly.
- Reconnect success should trigger session recovery, then state sync.
- Recovery must not restore business payloads or database state.
- State sync should use versioned snapshots and remain idempotent.
- Health checks should consume signals from managers instead of probing sockets directly.
- Mock scenarios should exercise the same public manager/service APIs used by production adapters.

## Million-Connection Readiness

- Keep per-connection state compact and serializable.
- Avoid global locks in hot paths.
- Bound reconnect queues.
- Prefer event-driven updates over polling every connection from a central task.
- Keep log cursor sync optional until the log subsystem has backpressure control.
