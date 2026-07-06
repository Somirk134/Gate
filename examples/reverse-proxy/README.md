# Reverse Proxy Example

Use a reverse proxy for TLS termination, host routing, and edge timeouts.

## Nginx

See [nginx.conf](./nginx.conf).

## Notes

- Preserve upgrade headers for WebSocket traffic.
- Align proxy idle timeouts with heartbeat settings.
- Terminate TLS at the proxy or enable native TLS when available.
