# Docker

Gate includes a server Dockerfile and Compose templates for self-hosted deployments.

Before starting the server, generate a deployment token and export it in the shell that runs Docker:

```bash
export GATE_AUTH_TOKEN="$(openssl rand -hex 32)"
```

## Recommended Linux deployment: host network

Gate creates public tunnel listeners dynamically from each tunnel `remotePort`. On Linux servers, the recommended Docker mode is `network_mode: host` so users only need to open the selected ports in the cloud security group or host firewall. No Compose port mapping changes are required when a new tunnel port is used.

From the repository root:

```bash
docker compose up -d --build
```

Or use the copy under `docker/`:

```bash
docker compose -f docker/docker-compose.yml up -d --build
```

Open these ports on the server firewall/security group:

- `5800/tcp` for the Gate desktop client connection.
- Any tunnel `remotePort` used by users, for example `18080/tcp`.

`network_mode: host` is intended for Linux servers. For Docker Desktop on Windows/macOS or environments where host networking is not desired, use the bridge template below.

## Run from a published image

After the image is published to Docker Hub, users do not need source code. They can use the release template:

```bash
docker compose -f docker/docker-compose.release.yml up -d
```

The default image is `qwe1235/gate-server:0.9.1`. Override it with `GATE_SERVER_IMAGE` if a different registry, repository, or tag is required.

## Bridge network fallback

The bridge template keeps Docker port mappings explicit:

```bash
GATE_PORT=5800 \
GATE_TUNNEL_PORT_RANGE=18080-18100 \
  docker compose -f docker/docker-compose.bridge.yml up -d --build
```

In bridge mode, every tunnel port must be included in the mapped range or added to the Compose file before it can be reached from outside the host.

## Build locally

```bash
docker build -f docker/Dockerfile.server -t gate-server:local .
```

## Run a container directly

For Linux host networking:

```bash
docker run --rm --network host \
  -e GATE_SERVER_ADDR=0.0.0.0:5800 \
  -e GATE_AUTH_TOKEN="$GATE_AUTH_TOKEN" \
  gate-server:local
```

For bridge networking:

```bash
docker run --rm -p 5800:5800 -p 18080-18100:18080-18100 \
  -e GATE_SERVER_ADDR=0.0.0.0:5800 \
  -e GATE_AUTH_TOKEN="$GATE_AUTH_TOKEN" \
  gate-server:local
```

## Health check

The image runs:

```bash
gate-server --healthcheck
```

Use the health check result with your orchestrator before routing traffic to the container.
