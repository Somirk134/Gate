# Communication Architecture

The Communication Layer is the infrastructure boundary between Gate clients and Gate servers. It owns connection lifecycle, transport abstraction, message routing, request/response correlation, event distribution, session context, retry policy, timeout policy, logging, metrics, and mocks.

It does not implement Tunnel, authentication, heartbeat loops, business workflows, or real network IO.

## Module Graph

```mermaid
flowchart TB
    UI["Vue3 / Tauri Client UI"]
    Tauri["Rust Command Boundary"]
    ClientTS["client/src/communication"]
    RustComm["crates/communication"]
    Protocol["crates/protocol"]
    Server["server Tokio Runtime"]

    UI --> ClientTS
    ClientTS --> Tauri
    Tauri --> RustComm
    Server --> RustComm
    RustComm --> Protocol

    subgraph Communication["Communication Layer"]
        Client["client"]
        ServerMod["server"]
        Transport["transport"]
        Connection["connection"]
        Dispatcher["dispatcher"]
        Handler["handler"]
        Session["session"]
        Router["router"]
        Reconnect["reconnect"]
        Timeout["timeout"]
        Queue["queue"]
        Metrics["metrics"]
        Shared["shared"]
        Mock["mock"]

        Client --> Transport
        ServerMod --> Transport
        Client --> Connection
        ServerMod --> Connection
        Client --> Dispatcher
        ServerMod --> Dispatcher
        Dispatcher --> Router
        Router --> Handler
        Dispatcher --> Queue
        Client --> Session
        ServerMod --> Session
        Transport --> Reconnect
        Transport --> Timeout
        Connection --> Metrics
        Mock --> Transport
        Mock --> Dispatcher
        Shared --> Metrics
    end
```

## Request Sequence

```mermaid
sequenceDiagram
    participant UI as Vue / Service
    participant API as CommunicationService
    participant RM as ClientRequestManager
    participant TX as ClientTransport
    participant RX as ClientDispatcher
    participant SR as ServerDispatcher
    participant RT as MessageRouter
    participant H as Empty Handler

    UI->>API: request(command, body)
    API->>RM: create + register RequestId
    API->>TX: send(Message)
    TX-->>SR: future transport adapter
    SR->>RT: route(Command)
    RT->>H: handle_request()
    H-->>SR: empty Response
    SR-->>RX: MessageType.Response
    RX->>RM: resolve(RequestId)
    RM-->>API: Promise<Response>
    API-->>UI: Response
```

## Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Created
    Created --> Connecting
    Connecting --> Connected
    Connected --> Authenticated: reserved
    Connected --> Running
    Authenticated --> Running
    Running --> Reconnecting
    Reconnecting --> Connected
    Connected --> Disconnected
    Authenticated --> Disconnected
    Running --> Disconnected
    Reconnecting --> Disconnected
    Connecting --> Disconnected
    Disconnected --> Connecting
    Disconnected --> Closed
    Created --> Closed
    Connecting --> Failed
    Connected --> Failed
    Authenticated --> Failed
    Running --> Failed
    Reconnecting --> Failed
    Failed --> Closed
    Closed --> [*]
```

## Naming Rules

- Rust crate: `gate-communication`.
- Rust modules: singular nouns for concepts, role modules for `client` and `server`.
- Rust traits: capability nouns, for example `Transport`, `Dispatcher`, `Connection`, `Session`, `RequestHandler`, `ResponseHandler`, `EventHandler`.
- Rust structs: role-prefixed when role-specific, for example `ClientTransport`, `ServerConnection`, `ClientRequestManager`.
- TypeScript interfaces: domain nouns, for example `Connection`, `Request`, `Response`, `Message`, `Event`, `Session`, `Transport`.
- TypeScript classes: role-prefixed service objects, for example `CommunicationService`, `ClientDispatcher`, `MockTransport`.
- Commands: dotted protocol commands from `gate-protocol`, for example `project.create`, `server.status`, `system.health`.
- States: lowercase strings in TypeScript, PascalCase enum variants in Rust.
