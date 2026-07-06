# Development Guide

## Workspace Checks

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
```

## Client Checks

```bash
cd client
npm install
npm run typecheck
npm run lint
npm run build
```

## Documentation Checks

```bash
cd website
npm install
npm run docs:build
```

## Working Agreements

- Keep changes scoped.
- Prefer existing workspace abstractions.
- Add tests for shared behavior or bug fixes.
- Update docs for operator-visible changes.
- Use issues and discussions for larger design changes.

## Directory Ownership

| Directory | Owner Mindset |
| --- | --- |
| `crates` | Runtime correctness and API boundaries |
| `client` | Desktop user experience and IPC |
| `server` | Deployable server behavior |
| `docs` | Operator and contributor clarity |
| `.github` | Automation and community maintenance |
