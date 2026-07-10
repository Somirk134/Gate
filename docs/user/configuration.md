# Configuration

Gate v0.9 keeps runtime configuration small and explicit.

## Server configuration

| Variable | Default | Description |
| --- | --- | --- |
| `GATE_SERVER_ADDR` | `127.0.0.1:7000` in source server bootstrap | Server bind address. Docker sets `0.0.0.0:5800`. |
| `GATE_AUTH_TOKEN` | None; required | Shared token used by clients. Must contain at least 16 characters and must not be a known weak default. |
| `GATE_TUNNEL_BIND_ADDR` | `0.0.0.0` in Docker | Optional bind address for dynamic tunnel listeners. Use `0.0.0.0` for public server deployments. |
| `GATE_HEARTBEAT_TIMEOUT_MS` | Runtime default | Optional heartbeat timeout override. |
| `GATE_CERT_DIR` | Deployment-specific | Certificate storage directory. |
| `GATE_CERTIFICATE_STORE` | Deployment-specific | Optional certificate store selector/path. |
| `GATE_SERVER_DOMAIN_DB` | Deployment-specific | Optional domain metadata database path. |

## ACME and certificates

Gate includes certificate management support. Enable ACME only when the deployment has a public domain and HTTP-01 challenge routing prepared.

| Variable | Purpose |
| --- | --- |
| `GATE_ACME_EMAIL` | Required email for ACME account creation. |
| `GATE_ACME_AUTO` | Enables automatic certificate issuance where supported. |
| `GATE_ACME_DIRECTORY_URL` | Optional custom ACME directory. |
| `GATE_ACME_STAGING` | Use staging CA when set for testing. |
| `GATE_ACME_HTTP01_PORT` | Optional HTTP-01 challenge port. |

## Local tunnel example

```toml
[server]
address = "127.0.0.1:7000"

[tunnel]
name = "local-web"
protocol = "tcp"
local_host = "127.0.0.1"
local_port = 3000
remote_port = 18080
```

Set `GATE_AUTH_TOKEN` in the process environment; do not store it in the configuration file.
