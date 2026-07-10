# Getting Started

This guide starts Gate from source on a local machine. It is the shortest path for evaluating Gate v0.9.

## Prerequisites

- Rust 1.88 or newer.
- Node.js 20 or newer.
- npm 10 or newer.
- Git.
- Platform-specific Tauri prerequisites if you want to run the desktop app.

## Clone

```bash
git clone https://github.com/Somirk134/Gate.git
cd Gate
```

## Install client dependencies

```bash
npm --prefix client ci
```

## Verify the workspace

```bash
cargo check --workspace
cargo test --workspace
npm run typecheck
npm run build
```

## Start the server

```bash
npm run dev:server
```

By default, the source server listens on `127.0.0.1:7000` and uses the local development token `gate-alpha-token`.

For a production-like local run, set an explicit bind address and token:

```bash
GATE_SERVER_ADDR=0.0.0.0:7000 \
GATE_AUTH_TOKEN=replace-with-a-long-random-token \
cargo run -p gate-server --release
```

## Start the desktop client

Open another terminal:

```bash
npm --prefix client run tauri dev
```

For frontend-only development:

```bash
npm --prefix client run dev
```

## Create a first tunnel

1. Start a local service, for example an app on `127.0.0.1:3000`.
2. Open the Gate desktop client.
3. Add the local Gate server `127.0.0.1:7000`.
4. Use the development token `gate-alpha-token` only for local testing.
5. Create a TCP tunnel that maps local port `3000` to a remote port such as `18080`.
6. Start the tunnel and check the Dashboard and Log Center.

## Next steps

- [Installation](./installation.md)
- [Deployment](./deployment.md)
- [Docker](./docker.md)
- [Configuration](./configuration.md)
- [Troubleshooting](./troubleshooting.md)
