# Basic Example

Expose a local web service through a Gate server.

## Local Service

```bash
python -m http.server 3000
```

## Tunnel Configuration

```toml
[tunnel]
name = "basic-web"
protocol = "tcp"
local = "127.0.0.1:3000"
remote = "0.0.0.0:8080"
```

## Run

```bash
cargo run -p gate-server
```

Then start the desktop client and create a tunnel using the values above.
