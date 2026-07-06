# Installation

Gate can be used from source today. Native installers and signed releases are part of the pre-1.0 packaging roadmap.

## Requirements

| Tool | Version | Notes |
| --- | --- | --- |
| Rust | 1.78+ | Workspace builds and server runtime |
| Node.js | 20+ | Desktop frontend development |
| npm | 10+ | Client dependency installation |
| Tauri prerequisites | Platform-specific | Needed for desktop app development |
| Docker | 24+ | Optional server container workflow |

## Source Install

```bash
git clone https://gitee.com/lancemorii-git/gate.git
cd gate
cargo build --workspace --release
```

The server binary is created at:

```text
target/release/gate-server
```

## Install Server Locally

```bash
cargo install --path server
```

Then run:

```bash
GATE_SERVER_ADDR=127.0.0.1:7000 \
GATE_AUTH_TOKEN=gate-alpha-token \
gate-server
```

## Desktop Client

```bash
cd client
npm install
npm run tauri dev
```

Use `npm run dev` for frontend-only development.

## Docker

```bash
docker build -f docker/Dockerfile.server -t gate-server:local .
docker run --rm -p 7000:7000 \
  -e GATE_SERVER_ADDR=0.0.0.0:7000 \
  -e GATE_AUTH_TOKEN=replace-me \
  gate-server:local
```

## Verify

```bash
cargo test --workspace
```

For client checks:

```bash
cd client
npm run typecheck
npm run build
```

## Release Packages

Release artifacts should eventually include:

- Server binaries for Linux, macOS, and Windows.
- Desktop installers for Windows, macOS, and Linux.
- Docker images.
- Checksums and signed release notes.

Track packaging progress in [Release Note](./release-note.md) and [Roadmap](../ROADMAP.md).
