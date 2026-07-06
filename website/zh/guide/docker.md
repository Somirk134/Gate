# Docker

```bash
docker build -f docker/Dockerfile.server -t gate-server:local .
docker run --rm -p 5800:5800 gate-server:local
```

自托管模板见 `docker/docker-compose.yml`。
