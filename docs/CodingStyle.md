# Gate Rust Coding Style

## Basic principles

- Use stable Rust.
- Keep each crate focused on one responsibility.
- Keep dependency direction aligned with Clean Architecture.
- Do not introduce cross-layer shortcuts.
- Avoid global mutable state. Runtime dependencies should be injected through `AppContext`, registries, providers, or factories.
- Add abstractions only when they express a clear boundary or match an existing local pattern.

## Module convention

- `mod.rs` should declare modules and expose only necessary re-exports.
- Domain modules use a consistent file shape: `service.rs`, `repository.rs`, `entity.rs`, `error.rs`, `event.rs`, `handler.rs`, and `types.rs`.
- Do not place domain entities in `shared`.
- Do not introduce Axum, Tower, SQLx, Redis, or Tokio network types into `domain`.
- Do not put business decisions in `transport`.

## Error convention

- Cross-layer errors should flow through `gate_shared::error`.
- Public boundaries expose `AppError`.
- Specific categories use `ConfigError`, `NetworkError`, `TunnelError`, and `InternalError`.
- New error types must provide stable `ErrorCode` mapping.
- Use `thiserror` to implement the `Error` trait.

## Async convention

- Tokio is the async runtime.
- Keep traits synchronous until an async boundary is justified by object safety, lifetimes, and call boundaries.
- Do not start background tasks, listeners, or long-lived spawned jobs without an explicit lifecycle owner.

## Logging convention

- Use `tracing`.
- Log levels are `Trace`, `Debug`, `Info`, `Warn`, and `Error`.
- Console, file, and JSON outputs should be configured through explicit sinks.
- Never log secrets, credentials, tokens, or raw tunnel payloads.

## CI checks

The Rust CI baseline includes:

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --workspace --all-features`
- `cargo build --workspace --all-features`

The workspace default members focus on server-side infrastructure so desktop Tauri builds do not block server iteration.
