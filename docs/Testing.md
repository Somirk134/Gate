# Testing

## Integration Tests

The Alpha V1 integration suite lives in `integration/tests`.

| Test | Purpose |
| --- | --- |
| `client_connect_test.rs` | Client connects to a real TCP protocol server. |
| `authentication_test.rs` | Valid token creates a session; invalid token is rejected. |
| `heartbeat_test.rs` | Ping/Pong updates RTT and server heartbeat statistics. |
| `tunnel_test.rs` | Real `TunnelRuntime` starts and accepts a session. |
| `forward_test.rs` | TCP bytes are forwarded through listener, connector, and target. |
| `reconnect_test.rs` | Client disconnects, reconnects, and authenticates again. |
| `statistics_test.rs` | Runtime counters come from the real request flow. |
| `shutdown_test.rs` | Server and runtime enter shutdown state cleanly. |
| `performance_simulation_test.rs` | Declares 100/500/1000/5000 connection simulation plans without executing load. |

## Commands

```powershell
cargo test -p gate-integration
cargo test -p gate-communication
cargo test -p gate-engine
npm --prefix client run typecheck
npm --prefix client run build
```

## Performance Framework

Alpha V1 does not run load tests. It defines simulation plans only:

- 100 connections
- 500 connections
- 1000 connections
- 5000 connections

Each plan contains connection count, ramp-up window, hold window, and `execute_load = false`.

## Test Rules

- Integration tests must use real TCP sockets on loopback.
- Protocol tests must encode/decode real frames.
- Runtime tests must use `TunnelRuntime`, not mock runtime.
- Mock tests stay isolated under mock-specific directories.
