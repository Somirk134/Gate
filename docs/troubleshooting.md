# Troubleshooting

Use this page to quickly isolate common Gate failures.

## Server Does Not Start

- Run `cargo run -p gate-server` locally and inspect logs.
- Check whether the configured bind port is already in use.
- Confirm required environment variables are present.

## Client Cannot Connect

- Confirm server address and port.
- Check authentication settings.
- Test network reachability with `curl` or platform tools.
- Review reverse proxy timeout and WebSocket settings.

## Tunnel Is Unhealthy

- Check heartbeat status.
- Confirm local service is reachable from the client machine.
- Review tunnel protocol and port configuration.
- Check server logs for rejected registration or forwarding errors.

## Docker Container Exits

- Run `docker logs <container>`.
- Verify bind address uses `0.0.0.0` inside the container.
- Mount a writable data directory.

## Useful Commands

```bash
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
npm --prefix client run typecheck
docker compose -f docker/docker-compose.yml logs -f
```
