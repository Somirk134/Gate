<p align="center">
  <a href="https://gitee.com/lancemorii-git/gate">
    <img src="./assets/logo/logo.png" alt="Gate logo" width="132" />
  </a>
</p>

<h1 align="center">Gate</h1>

<p align="center">
  Self-hosted tunnel infrastructure for exposing private TCP and HTTP services through your own server.
</p>

<p align="center">
  <strong>Rust runtime. Tauri desktop client. Docker-ready deployment. Built for teams that want control.</strong>
</p>

<p align="center">
  <a href="./README.zh-CN.md">简体中文</a>
  ·
  <a href="./docs/README.md">Documentation</a>
  ·
  <a href="./examples/README.md">Examples</a>
  ·
  <a href="./ROADMAP.md">Roadmap</a>
  ·
  <a href="./CONTRIBUTING.md">Contributing</a>
</p>

<p align="center">
  <a href="https://www.rust-lang.org"><img alt="Rust" src="https://img.shields.io/badge/Rust-1.78%2B-orange?logo=rust" /></a>
  <a href="./LICENSE"><img alt="License" src="https://img.shields.io/badge/license-MIT-blue" /></a>
  <img alt="Status" src="https://img.shields.io/badge/status-alpha-yellow" />
  <img alt="Platforms" src="https://img.shields.io/badge/platform-Windows%20%7C%20macOS%20%7C%20Linux-0ea5e9" />
  <a href="https://gitee.com/lancemorii-git/gate"><img alt="Gitee" src="https://img.shields.io/badge/repo-Gitee-C71D23?logo=gitee&logoColor=white" /></a>
</p>

<p align="center">
  <img src="./assets/screenshots/hero.png" alt="Gate dashboard preview" />
</p>

## What Is Gate?

Gate is an open source tunneling project for teams that need private services to be reachable from a controlled public entrypoint. It combines a Rust server/runtime foundation, a desktop client, authentication, heartbeat, monitoring views, Docker deployment templates, and a documentation-first open source workflow.

Gate is currently pre-1.0. The repository is being shaped in public, and this README reflects the current alpha boundary: server authentication and runtime foundations are available, while tunnel UX and production hardening are still evolving.

## Why Gate?

| Need | Gate approach |
| --- | --- |
| Keep traffic under your control | Run the entry server on your own VPS, lab machine, or private cloud. |
| Avoid SaaS lock-in | Configuration, deployment, and examples live in the repository. |
| Give teams a visual workflow | Use the desktop client for projects, tunnels, servers, logs, and settings. |
| Build on a systems foundation | Rust, Tokio, typed protocol crates, integration tests, and explicit architecture docs. |
| Make operations repeatable | Docker, release notes, troubleshooting, benchmark templates, and upgrade guides are included. |

## Features

- Self-hosted server runtime with token authentication and heartbeat primitives.
- Desktop client built with Tauri, Vue, TypeScript, Pinia, and Naive UI.
- Project, tunnel, server, dashboard, log center, and settings surfaces in the client UI.
- Rust workspace split into domain, application, infrastructure, protocol, communication, transport, engine, server, shared, and integration crates.
- Docker and Compose templates for local deployment.
- Examples for TCP, webhooks, SSH, databases, reverse proxies, and common app stacks.
- Examples for real callback and remote-access scenarios.
- Documentation, release, benchmark, security, and contribution templates for maintainers.

## Screenshots

| Dashboard | Tunnel Workspace | Log Center |
| --- | --- | --- |
| ![Dashboard](./assets/screenshots/dashboard.png) | ![Tunnel](./assets/screenshots/tunnel.png) | ![Log center](./assets/screenshots/log-center.png) |

Additional visual assets are documented in [branding/screenshot-guidelines.md](./branding/screenshot-guidelines.md).

## Quick Start

```bash
git clone https://gitee.com/lancemorii-git/gate.git
cd gate
cargo test --workspace
```

Start the local alpha server. It listens on `127.0.0.1:7000` and uses the default local token `gate-alpha-token`:

```bash
npm run dev:server
```

On Windows, the helper script prints the address and token before starting:

```powershell
npm run dev:server:local
```

Launch the desktop client:

```bash
cd client
npm install
npm run tauri dev
```

Read the full guide in [docs/quick-start.md](./docs/quick-start.md).

## Installation

| Target | Command |
| --- | --- |
| Build workspace | `cargo build --workspace --release` |
| Run local server from source | `npm run dev:server` or `cargo run -p gate-server` |
| Install server locally | `cargo install --path server` |
| Run desktop web shell | `cd client && npm install && npm run dev` |
| Run desktop app | `cd client && npm install && npm run tauri dev` |

See [docs/installation.md](./docs/installation.md) for platform notes and Tauri prerequisites.

## Deployment

Gate can be deployed from source, as a local binary, or with Docker. For production-like use, run the server on an explicit public bind address and set a non-default token.

```bash
GATE_SERVER_ADDR=0.0.0.0:7000 \
GATE_AUTH_TOKEN=replace-with-a-long-random-token \
./target/release/gate-server
```

Deployment guides:

- [Server](./docs/server.md)
- [Deployment](./docs/deployment.md)
- [Docker](./docs/docker.md)
- [Upgrade](./docs/upgrade.md)
- [Troubleshooting](./docs/troubleshooting.md)

## Docker

```bash
docker build -f docker/Dockerfile.server -t gate-server:local .
docker run --rm -p 7000:7000 \
  -e GATE_SERVER_ADDR=0.0.0.0:7000 \
  -e GATE_AUTH_TOKEN=replace-me \
  gate-server:local
```

Or use Compose:

```bash
docker compose -f docker/docker-compose.yml up -d
```

## Desktop Client

The desktop client is designed for operators who prefer a visual workflow:

- Welcome wizard for first-run setup.
- Server manager for self-hosted endpoints.
- Project workspace for grouping tunnels.
- Tunnel wizard for local-to-remote mappings.
- Dashboard for connection and traffic health.
- Log center for filtering runtime events.

Read [docs/client.md](./docs/client.md), [docs/dashboard.md](./docs/dashboard.md), and [docs/log-center.md](./docs/log-center.md).

## Server

The alpha server currently exposes a TCP protocol endpoint and uses:

- `GATE_SERVER_ADDR` for bind address, default `127.0.0.1:7000`.
- `GATE_AUTH_TOKEN` for token authentication, default `gate-alpha-token`.

Read [docs/server.md](./docs/server.md) and [docs/authentication.md](./docs/authentication.md).

## Configuration

Minimal server configuration:

```bash
GATE_SERVER_ADDR=0.0.0.0:7000
GATE_AUTH_TOKEN=change-this-before-sharing-a-server
```

Example tunnel configuration used by docs and examples:

```toml
[server]
address = "127.0.0.1:7000"
auth_token = "gate-alpha-token"

[tunnel]
name = "local-web"
protocol = "tcp"
local_host = "127.0.0.1"
local_port = 3000
remote_port = 18080
```

Read [docs/configuration.md](./docs/configuration.md) for the current alpha configuration matrix.

## Create Your First Tunnel

1. Start a local app on `127.0.0.1:3000`.
2. Start the local server with `npm run dev:server`; use token `gate-alpha-token` in the desktop client.
3. Open the desktop client and add your server.
4. Create a TCP tunnel named `local-web`.
5. Set local port `3000` and remote port `18080`.
6. Start the tunnel and verify traffic in Dashboard and Log Center.

Read [docs/tunnel.md](./docs/tunnel.md) and [examples/basic-tcp](./examples/basic-tcp).

## Common Use Cases

- Receive payment callbacks on a local development machine.
- Test GitHub, Gitea, Jenkins, and webhook integrations.
- Reach SSH, MySQL, Redis, and internal tools from a controlled endpoint.
- Expose a local Node.js, Flask, Spring Boot, or Go service for QA.
- Access a home server, NAS, or remote development box.
- Evaluate a self-hosted alternative to managed tunnel services.

See [examples](./examples/README.md).

## Roadmap

The current roadmap focuses on:

- Stabilizing tunnel runtime behavior.
- Completing desktop client flows.
- Hardening Docker and release packaging.
- Publishing benchmark data.
- Improving authentication, upgrade, and backup stories.

Track details in [ROADMAP.md](./ROADMAP.md).

## Documentation

| Area | Link |
| --- | --- |
| Start | [Quick Start](./docs/quick-start.md), [Installation](./docs/installation.md) |
| Operate | [Server](./docs/server.md), [Client](./docs/client.md), [Tunnel](./docs/tunnel.md), [Project](./docs/project.md) |
| Observe | [Dashboard](./docs/dashboard.md), [Log Center](./docs/log-center.md), [Settings](./docs/settings.md) |
| Secure | [Authentication](./docs/authentication.md), [Security](./SECURITY.md) |
| Deploy | [Deployment](./docs/deployment.md), [Docker](./docs/docker.md), [Upgrade](./docs/upgrade.md) |
| Build | [Architecture](./docs/architecture.md), [Developer Guide](./docs/developer-guide.md), [Contribution](./docs/contribution.md) |
| Reference | [FAQ](./docs/faq.md), [Troubleshooting](./docs/troubleshooting.md), [Release Note](./docs/release-note.md) |

## FAQ

**Why choose Gate?**
Choose Gate when you want a self-hosted tunnel foundation with a desktop operator workflow and a Rust codebase you can inspect, deploy, and extend.

**How is Gate different from FRP?**
FRP is a mature tunnel proxy. Gate is a newer project focused on a desktop-first management experience, typed Rust workspace architecture, and GitHub-native project maintainability. Gate is not yet a drop-in replacement for FRP in production.

**Does Gate support Docker?**
Yes. Docker assets live in [docker](./docker), and the guide is in [docs/docker.md](./docs/docker.md).

**Does Gate support HTTPS?**
Use a reverse proxy for TLS termination while native HTTPS support is being stabilized.

**Is Gate free?**
Yes. Gate is open source under the MIT License.

More answers live in [docs/faq.md](./docs/faq.md).

## Contributing

Contributions are welcome, especially documentation, examples, tests, packaging, and issue triage. Start with:

- [CONTRIBUTING.md](./CONTRIBUTING.md)
- [docs/developer-guide.md](./docs/developer-guide.md)
- [docs/contribution.md](./docs/contribution.md)
- [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md)

## License

Gate is released under the [MIT License](./LICENSE).

## Star Gate

If Gate is useful to you, please give the repository a Star. It helps new users discover the project and gives maintainers a clear signal about which open source work matters most.
