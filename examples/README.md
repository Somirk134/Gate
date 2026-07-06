# Gate Examples

Examples are intentionally small and configuration-focused. They document common deployment shapes
without adding business-specific code.

| Example | Directory | Purpose |
| --- | --- | --- |
| Basic Example | [basic](./basic) | One local service exposed through Gate |
| Multi Tunnel | [multi-tunnel](./multi-tunnel) | Multiple tunnel definitions in one project |
| Docker Example | [docker](./docker) | Compose-based server startup |
| Self-hosted Example | [self-hosted](./self-hosted) | VPS or private server deployment notes |
| Reverse Proxy Example | [reverse-proxy](./reverse-proxy) | TLS termination and proxy forwarding |

## Naming

Example directories use lowercase kebab-case. Config files should be copyable and should not include
real secrets.
