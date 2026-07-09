# Docker

Docker assets live in [docker](../docker).

## Build

```bash
docker build -f docker/Dockerfile.server -t gate-server:local .
```

## Run

```bash
docker run --rm \
  -p 5800:5800 \
  -e GATE_SERVER_ADDR=0.0.0.0:5800 \
  -e GATE_AUTH_TOKEN=replace-me \
  gate-server:local
```

## Compose

```bash
GATE_AUTH_TOKEN=replace-me GATE_PORT=5800 \
docker compose -f docker/docker-compose.yml up -d
```

The Docker image and Compose template listen on container port `5800`. Set the desktop client server port to the host port exposed by `GATE_PORT`; the default is `5800`.

View logs:

```bash
docker compose -f docker/docker-compose.yml logs -f gate-server
```

Stop:

```bash
docker compose -f docker/docker-compose.yml down
```

## Environment

| Variable | Required | Description |
| --- | --- | --- |
| `GATE_SERVER_ADDR` | Yes | Use `0.0.0.0:5800` inside the container |
| `GATE_AUTH_TOKEN` | Yes | Shared client/server token |
| `GATE_PORT` | Compose only | Host port mapped to container port `5800`; defaults to `5800` |

## Image Policy

- Do not bake secrets into images.
- Keep images minimal.
- Prefer explicit tags for published releases.
- Publish checksums and release notes with official images.
- Scan images in CI before publishing.
