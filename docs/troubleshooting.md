# Troubleshooting

Use this guide when Gate does not start, connect, authenticate, or forward traffic as expected.

## Server Does Not Start

Check whether the port is already in use:

```bash
netstat -ano | findstr :7000
```

For local source development, the default server starts with:

```bash
npm run dev:server
```

On Windows, try a different bind address with:

```powershell
npm run dev:server:local -- -Addr "127.0.0.1:7001"
```

## Client Cannot Connect

Verify:

- Server is running.
- Address uses the correct host and port.
- Firewall allows the port.
- Token matches `GATE_AUTH_TOKEN`.
- Docker container exposes port `7000`.

## Authentication Fails

Symptoms:

- Client disconnects after login.
- Server logs mention authentication failure.

Fix:

1. Stop the client.
2. Confirm `GATE_AUTH_TOKEN` on the server.
3. Update the saved server token in the client.
4. Retry the connection.

## Tunnel Does Not Receive Traffic

Check:

- Local service is running.
- Local host and port are correct.
- Remote port is not occupied.
- Tunnel status is `running`.
- Log Center has no `error` or `disconnected` event.

## Docker Container Starts But Is Not Reachable

Inside the container, use:

```bash
GATE_SERVER_ADDR=0.0.0.0:7000
```

Expose the same port:

```bash
-p 7000:7000
```

## Useful Diagnostics

```bash
cargo test --workspace
npm run dev:server
docker compose -f docker/docker-compose.yml logs -f gate-server
```

When opening an issue, include:

- Operating system.
- Gate version or commit SHA.
- Startup command.
- Redacted environment variables.
- Redacted logs.
- Steps to reproduce.
