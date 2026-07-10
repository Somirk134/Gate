# Installation

Gate v0.9 can be installed from source, release artifacts, or Docker images.

## Requirements

| Component | Version | Required for |
| --- | --- | --- |
| Rust | 1.88+ | Server, Rust crates, Tauri backend |
| Node.js | 20+ | Desktop frontend build |
| npm | 10+ | Client dependency installation |
| Docker | 24+ | Containerized server deployment |
| Tauri prerequisites | Platform-specific | Desktop package builds |

## Source build

```bash
git clone https://github.com/Somirk134/Gate.git
cd Gate
cargo build --workspace --release
```

The server binary is produced at:

```text
target/release/gate-server
```

## Install the server locally

```bash
cargo install --path server
```

Run it with explicit configuration:

```bash
export GATE_AUTH_TOKEN="$(openssl rand -hex 32)"
GATE_SERVER_ADDR=127.0.0.1:7000 \
gate-server
```

## Desktop client from source

```bash
npm --prefix client ci
npm --prefix client run tauri dev
```

To create a desktop package locally:

```bash
npm --prefix client run tauri build
```

## Release packages

Gate v0.9 release automation is prepared to publish:

- Server archives for Linux, Windows, and macOS.
- Windows desktop installer.
- macOS `.dmg` packages for Intel and Apple Silicon.
- Linux `.AppImage` and `.deb` packages.
- Docker image for the server.

See [Release](../development/release.md) for maintainer release steps.
