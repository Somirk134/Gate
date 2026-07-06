# Docker Example

Start Gate server with Docker Compose.

```bash
docker compose -f docker/docker-compose.yml up -d
docker compose -f docker/docker-compose.yml logs -f
```

## Environment

Copy `examples/docker/gate.env.example` to your deployment environment and replace secrets before
production use.
