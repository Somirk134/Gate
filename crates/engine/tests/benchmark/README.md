# Benchmark Tests

Runtime reliability stress coverage is implemented as ignored integration tests
in `crates/engine/tests/stress.rs`.

Run manually:

```powershell
cargo test -p gate-engine --test stress -- --ignored --nocapture
```

Connection pressure matrix:

- TCP short connections: 1000, 5000, 10000
- TCP long connections: 1000, 5000, 10000
- HTTP KeepAlive: 1000, 5000, 10000
- HTTPS TLS KeepAlive: 1000, 5000, 10000

Record benchmark output in `benchmark/runtime-reliability.md`.
