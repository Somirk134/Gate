# Health

HealthCenter provides one unified health report for Gate modules.

## Status

- `Healthy`
- `Warning`
- `Critical`
- `Offline`

## Targets

- Tunnel
- Connection
- Runtime
- Heartbeat
- Server
- System

## Flow

```mermaid
flowchart TB
  Statistics["Statistics Snapshot"] --> Rules["Health Rules"]
  Rules --> Signals["HealthSignal[]"]
  Signals --> Report["HealthReport"]
  Report --> Dashboard["System Health Panel"]
  Report --> Alert["AlertManager.evaluate_health"]
```

Health is intentionally local and in-memory in this phase. Persistence, distributed health quorum, and external incident channels are future work.
