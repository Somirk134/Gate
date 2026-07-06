# Contribution Workflow

## Flow

```mermaid
flowchart LR
  Start["Find issue or start discussion"] --> Branch["Create branch"]
  Branch --> Change["Make focused change"]
  Change --> Test["Run local checks"]
  Test --> PR["Open pull request"]
  PR --> CI["CI and security checks"]
  CI --> Review["Maintainer review"]
  Review --> Merge["Merge"]
```

## Local Checks

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
npm --prefix client run typecheck
```

## Review Tips

- Explain why the change is needed.
- Keep screenshots for UI changes.
- Update docs for user-visible behavior.
- Note breaking changes clearly.
