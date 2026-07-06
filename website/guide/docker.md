# Docker

```bash
docker build -f docker/Dockerfile.server -t gate-server:local .
docker run --rm -p 5800:5800 gate-server:local
```

See `docker/docker-compose.yml` for a self-hosted template.
