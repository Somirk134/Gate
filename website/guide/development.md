# Development Guide

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
npm --prefix client run typecheck
```

Keep pull requests focused and update docs for user-visible behavior.
