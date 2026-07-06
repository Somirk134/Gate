# Upgrade

Gate is pre-1.0, so upgrades should be deliberate and reversible.

## Before Upgrading

1. Read the release note.
2. Check Breaking Changes.
3. Back up configuration and tokens.
4. Record the current binary, image tag, or commit SHA.
5. Stop non-critical tunnels.

## Source Upgrade

```bash
git pull --ff-only
cargo test --workspace
cargo build --workspace --release
```

Restart the server with the same environment variables:

```bash
GATE_SERVER_ADDR=0.0.0.0:7000 \
GATE_AUTH_TOKEN=replace-with-a-long-random-token \
./target/release/gate-server
```

## Docker Upgrade

```bash
docker compose -f docker/docker-compose.yml pull
docker compose -f docker/docker-compose.yml up -d
docker compose -f docker/docker-compose.yml logs -f gate-server
```

For local images:

```bash
docker build -f docker/Dockerfile.server -t gate-server:local .
docker compose -f docker/docker-compose.yml up -d --build
```

## Rollback

1. Stop the new server.
2. Restore the previous binary or image.
3. Restore configuration if it changed.
4. Start the previous version.
5. Verify authentication, heartbeat, and critical tunnels.

## Compatibility Policy

Before 1.0, configuration and UI flows can change. Every release note should include:

- Breaking Changes.
- Known Issues.
- Upgrade Guide.
- Rollback notes when needed.
