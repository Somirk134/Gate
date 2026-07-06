# Runtime Reliability Benchmark

Run date:

Command:

```powershell
cargo test -p gate-engine --test stress -- --ignored --nocapture
```

## Environment

| Field | Value |
| --- | --- |
| Commit |  |
| OS |  |
| CPU |  |
| Memory |  |
| Rust |  |
| Runtime profile | release/debug |

## CPU

| Scenario | Connections | CPU Avg | CPU Peak | Notes |
| --- | ---: | ---: | ---: | --- |
| TCP short | 1000 |  |  |  |
| TCP short | 5000 |  |  |  |
| TCP short | 10000 |  |  |  |
| HTTP KeepAlive | 1000 |  |  |  |
| HTTP KeepAlive | 5000 |  |  |  |
| HTTP KeepAlive | 10000 |  |  |  |
| HTTPS TLS KeepAlive | 1000 |  |  |  |
| HTTPS TLS KeepAlive | 5000 |  |  |  |
| HTTPS TLS KeepAlive | 10000 |  |  |  |

## Memory

| Scenario | Connections | Memory Start | Memory Peak | Memory End | Growth |
| --- | ---: | ---: | ---: | ---: | ---: |
| TCP long | 1000 |  |  |  |  |
| TCP long | 5000 |  |  |  |  |
| TCP long | 10000 |  |  |  |  |
| HTTPS TLS KeepAlive | 1000 |  |  |  |  |
| HTTPS TLS KeepAlive | 5000 |  |  |  |  |
| HTTPS TLS KeepAlive | 10000 |  |  |  |  |

## Latency

| Scenario | Connections | P50 | P90 | P99 | Max |
| --- | ---: | ---: | ---: | ---: | ---: |
| TCP short | 1000 |  |  |  |  |
| HTTP KeepAlive | 1000 |  |  |  |  |
| HTTPS TLS KeepAlive | 1000 |  |  |  |  |

## TLS

| Scenario | Connections | Handshakes | Avg Handshake | Handshake Errors |
| --- | ---: | ---: | ---: | ---: |
| HTTPS TLS KeepAlive | 1000 |  |  |  |
| HTTPS TLS KeepAlive | 5000 |  |  |  |
| HTTPS TLS KeepAlive | 10000 |  |  |  |

## Throughput

| Scenario | Connections | Requests | Elapsed | Requests/sec | Bytes/sec |
| --- | ---: | ---: | ---: | ---: | ---: |
| TCP short | 1000 |  |  |  |  |
| TCP short | 5000 |  |  |  |  |
| TCP short | 10000 |  |  |  |  |
| HTTP KeepAlive | 1000 |  |  |  |  |
| HTTP KeepAlive | 5000 |  |  |  |  |
| HTTP KeepAlive | 10000 |  |  |  |  |
| HTTPS TLS KeepAlive | 1000 |  |  |  |  |
| HTTPS TLS KeepAlive | 5000 |  |  |  |  |
| HTTPS TLS KeepAlive | 10000 |  |  |  |  |

## Reliability

| Metric | Value |
| --- | --- |
| Runtime Count |  |
| Connection Count |  |
| TLS Session |  |
| Restart Count |  |
| Health Score |  |
| WatchDog Findings |  |
| Recovery Events |  |

## Summary

Record regressions, bottlenecks, and follow-up actions here.
