# Gate Documentation

Welcome to the Gate documentation. The docs are organized around the path a new user follows: understand the project, install it, run a server, connect a client, create a tunnel, operate it, and contribute back.

## Start Here

| Goal | Read |
| --- | --- |
| Try Gate locally | [Quick Start](./quick-start.md) |
| Install dependencies | [Installation](./installation.md) |
| Run the server | [Server](./server.md) |
| Launch the desktop client | [Client](./client.md) |
| Create the first tunnel | [Tunnel](./tunnel.md) |

## Product Guides

| Area | Guide |
| --- | --- |
| Projects | [Project](./project.md) |
| Dashboard | [Dashboard](./dashboard.md) |
| Logs | [Log Center](./log-center.md) |
| Settings | [Settings](./settings.md) |
| Authentication | [Authentication](./authentication.md) |

## Operations

| Area | Guide |
| --- | --- |
| Deployment | [Deployment](./deployment.md) |
| Docker | [Docker](./docker.md) |
| Upgrade | [Upgrade](./upgrade.md) |
| Troubleshooting | [Troubleshooting](./troubleshooting.md) |
| FAQ | [FAQ](./faq.md) |

## Maintainers

| Area | Guide |
| --- | --- |
| Architecture | [Architecture](./architecture.md) |
| Developer Guide | [Developer Guide](./developer-guide.md) |
| Contribution | [Contribution](./contribution.md) |
| Release Note | [Release Note](./release-note.md) |

## Reference Areas

The repository also contains deeper implementation notes that are useful for maintainers:

- `docs/runtime`
- `docs/communication`
- `docs/monitoring`
- `docs/heartbeat-reconnect`
- `docs/tunnel-engine`
- `docs/ADR`

These references may keep their historical filenames while the public documentation keeps lowercase kebab-case names.

## Documentation Style

- Use one H1 per page.
- Keep titles short and descriptive.
- Prefer numbered steps for procedures.
- Use fenced code blocks with a language label.
- Use lowercase kebab-case filenames.
- Avoid secrets, private hostnames, and environment-specific values in examples.
- Document alpha limitations explicitly.
