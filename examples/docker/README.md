# Docker

## Description

Run the Gate server in Docker for repeatable local or VPS deployment.

## Configuration

```yaml
services:
  gate-server:
    image: gate-server:local
    build:
      context: ../..
      dockerfile: docker/Dockerfile.server
    environment:
      GATE_SERVER_ADDR: 0.0.0.0:5800
      GATE_AUTH_TOKEN: replace-me
    ports:
      - "5800:5800"
```

## Screenshot

![Server screenshot](../../assets/screenshots/server.svg)

## Run Steps

1. Build the image from the repository root.
2. Start Compose.
3. Confirm logs show the server is listening.
4. Connect with the desktop client.
5. Stop Compose when finished.

```bash
docker build -f docker/Dockerfile.server -t gate-server:local .
GATE_AUTH_TOKEN=replace-me GATE_PORT=5800 \
docker compose -f docker/docker-compose.yml up -d
docker compose -f docker/docker-compose.yml logs -f gate-server
```
