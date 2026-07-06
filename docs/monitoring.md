# Monitoring

Gate monitoring focuses on tunnel health, runtime stability, traffic, latency, and error signals.

## Metric Groups

| Group | Examples |
| --- | --- |
| Runtime | uptime, worker count, active sessions |
| Connection | active connections, reconnect count, failures |
| Traffic | bytes in, bytes out, packets, throughput |
| Heartbeat | last seen, missed heartbeat, timeout count |
| System | memory, CPU, binary size, runtime latency |

## Dashboard Expectations

- Show health first.
- Make stale data visually distinct.
- Separate live counters from historical trends.
- Link errors to troubleshooting guidance.

## Alert Candidates

- Session heartbeat timeout.
- Reconnect loop exceeds threshold.
- Unexpected traffic drop.
- CPU or memory exceeds baseline.
- Server rejects authentication repeatedly.
