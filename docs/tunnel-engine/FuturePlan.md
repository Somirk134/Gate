# Future Plan

## Phase 1: Core Architecture

- Establish `gate-engine`.
- Define lifecycle, traits, config, events, errors, runtime, health, statistics.
- Add tests and documentation directories.

## Phase 2: TCP Tunnel MVP

- Implement TCP listener and connector.
- Add connection accept loop.
- Add simple bidirectional copy using Tokio.
- Add basic runtime task cancellation.
- Add focused unit and integration tests.

## Phase 3: HTTP Tunnel

- Add HTTP listener adapter.
- Add HTTP route matching.
- Add request metadata model.
- Add basic observability hooks.

## Phase 4: Security

- Add authentication boundary.
- Add rustls support.
- Add certificate and key configuration.
- Add audit events.

## Phase 5: Protocol Expansion

- Add HTTPS.
- Add UDP.
- Add WebSocket transport.
- Add QUIC transport with Quinn.
- Add P2P discovery and relay architecture.

## Phase 6: Persistence And Operations

- Implement `TunnelRepository`.
- Add durable session store.
- Add runtime recovery.
- Add benchmark suite.
- Add load balancing strategies in `TunnelRouter`.
