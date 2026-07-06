# Performance

V1 focuses on correctness, stable lifecycle, and a clean data-plane boundary.

## Current Strategy

- Tokio TCP listener and streams.
- `copy_bidirectional` for kernel-backed async socket forwarding.
- Atomic traffic counters.
- `DashMap` for session and task registries.
- Reusable `BufferPool` boundary for future custom pipeline loops.
- Centralized spawning through `RuntimeScheduler`.

## Preserved Future Paths

- custom buffer reuse and zero-copy pipeline
- io_uring runtime backend
- mio runtime backend
- protocol-aware HTTP and HTTPS pipeline stages
- UDP datagram forwarding
- P2P relay and direct connection strategies

## Benchmark Plan

Benchmark interfaces are reserved under `crates/engine/tests/benchmark`.

Planned scenarios:

- 100 concurrent TCP connections
- 500 concurrent TCP connections
- 1000 concurrent TCP connections
- 5000 concurrent TCP connections
- 10000 concurrent TCP connections

V1 intentionally does not implement pressure benchmarks.
