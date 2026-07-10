# Docker

Gate includes a server Dockerfile and Compose templates for self-hosted deployments.

## Build locally

```bash
docker build -f docker/Dockerfile.server -t gate-server:local .
```

## Run a container

```bash
docker run --rm -p 5800:5800 \
  -e GATE_SERVER_ADDR=0.0.0.0:5800 \
  -e GATE_AUTH_TOKEN=replace-with-a-long-random-token \
  gate-server:local
```

## Docker Compose

From the repository root:

```bash
GATE_AUTH_TOKEN=replace-with-a-long-random-token GATE_PORT=5800 \
docker compose up -d
```

Or use the copy under `docker/`:

```bash
GATE_AUTH_TOKEN=replace-with-a-long-random-token GATE_PORT=5800 \
docker compose -f docker/docker-compose.yml up -d
```

The Compose template maps `${GATE_PORT:-5800}` on the host to container port `5800` and stores data in named volumes.

## Health check

The image runs:

```bash
gate-server --healthcheck
```

Use the health check result with your orchestrator before routing traffic to the container.
