# Tunnel Runtime Architecture

Tunnel Runtime is the V1 TCP data-plane runtime for Gate. It is hosted in
`crates/engine/src/runtime` and is intentionally separated from protocol,
communication, and UI layers.

V1 supports TCP only. HTTP, HTTPS, UDP, P2P, custom copy loops, io_uring, and
mio are extension points, not V1 behavior.

## Module Layout

- `runtime`: public runtime facade and lifecycle.
- `listener`: TCP bind and accept service.
- `connector`: TCP target connector with timeout and retry.
- `forward`: bidirectional TCP forwarding pipeline.
- `session`: session identity, lifecycle, and registry.
- `buffer`: reusable buffer pool boundary.
- `stream`: stream wrappers and stream statistics hooks.
- `worker`: Tokio task registry.
- `scheduler`: centralized task scheduling API.
- `monitor`: traffic counters and runtime metrics.
- `mock`: debug mocks for client integration.

## Architecture

```mermaid
flowchart TB
    UI["Client UI / Debug Tools"]
    Manager["RuntimeManager"]
    Runtime["TunnelRuntime"]
    Context["RuntimeContext"]
    Listener["TcpListenerService"]
    Connector["TcpConnector"]
    Forward["ForwardPipeline"]
    Sessions["SessionManager"]
    Buffers["BufferPool"]
    Scheduler["RuntimeScheduler"]
    Workers["WorkerPool"]
    Monitor["RuntimeMonitor"]
    Stats["TrafficStatistics"]

    UI --> Manager
    Manager --> Runtime
    Runtime --> Context
    Runtime --> Listener
    Runtime --> Connector
    Runtime --> Forward
    Runtime --> Scheduler
    Runtime --> Monitor
    Context --> Sessions
    Context --> Buffers
    Context --> Stats
    Scheduler --> Workers
    Listener --> Sessions
    Listener --> Scheduler
    Listener --> Connector
    Connector --> Forward
    Forward --> Stats
    Forward --> Buffers
    Monitor --> Stats
    Monitor --> Sessions
```

## Data Flow

```mermaid
sequenceDiagram
    participant Client
    participant Listener as TcpListenerService
    participant Session as SessionManager
    participant Scheduler as RuntimeScheduler
    participant Connector as TcpConnector
    participant Forward as ForwardPipeline
    participant Target
    participant Stats as TrafficStatistics

    Client->>Listener: TCP connect
    Listener->>Session: create Session
    Listener->>Scheduler: spawn Forward Task
    Scheduler->>Connector: connect target
    Connector->>Target: TCP connect with retry
    Connector-->>Scheduler: target stream
    Scheduler->>Forward: start copy_bidirectional
    Forward->>Target: Client -> Target bytes
    Forward->>Client: Target -> Client bytes
    Forward->>Stats: upload/download counters
    Forward-->>Session: finish or fail
    Session-->>Listener: close Session
```

## Runtime Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Created
    Created --> Starting: start
    Created --> ShuttingDown: shutdown
    Starting --> Running: listener ready
    Starting --> Error: start failed
    Running --> Paused: pause
    Running --> Stopping: stop
    Running --> Restarting: restart
    Running --> ShuttingDown: shutdown
    Running --> Error: fatal error
    Paused --> Running: resume
    Paused --> Stopping: stop
    Paused --> Restarting: restart
    Paused --> ShuttingDown: shutdown
    Stopping --> Stopped: tasks drained
    Stopped --> Starting: start
    Stopped --> ShuttingDown: shutdown
    Restarting --> Stopping: drain old runtime
    Restarting --> Starting: start again
    ShuttingDown --> Shutdown: graceful shutdown complete
    Error --> Stopping: stop
    Error --> ShuttingDown: shutdown
    Error --> Shutdown: force close
    Shutdown --> [*]
```

## Session Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Created
    Created --> Connecting: accepted by listener
    Connecting --> Forwarding: target connected
    Connecting --> Failed: connect failed
    Forwarding --> Paused: runtime paused
    Paused --> Forwarding: runtime resumed
    Forwarding --> Closing: EOF or shutdown
    Forwarding --> Failed: I/O error or idle timeout
    Closing --> Closed
    Failed --> Closed
    Closed --> [*]
```

## Naming Convention

- Runtime types use `Runtime*`: `RuntimeConfig`, `RuntimeContext`,
  `RuntimeState`, `RuntimeBuilder`, `RuntimeLifecycle`.
- TCP-specific implementations use `Tcp*`: `TcpListenerService`,
  `TcpConnector`.
- Protocol-neutral extension points use purpose names:
  `ForwardPipeline`, `BufferPool`, `SessionManager`, `RuntimeScheduler`.
- State enums use `*State`: `RuntimeState`, `SessionState`,
  `ConnectionState`, `ForwardState`.
- Errors use `*Error`: `RuntimeError`, `ListenerError`, `ConnectorError`,
  `ForwardError`, `BufferError`, `SchedulerError`.

## Coding Convention

- All Tokio tasks are spawned through `RuntimeScheduler`.
- Public structs, traits, and enums carry Rust doc comments.
- V1 data forwarding uses `tokio::io::copy_bidirectional`.
- Shared mutable runtime state uses `Arc`, `DashMap`, atomics, or small locks.
- Runtime shutdown is signaled through a `watch` channel and drained through
  `WorkerPool::graceful_shutdown`.
- Future HTTP, HTTPS, UDP, and P2P code should add new listener, connector,
  and forward implementations without changing the TCP public API.
