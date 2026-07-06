# Developer Guide

This guide helps contributors work on Gate without changing unrelated runtime behavior.

## Setup

```bash
git clone https://gitee.com/lancemorii-git/gate.git
cd gate
cargo test --workspace
```

Client setup:

```bash
cd client
npm install
npm run typecheck
```

## Common Commands

| Task | Command |
| --- | --- |
| Format Rust | `cargo fmt --all` |
| Check Rust | `cargo check --workspace` |
| Test Rust | `cargo test --workspace` |
| Build server | `cargo build -p gate-server --release` |
| Run server | `cargo run -p gate-server` |
| Typecheck client | `cd client && npm run typecheck` |
| Build client | `cd client && npm run build` |
| Run desktop | `cd client && npm run tauri dev` |

## Contribution Boundaries

- Runtime and protocol changes need tests and architecture notes.
- UI changes need screenshots when visible behavior changes.
- Documentation changes should update links and examples together.
- Configuration changes must update [Configuration](./configuration.md), [Server](./server.md), and Docker docs.

## Branches

Use descriptive branch names:

```text
codex/docs-open-source-polish
feature/server-health-check
fix/auth-token-redaction
```

## Pull Request Checklist

- Scope is clear.
- Tests or verification steps are listed.
- Docs are updated.
- Screenshots are added for UI changes.
- Breaking changes are called out.

## Architecture Reading Order

1. [Architecture](./architecture.md)
2. `docs/runtime`
3. `docs/communication`
4. `docs/tunnel-engine`
5. `docs/ADR`
