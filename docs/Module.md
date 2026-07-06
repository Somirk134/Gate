# Gate Server Module Guide

## Domain modules

The domain layer currently reserves these modules:

- `Project`
- `Tunnel`
- `Server`
- `Connection`
- `Statistics`
- `Log`
- `Settings`
- `Session`
- `Health`
- `Event`

Each domain module should keep the same file structure:

```text
module/
  mod.rs
  service.rs
  repository.rs
  entity.rs
  error.rs
  event.rs
  handler.rs
  types.rs
```

## File responsibilities

| File | Responsibility |
| --- | --- |
| `entity.rs` | Aggregates, entities, and value objects when the module becomes active |
| `repository.rs` | Domain repository traits |
| `service.rs` | Domain service traits and domain rules |
| `error.rs` | Module-local domain errors |
| `event.rs` | Domain events |
| `handler.rs` | Domain event handler boundary |
| `types.rs` | Type aliases and value object types |
| `mod.rs` | Module declarations and narrow re-exports |

## Repository traits

Current repository traits include:

- `ProjectRepository`
- `TunnelRepository`
- `ServerRepository`
- `SettingsRepository`
- `LogRepository`

These traits should gain methods only from concrete use-case needs. Do not design CRUD surfaces for
database tables before the application layer needs them.

## Application traits

The application layer exposes these abstractions:

- `Service`
- `UseCase`
- `Command`
- `Query`
- `Handler`
- `CommandDispatcher`
- `QueryDispatcher`
- `EventDispatcher`
- `EventHandler`

The application layer describes what the system should do. It should not depend on transport
protocols, storage engines, or deployment details.

## Infrastructure traits

The infrastructure layer provides implementation ports:

- `Storage`
- `ConfigProvider`
- `LoggerProvider`
- `Cache`
- `Network`
- `RuntimeProvider`
- `Scheduler`

SQLx, Redis, TLS, file logging, JSON logging, and MessagePack must enter as implementations behind
ports. Implementation types should not leak back into the domain layer.

## Transport traits

The transport layer currently exposes:

- `Transport`
- `TcpTransport`
- `TcpEndpoint`
- `HttpTransport`
- `IpcTransport`
- `WebSocketTransport`

TCP is the current reserved transport module. It should not listen on ports, parse protocol frames,
or forward data unless runtime ownership is explicit.
