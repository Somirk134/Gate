# Runtime

`TunnelRuntime` is the V1 TCP runtime facade. It wires together listener,
connector, forward pipeline, sessions, buffer pool, scheduler, workers, and
monitoring.

## Public API

- `TunnelRuntime`: single TCP tunnel runtime.
- `RuntimeManager`: manages multiple `TunnelRuntime` instances and keeps the
  older task-manager API used by engine foundation code.
- `RuntimeContext`: shared runtime state and shutdown channel.
- `RuntimeState`: runtime lifecycle state machine.
- `RuntimeBuilder`: builder for `TunnelRuntime`.
- `RuntimeLifecycle`: async lifecycle trait.

## Lifecycle Operations

- `start`: bind TCP listener and accept new clients.
- `stop`: stop listener, request shutdown, drain tasks, close sessions.
- `restart`: stop and start again.
- `pause`: keep listener task alive but stop accepting new sessions.
- `resume`: continue accepting sessions.
- `shutdown`: graceful terminal shutdown.

## Configuration

`RuntimeConfig` owns:

- `ListenerConfig`
- `ConnectorConfig`
- `BufferConfig`
- `RetryConfig`
- `TimeoutConfig`
- worker, task, session, monitor, and cleanup limits

At the crate root, the data-plane runtime config is exported as
`TunnelRuntimeConfig` to avoid confusion with the older engine
`config::RuntimeConfig`.

## Logging

The runtime writes structured tracing logs for:

- `Runtime Start`
- `Session Create`
- `Session Close`
- `Connect Success`
- `Connect Failed`
- `Forward Started`
- `Forward Finished`

All runtime logs use the `gate_runtime` tracing target.
