# Quick Start

This guide verifies the workspace, starts the server, and launches the desktop client.

## Prerequisites

- Rust 1.78 or newer.
- Node.js 20 or newer.
- npm 10 or newer.
- Platform-specific Tauri prerequisites.

## Clone

```bash
git clone https://github.com/lancemorii-git/gate.git
cd gate
```

## Verify Rust Workspace

```bash
cargo fmt --all --check
cargo test --workspace
```

## Start Server

```bash
cargo run -p gate-server
```

Default development bind address:

```text
127.0.0.1:5800
```

## Start Desktop Client

```bash
cd client
npm install
npm run tauri dev
```

## Next Steps

- Configure a tunnel with [Tunnel](./tunnel.md).
- Review runtime health with [Monitoring](./monitoring.md).
- Deploy with [Docker](./docker.md) or [Deployment](./deployment.md).
