# Integration

Alpha V1 integration connects real module boundaries and keeps mocks out of the runtime path. Test mocks may remain under dedicated mock/test directories.

## Full Link Sequence

```mermaid
sequenceDiagram
    participant UI as Dashboard / Services
    participant IPC as Tauri IPC
    participant CR as ClientRuntimeState
    participant CT as TcpTransport
    participant P as Protocol
    participant S as Server Bootstrap
    participant A as Authentication
    participant H as Heartbeat
    participant R as TunnelRuntime
    participant M as Monitoring
    participant L as Log Center

    UI->>IPC: connect(serverAddr, token)
    IPC->>CR: command connect
    CR->>CT: TCP connect
    CT->>S: framed auth.login
    S->>P: decode frame
    P->>A: validate token
    A-->>S: session accepted
    S-->>CT: auth response(sessionId)
    CT-->>CR: response
    CR-->>UI: connected + authenticated

    UI->>IPC: heartbeat()
    CR->>CT: heartbeat.ping
    CT->>S: Ping
    S->>H: update heartbeat
    H-->>S: Pong
    S-->>CT: heartbeat.pong
    CR->>M: update RTT/statistics
    M-->>UI: runtime_get_dashboard

    UI->>IPC: create_tunnel
    CR->>CT: tunnel.create
    S->>R: runtime lifecycle available
    R->>R: listen -> accept -> session -> forward
    R->>M: runtime metrics
    M->>L: runtime/protocol/auth/heartbeat logs
```

## Integrated Modules

- Client: `CommunicationService`, auth/tunnel/connection/project/server service facades, Tauri IPC, monitoring services.
- Server: boot, listen, accept, authenticate, heartbeat, statistics, shutdown.
- Protocol: framed JSON V1 request/response/event/heartbeat messages.
- Communication: real `TcpTransport`, reconnect, send, receive.
- Tunnel: real `TunnelRuntime`, TCP listener, connector, session manager, forward pipeline, runtime metrics.
- Dashboard: live runtime dashboard via `runtime_get_dashboard`.
- Log Center: runtime logs via `runtime_get_logs`.
- Settings: runtime config via `get_config` and `set_config`.

## Mock Cleanup Rule

- Runtime paths must not instantiate mock services.
- Mock data may remain only for isolated component or mock tests.
- New integration tests must use `gate-integration` harness, `TcpTransport`, and `TunnelRuntime`.
