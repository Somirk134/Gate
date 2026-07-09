# Docker

This directory contains Docker assets for the Gate server.

## Files

| File | Purpose |
| --- | --- |
| `Dockerfile.server` | Multi-stage server image |
| `docker-compose.yml` | Local self-hosted deployment template |

## Run

```bash
GATE_AUTH_TOKEN=replace-me GATE_PORT=5800 \
docker compose -f docker/docker-compose.yml up -d
```

The Compose template maps `${GATE_PORT:-5800}` on the host to container port `5800`.

## Build

```bash
docker build -f docker/Dockerfile.server -t gate-server:local .
```

## Production Notes

- Replace development defaults before production use.
- Store secrets outside Compose files.
- Put TLS at a reverse proxy or native TLS endpoint.
- Pin image tags when official releases are published.
