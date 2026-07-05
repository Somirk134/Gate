# Tunnel Engine

Tunnel Engine is the protocol-agnostic core of Gate. This phase establishes the
architecture, contracts, lifecycle, configuration model, events, errors, and
extension points. It intentionally does not implement real data forwarding,
authentication, encryption, database persistence, or protocol parsing.

## Core Types

- `TunnelEngine`: public engine facade.
- `EngineContext`: shared configuration, state, runtime, and event publisher.
- `EngineConfig`: engine-wide configuration.
- `EngineState`: thread-safe lifecycle state holder.
- `EngineManager`: aggregate for router, connection, session, heartbeat, and health managers.
- `EngineLifecycle`: lifecycle trait for initialize, start, pause, stop, and restart.
- `EngineBuilder`: construction entry point.

## Module Layout

```text
crates/engine/src/
  config/
  connection/
  core/
  error/
  event/
  forwarder/
  health/
  heartbeat/
  listener/
  pipeline/
  repository/
  router/
  runtime/
  session/
  statistics/
  transport/
```

## Public Boundary

The engine crate exports stable public contracts from `crates/engine/src/lib.rs`.
Future protocol implementations should depend on traits instead of concrete
managers whenever possible.

## Current Scope

- Lifecycle state machine.
- Tunnel, listener, connector, forwarder, protocol, pipeline, repository traits.
- Mock statistics model.
- Health check interfaces.
- Event and error taxonomy.
- Runtime task manager facade.
- Test and documentation directories.

## Out Of Scope

- Real byte forwarding.
- Authentication.
- Encryption and TLS.
- Database persistence.
- Full HTTP/TCP/UDP/P2P protocol parsing.
- QUIC transport implementation.
