# Authentication

Gate alpha authentication uses a shared token between client and server. This is intentionally simple while the project stabilizes runtime behavior.

## Server Token

```bash
GATE_AUTH_TOKEN=replace-with-a-long-random-token
```

If unset, the alpha server defaults to:

```text
gate-alpha-token
```

Do not use the default token for a server reachable by other people.

## Client Setup

When adding a server in the desktop client:

1. Enter the server address, for example `127.0.0.1:7000`.
2. Enter the token.
3. Save the server.
4. Verify the connection.

## Security Guidance

- Generate a long random token for shared environments.
- Rotate tokens after demos, incidents, or accidental exposure.
- Store tokens outside scripts and examples.
- Redact tokens in logs, screenshots, and issues.
- Use firewall rules to restrict server access.

## Future Direction

The authentication roadmap may include:

- Token rotation workflow.
- Multiple client credentials.
- Scoped tokens.
- Audit events.
- Optional external identity integration.

Document compatibility and migration steps before changing the public auth model.
