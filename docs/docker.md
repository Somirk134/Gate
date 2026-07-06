# Docker

Docker assets live in the top-level `docker` directory.

## Compose

```bash
docker compose -f docker/docker-compose.yml up -d
```

## Build

```bash
docker build -f docker/Dockerfile.server -t gate-server:local .
```

## Run

```bash
docker run --rm \
  -p 5800:5800 \
  -e GATE_ENV=production \
  -e GATE_BIND=0.0.0.0:5800 \
  -v gate-data:/var/lib/gate \
  gate-server:local
```

## Image Policy

- Keep images minimal.
- Do not bake secrets into images.
- Pin base image versions when release builds become stable.
- Scan images in CI before publishing.
