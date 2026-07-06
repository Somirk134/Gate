# Quick Start

This guide gets Gate running locally: verify the Rust workspace, start the alpha server, and launch the desktop client.

## Prerequisites

- Rust 1.78 or newer.
- Node.js 20 or newer.
- npm 10 or newer.
- Platform-specific Tauri prerequisites.
- Git.

## Clone

```bash
git clone https://gitee.com/lancemorii-git/gate.git
cd gate
```

## Verify The Workspace

```bash
cargo fmt --all --check
cargo test --workspace
```

If you only want to build the main Rust targets:

```bash
cargo build --workspace
```

## Start The Server

The current alpha server reads `GATE_SERVER_ADDR` and `GATE_AUTH_TOKEN`.

```bash
GATE_SERVER_ADDR=127.0.0.1:7000 \
GATE_AUTH_TOKEN=gate-alpha-token \
cargo run -p gate-server
```

PowerShell:

```powershell
$env:GATE_SERVER_ADDR = "127.0.0.1:7000"
$env:GATE_AUTH_TOKEN = "gate-alpha-token"
cargo run -p gate-server
```

Expected bind address:

```text
127.0.0.1:7000
```

## Start The Desktop Client

Open another terminal:

```bash
cd client
npm install
npm run tauri dev
```

For browser-only UI development:

```bash
npm run dev
```

## Create A First Tunnel

Use the desktop client:

1. Open the Welcome Wizard.
2. Add the local server `127.0.0.1:7000`.
3. Use token `gate-alpha-token`.
4. Create a TCP tunnel from local port `3000` to remote port `18080`.
5. Start the tunnel and check Dashboard and Log Center.

The tunnel workflow is still alpha. Use [Tunnel](./tunnel.md) for the current UI and configuration contract.

## Next Steps

- Install for your platform: [Installation](./installation.md)
- Run a server: [Server](./server.md)
- Learn the desktop client: [Client](./client.md)
- Deploy with Docker: [Docker](./docker.md)
- Troubleshoot startup issues: [Troubleshooting](./troubleshooting.md)
