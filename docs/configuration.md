# Configuration

Gate uses environment variables for deployable services and local files for desktop preferences.

## Server Environment

| Variable | Default | Description |
| --- | --- | --- |
| `GATE_ENV` | `development` | Runtime profile |
| `GATE_BIND` | `127.0.0.1:5800` | Server bind address |
| `GATE_LOG` | `info` | Log filter |
| `GATE_DATA_DIR` | `./data` | Data directory |
| `GATE_CONFIG` | unset | Optional config file path |
| `GATE_AUTH_REQUIRED` | `true` | Require authenticated sessions |
| `GATE_HEARTBEAT_INTERVAL` | `30s` | Client heartbeat interval |
| `GATE_HEARTBEAT_TIMEOUT` | `90s` | Heartbeat timeout window |

## Example

```bash
GATE_ENV=production \
GATE_BIND=0.0.0.0:5800 \
GATE_LOG=info \
GATE_DATA_DIR=/var/lib/gate \
cargo run -p gate-server --release
```

## Configuration Principles

- Prefer environment variables in containers.
- Keep secrets outside the repository.
- Use explicit production values rather than relying on development defaults.
- Document every new public configuration option in this file.
