# Configuration

Gate alpha uses environment variables for the server and local application state for the desktop client.

## Server Environment

| Variable | Default | Description |
| --- | --- | --- |
| `GATE_SERVER_ADDR` | `127.0.0.1:7000` | TCP bind address |
| `GATE_AUTH_TOKEN` | `gate-alpha-token` | Shared client/server token |

Older drafts mention variables such as `GATE_BIND`, `GATE_ENV`, or `GATE_DATA_DIR`. Those are reserved deployment concepts and should not be documented as active runtime behavior until the server reads them.

## Local Server Example

```bash
GATE_SERVER_ADDR=127.0.0.1:7000 \
GATE_AUTH_TOKEN=gate-alpha-token \
cargo run -p gate-server
```

## Public Server Example

```bash
GATE_SERVER_ADDR=0.0.0.0:7000 \
GATE_AUTH_TOKEN=replace-with-a-long-random-token \
./target/release/gate-server
```

## Tunnel Template

```toml
[server]
address = "127.0.0.1:7000"
auth_token = "gate-alpha-token"

[tunnel]
name = "local-web"
protocol = "tcp"
local_host = "127.0.0.1"
local_port = 3000
remote_port = 18080
auto_start = false
```

## Configuration Principles

- Keep secrets out of git.
- Prefer environment variables for deployable services.
- Prefer local encrypted or OS-managed storage for desktop tokens.
- Update docs and examples whenever public config changes.
- Make alpha placeholders explicit.
