# Docker

Docker assets live in [docker](../docker).

## Build

```bash
docker build -f docker/Dockerfile.server -t gate-server:local .
```

## Run

```bash
docker run --rm \
  -p 7000:7000 \
  -e GATE_SERVER_ADDR=0.0.0.0:7000 \
  -e GATE_AUTH_TOKEN=replace-me \
  gate-server:local
```

## Compose

```bash
docker compose -f docker/docker-compose.yml up -d
```

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
| `GATE_SERVER_ADDR` | Yes | Use `0.0.0.0:7000` inside the container |
| `GATE_AUTH_TOKEN` | Yes | Shared client/server token |

## Image Policy

- Do not bake secrets into images.
- Keep images minimal.
- Prefer explicit tags for published releases.
- Publish checksums and release notes with official images.
- Scan images in CI before publishing.
