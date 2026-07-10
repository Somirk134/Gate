# Troubleshooting

## Server does not start

Check the bind address and whether another process is using the same port:

```bash
GATE_SERVER_ADDR=127.0.0.1:7000 cargo run -p gate-server
```

If the port is busy, choose another address or stop the conflicting process.

## Client cannot authenticate

- Confirm the server is reachable from the client machine.
- Confirm `GATE_AUTH_TOKEN` matches the token configured in the desktop client.
- Replace the default token before sharing a server.

## Docker container is unhealthy

Inspect logs and the health check:

```bash
docker compose logs gate-server
docker compose ps
```

Confirm the container listens on `0.0.0.0:5800` and the host port is mapped correctly.

## Desktop build fails on Linux

Install Tauri Linux prerequisites, including WebKitGTK, AppIndicator, librsvg, and build tools. The release workflow documents the packages used by CI.

## Update checking reports unavailable

Gate v0.9 does not ship a dedicated updater service endpoint. Desktop update checks should fall back to GitHub Releases when available. Installers can be downloaded manually from the release page.

## Before opening an issue

Please include:

- Gate version and commit SHA.
- Operating system and architecture.
- Server startup command or Docker Compose snippet with secrets removed.
- Relevant logs.
- Steps to reproduce.
