# Gate Documentation

Gate documentation is organized for operators, contributors, maintainers, and future plugin authors.

## Start Here

- [Quick Start](./quick-start.md)
- [Install](./install.md)
- [Configuration](./configuration.md)
- [Troubleshooting](./troubleshooting.md)

## Operator Guides

- [Tunnel](./tunnel.md)
- [Project](./project.md)
- [Authentication](./authentication.md)
- [Heartbeat](./heartbeat.md)
- [Monitoring](./monitoring.md)
- [Deployment](./deployment.md)
- [Docker](./docker.md)

## Maintainer Guides

- [Architecture](./architecture.md)
- [Development Guide](./development-guide.md)
- [Plugin Guide](./plugin-guide.md)
- [API](./api.md)
- [Documentation Style](./documentation-style.md)
- [Naming Conventions](./naming-conventions.md)
- [Diagrams](./diagrams.md)

## Existing Deep Dives

Existing module deep dives remain available under:

- `docs/runtime`
- `docs/communication`
- `docs/monitoring`
- `docs/heartbeat-reconnect`
- `docs/tunnel-engine`
- `docs/ADR`

## Naming Rules

- Public documentation files use lowercase kebab-case.
- Existing implementation notes may keep their historical names until migrated.
- Images and diagrams should live in `assets` or `design`.
- Mermaid is the default diagram format. PlantUML is reserved for future architecture packs.
