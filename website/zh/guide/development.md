# 开发指南

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
npm --prefix client run typecheck
```

Pull Request 保持聚焦，用户可见行为需要同步更新文档。
