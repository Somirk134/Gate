# Testing

Gate uses Rust workspace tests and client type/build checks as the v0.9 release gate.

## Required release checks

```bash
cargo check --workspace
cargo test --workspace
npm run typecheck
npm run build
```

## Rust checks

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features
cargo test --workspace --all-features
```

## Client checks

```bash
npm --prefix client ci
npm --prefix client run typecheck
npm --prefix client run build
npm --prefix client run lint:check
```

## Integration tests

```bash
cargo test -p gate-integration --all-features
```

Historical validation reports were removed from the public docs set. Reproducible test commands are preferred over one-off process reports.
