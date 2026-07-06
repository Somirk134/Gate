# Self-hosted Example

This example outlines a small VPS deployment.

## Recommended Layout

```text
/opt/gate
  docker-compose.yml
  gate.env
/var/lib/gate
  data
```

## Steps

1. Create a non-root user for the service.
2. Copy `docker/docker-compose.yml` to `/opt/gate/docker-compose.yml`.
3. Create `/opt/gate/gate.env` from `examples/docker/gate.env.example`.
4. Start with `docker compose --env-file gate.env up -d`.
5. Put TLS and public routing behind a reverse proxy.

## Maintenance

- Record each deployed version.
- Keep backups of configuration and data.
- Review `CHANGELOG.md` before upgrades.
