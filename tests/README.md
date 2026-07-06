# Tests

Alpha V1 integration tests live in `integration/tests` and are run as the
`gate-integration` workspace crate.

```powershell
cargo test -p gate-integration
```

Legacy module-specific test folders under `tests/` remain available for
fixtures, monitoring notes, and mock-only coverage.
