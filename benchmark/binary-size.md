# Binary Size Benchmark

## Results

| Target | Binary | Size | Stripped Size | Notes |
| --- | --- | --- | --- | --- |
| `x86_64-unknown-linux-gnu` | `gate-server` |  |  |  |
| `x86_64-pc-windows-msvc` | `gate-server.exe` |  |  |  |
| `aarch64-apple-darwin` | `gate-server` |  |  |  |

## Commands

```bash
cargo build -p gate-server --release
```

## Notes

Track large dependency additions in release notes.
