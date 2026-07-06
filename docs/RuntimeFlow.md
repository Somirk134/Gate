# Runtime Flow

## Lifecycle

```mermaid
stateDiagram-v2
    [*] --> ClientStart
    ClientStart --> ClientConnect
    ClientConnect --> ClientAuthenticate
    ClientAuthenticate --> ClientHeartbeat
    ClientHeartbeat --> ClientTunnel
    ClientTunnel --> ClientRunning
    ClientRunning --> ClientDisconnect
    ClientDisconnect --> ClientReconnect
    ClientReconnect --> ClientAuthenticate
    ClientRunning --> ClientShutdown
    ClientShutdown --> [*]

    [*] --> ServerBoot
    ServerBoot --> ServerListen
    ServerListen --> ServerAccept
    ServerAccept --> ServerAuthenticate
    ServerAuthenticate --> ServerRuntime
    ServerRuntime --> ServerRunning
    ServerRunning --> ServerShutdown
    ServerShutdown --> [*]
```

## Tunnel Flow

```mermaid
sequenceDiagram
    participant C as Client TCP
    participant L as Runtime Listener
    participant S as Session Manager
    participant X as Connector
    participant F as Forward Pipeline
    participant T as Target Service
    participant Stats as Runtime Statistics

    C->>L: connect
    L->>S: create session
    S-->>L: session id
    L->>X: connect target
    X->>T: TCP connect
    L->>F: start bidirectional forwarding
    C->>F: upload bytes
    F->>T: forward upload
    T->>F: response bytes
    F->>C: forward download
    F->>Stats: upload/download/session metrics
    C->>L: disconnect
    L->>S: close session
```

## Error Handling

- Client errors flow through service promises, UI error states, notification/toast handlers, and runtime logs.
- Server errors use tracing logs and structured protocol error responses.
- Recoverable connection loss uses reconnect through `TcpTransport::reconnect`.
- Authentication failure returns `AUTH_FAILED` and closes the session.
