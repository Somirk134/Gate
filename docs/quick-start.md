# Quick Start

This guide gets Gate running locally: verify the Rust workspace, start the alpha server, and launch the desktop client.

## Prerequisites

- Rust 1.88 or newer.
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

For local testing, start the server from the repository root:

```bash
npm run dev:server
```

This uses the server defaults:

- address: `127.0.0.1:7000`
- token: `gate-alpha-token`

On Windows, the helper script prints those values before starting:

```powershell
npm run dev:server:local
```

If you prefer Cargo directly, this is equivalent for local testing:

```bash
cargo run -p gate-server
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
