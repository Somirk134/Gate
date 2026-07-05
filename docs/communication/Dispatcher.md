# Dispatcher

Dispatcher turns protocol messages into framework actions.

## Responsibilities

- Dispatch request messages by command.
- Dispatch response messages by request id.
- Dispatch events to subscribers.
- Register and remove command handlers.
- Keep Project, Tunnel, Server, System, and Log handlers empty until business layers are implemented.

## Command Routing

```mermaid
flowchart LR
    Message["Message"]
    Type{"MessageType"}
    Request["Request"]
    Response["Response"]
    Event["Event"]
    Router["MessageRouter"]
    RequestManager["RequestManager"]
    EventDispatcher["EventDispatcher"]
    Handler["Empty Handler"]

    Message --> Type
    Type -->|Request| Request
    Type -->|Response| Response
    Type -->|Event / Broadcast / Notification| Event
    Request --> Router
    Router --> Handler
    Response --> RequestManager
    Event --> EventDispatcher
```

## Empty Handlers

- `ProjectHandler`
- `TunnelHandler`
- `ServerHandler`
- `SystemHandler`
- `LogHandler`

Each handler returns an empty framework response today. Real business behavior should be added in later business modules, not inside the communication foundation.
