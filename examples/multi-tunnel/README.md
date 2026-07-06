# Multi Tunnel Example

Use this example when one project exposes several local services.

## Configuration

```toml
[[tunnels]]
name = "web"
protocol = "tcp"
local = "127.0.0.1:3000"
remote = "0.0.0.0:8080"

[[tunnels]]
name = "api"
protocol = "tcp"
local = "127.0.0.1:4000"
remote = "0.0.0.0:8081"
```

## Notes

- Keep tunnel names stable.
- Use separate remote ports for each tunnel.
- Document project ownership before sharing access.
