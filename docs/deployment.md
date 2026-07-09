# Deployment

This page describes deployment options for Gate alpha. Use it together with [Server](./server.md), [Docker](./docker.md), [Upgrade](./upgrade.md), and [Security](../SECURITY.md).

## Deployment Options

| Option | Status | Best for |
| --- | --- | --- |
| Source run | Supported | Development and local testing |
| Release binary | Planned | Simple server installation |
| Docker | Supported template | VPS, homelab, and repeatable demos |
| systemd | Template planned | Linux long-running service |
| Reverse proxy | Supported pattern | TLS termination and public host routing |
| Kubernetes | Future | Larger teams after health checks stabilize |

## Production-Like Checklist

- Build a release binary or image.
- Set `GATE_SERVER_ADDR=0.0.0.0:7000` for source/binary deployments, or `0.0.0.0:5800` for the Docker image.
- Set a non-default `GATE_AUTH_TOKEN`.
- Restrict inbound ports with a firewall.
- Put TLS at the reverse proxy when exposing HTTP entrypoints.
- Capture logs.
- Back up configuration and token material.
- Record the deployed version and commit SHA.

## Binary Deployment

```bash
cargo build -p gate-server --release
install -m 0755 target/release/gate-server /usr/local/bin/gate-server
```

Run:

```bash
GATE_SERVER_ADDR=0.0.0.0:7000 \
GATE_AUTH_TOKEN=replace-with-a-long-random-token \
gate-server
```

## Reverse Proxy Pattern

Use a reverse proxy for:

- TLS termination.
- HTTP routing.
- Request logging.
- Rate limiting.
- Access policy.

See [examples/nginx](../examples/nginx).

## Upgrade Flow

```mermaid
flowchart LR
  Backup["Back up config"] --> Pull["Pull new binary or image"]
  Pull --> Stop["Stop old process"]
  Stop --> Start["Start new version"]
  Start --> Verify["Verify server and tunnels"]
  Verify --> Record["Record release note"]
```
