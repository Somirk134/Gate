# Docker

This directory contains Docker assets for the Gate server.

## Files

| File | Purpose |
| --- | --- |
| `Dockerfile.server` | Multi-stage server image |
| `docker-compose.yml` | Recommended Linux host-network deployment template |
| `docker-compose.bridge.yml` | Bridge-network fallback for Docker Desktop or fixed port ranges |
| `docker-compose.release.yml` | Host-network template for users pulling a published image |

## Recommended run mode

Gate listens on tunnel ports dynamically. On Linux servers, use host networking so operators only need to open the Gate control port and the selected tunnel ports in the firewall/security group.

```bash
export GATE_AUTH_TOKEN="$(openssl rand -hex 32)"
docker compose -f docker/docker-compose.yml up -d --build
```

Required firewall/security-group ports:

- `5800/tcp` for desktop client connections.
- Any tunnel `remotePort` selected by users.


## Published image template

After publishing the image to Docker Hub, users can run without source code:

```bash
export GATE_AUTH_TOKEN="$(openssl rand -hex 32)"
docker compose -f docker/docker-compose.release.yml up -d
```

The default image is `qwe1235/gate-server:0.9.1`. Override it with `GATE_SERVER_IMAGE` if needed.

## Bridge fallback

```bash
export GATE_AUTH_TOKEN="$(openssl rand -hex 32)"
GATE_PORT=5800 \
GATE_TUNNEL_PORT_RANGE=18080-18100 \
  docker compose -f docker/docker-compose.bridge.yml up -d --build
```

In bridge mode, mapped tunnel ports must be declared before use.

## Build

```bash
docker build -f docker/Dockerfile.server -t gate-server:local .
```

## Production Notes

- Generate a strong `GATE_AUTH_TOKEN` before starting the container.
- Store secrets outside Compose files.
- Use Linux host networking when dynamic tunnel ports are required.
- Put TLS at a reverse proxy or native TLS endpoint.
- Pin image tags when official releases are published.
