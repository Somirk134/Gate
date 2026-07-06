# Redis

## Description

Expose a local Redis instance for temporary development or QA access.

## Configuration

```toml
[server]
address = "gate.example.com:7000"
auth_token = "replace-me"

[tunnel]
name = "redis-dev"
protocol = "tcp"
local_host = "127.0.0.1"
local_port = 6379
remote_port = 16379
```

Client command:

```bash
redis-cli -h gate.example.com -p 16379
```

## Screenshot

![Log Center screenshot](../../assets/screenshots/log-center.svg)

## Run Steps

1. Start Redis locally.
2. Ensure Redis is not exposed without authentication in shared environments.
3. Start Gate server.
4. Create the `redis-dev` tunnel.
5. Test with `redis-cli`.
6. Stop the tunnel when finished.
