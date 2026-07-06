# Heartbeat

Heartbeat keeps client and server session state synchronized.

## Responsibilities

- Detect dead clients.
- Mark tunnel sessions unhealthy.
- Trigger reconnect flow.
- Feed monitoring metrics and troubleshooting views.

## Defaults

| Setting | Example |
| --- | --- |
| Interval | `30s` |
| Timeout | `90s` |
| Retry backoff | Exponential with jitter |

## Flow

```mermaid
sequenceDiagram
  participant C as Client Runtime
  participant S as Server
  loop Active session
    C->>S: heartbeat
    S-->>C: heartbeat ack
  end
  S->>S: mark unhealthy on timeout
```

## Troubleshooting

- Check network reachability between client and server.
- Check server logs for authentication or timeout errors.
- Compare heartbeat interval and timeout values with proxy idle timeout settings.
