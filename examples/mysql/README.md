# MySQL

## Description

Expose a local MySQL instance for controlled development access.

## Configuration

```toml
[server]
address = "gate.example.com:7000"
auth_token = "replace-me"

[tunnel]
name = "mysql-dev"
protocol = "tcp"
local_host = "127.0.0.1"
local_port = 3306
remote_port = 13306
```

Client command:

```bash
mysql -h gate.example.com -P 13306 -u app_user -p
```

## Screenshot

![Tunnel screenshot](../../assets/screenshots/tunnel.svg)

## Run Steps

1. Start MySQL locally.
2. Create a database user with the least required privileges.
3. Start Gate server.
4. Create the `mysql-dev` tunnel.
5. Connect through the remote port.
6. Stop the tunnel after debugging.
