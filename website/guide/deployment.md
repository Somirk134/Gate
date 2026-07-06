# Deployment

```bash
docker compose -f docker/docker-compose.yml up -d
```

## Checklist

- Configure production bind address intentionally.
- Put TLS at a reverse proxy or native runtime layer.
- Store secrets outside images and git.
- Validate heartbeat under proxy timeouts.
