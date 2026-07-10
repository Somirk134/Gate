# Deployment

This page covers production-style server deployment. Keep secrets out of source control and rotate any token that was ever shared in logs or screenshots.

## Deployment options

| Option | Status | Best for |
| --- | --- | --- |
| Source run | Supported | Development and small internal trials |
| Release binary | Supported by release workflow | VPS or single-host deployment |
| Docker Compose | Supported template | Repeatable self-hosted deployment |
| Reverse proxy | Supported pattern | Public domain and TLS termination |

## Environment variables

| Variable | Purpose | Example |
| --- | --- | --- |
| `GATE_SERVER_ADDR` | Server bind address | `0.0.0.0:7000` |
| `GATE_AUTH_TOKEN` | Client authentication token | `replace-with-a-long-random-token` |
| `GATE_TUNNEL_BIND_ADDR` | Optional tunnel bind address | `0.0.0.0` |
| `GATE_CERT_DIR` | Certificate storage directory | `/var/lib/gate/certificates` |
| `GATE_ACME_EMAIL` | ACME account email when ACME is enabled | `ops@example.com` |
| `GATE_ACME_AUTO` | Enable automatic ACME issuance where supported | `true` |

## Binary deployment

```bash
cargo build -p gate-server --release
GATE_SERVER_ADDR=0.0.0.0:7000 \
GATE_AUTH_TOKEN=replace-with-a-long-random-token \
./target/release/gate-server
```

Recommended hardening:

- Use a unique token for each environment.
- Restrict inbound ports with a firewall.
- Put TLS at a reverse proxy when exposing public HTTP entrypoints.
- Capture logs through your service manager.
- Back up configuration and certificate storage.

## Reverse proxy pattern

Use a reverse proxy such as Nginx, Caddy, or Traefik to terminate public TLS and route traffic to Gate server ports. Keep Gate server tokens private and avoid exposing development defaults.

## Upgrade checklist

1. Read the release notes and breaking-change section.
2. Back up configuration and certificate directories.
3. Stop the old server process.
4. Replace the binary or Docker image.
5. Start the server with the same environment variables.
6. Verify client login, tunnel creation, and logs.
