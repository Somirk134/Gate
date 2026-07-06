# Project

Projects group related tunnels, servers, tags, and operational context. They help teams avoid a flat list of tunnels as usage grows.

## When To Use Projects

Use a project when tunnels share one of these boundaries:

- Product or application.
- Environment such as development, staging, or production.
- Team ownership.
- Customer or lab environment.
- Temporary incident or debugging workspace.

## Example

```toml
[project]
name = "checkout-dev"
owner = "payments"
environment = "development"

[[tunnels]]
name = "wechat-pay-callback"
protocol = "http"
local_host = "127.0.0.1"
local_port = 3000
remote_port = 18080
```

## Recommended Fields

| Field | Description |
| --- | --- |
| Name | Short product or workflow name |
| Owner | Team, person, or service owner |
| Environment | Development, staging, production, lab, or demo |
| Tags | Domain, protocol, priority, or lifecycle labels |
| Notes | Operational context and safety warnings |

## Project Hygiene

- Keep demo projects separate from production-like work.
- Archive projects when callbacks or debugging sessions are no longer needed.
- Use consistent tags across projects.
- Document external systems that call a project tunnel.

## Related

- [Tunnel](./tunnel.md)
- [Dashboard](./dashboard.md)
- [Settings](./settings.md)
