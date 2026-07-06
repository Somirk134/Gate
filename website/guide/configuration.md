# Configuration

| Variable | Default | Description |
| --- | --- | --- |
| `GATE_ENV` | `development` | Runtime profile |
| `GATE_BIND` | `127.0.0.1:5800` | Server bind address |
| `GATE_LOG` | `info` | Log filter |
| `GATE_DATA_DIR` | `./data` | Data directory |

```bash
GATE_ENV=production GATE_BIND=0.0.0.0:5800 cargo run -p gate-server --release
```
