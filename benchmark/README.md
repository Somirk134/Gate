# Benchmark

Benchmark templates keep performance reports consistent across releases.

## Areas

| Area | Template |
| --- | --- |
| Memory | [memory.md](./memory.md) |
| CPU | [cpu.md](./cpu.md) |
| Latency | [latency.md](./latency.md) |
| Connection | [connection.md](./connection.md) |
| Binary Size | [binary-size.md](./binary-size.md) |
| Runtime | [runtime.md](./runtime.md) |

## Reporting Rules

- Include commit SHA, platform, CPU, memory, OS, and build profile.
- Compare against the previous release when possible.
- Keep raw logs or generated reports linked from the release notes.
- Separate synthetic benchmarks from real deployment measurements.
