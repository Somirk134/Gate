# Gate Examples

Examples are small, copyable recipes for common tunnel shapes. They are documentation-first and do not add business runtime code.

## Standard Examples

| Example | Directory | Use case |
| --- | --- | --- |
| Basic TCP | [basic-tcp](./basic-tcp) | Expose a local TCP service |
| Webhook | [webhook](./webhook) | Receive public callbacks locally |
| SSH | [ssh](./ssh) | Reach a development machine through a controlled tunnel |
| MySQL | [mysql](./mysql) | Connect to a local database from a remote client |
| Redis | [redis](./redis) | Expose Redis for controlled development access |
| Docker | [docker](./docker) | Run Gate server with Docker or Compose |
| Nginx | [nginx](./nginx) | Expose a local Nginx HTTP service |
| Spring Boot | [spring-boot](./spring-boot) | Share a local Java service |
| Express | [express](./express) | Share a local Express API |
| Node.js | [node-js](./node-js) | Share a local Node service |
| Python Flask | [python-flask](./python-flask) | Share a local Flask app |
| Go Gin | [go-gin](./go-gin) | Share a local Go API |

## Legacy Examples

Older examples remain available while links are migrated:

- [basic](./basic)
- [multi-tunnel](./multi-tunnel)
- [reverse-proxy](./reverse-proxy)
- [self-hosted](./self-hosted)

## Conventions

- Directory names use lowercase kebab-case.
- Config values use `127.0.0.1`, `gate.example.com`, and fake ports.
- Never commit real tokens.
- Each example includes description, configuration, screenshot, and run steps.
