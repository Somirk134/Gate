# Recovery

`SessionRecoveryManager` 只恢复运行态元数据，不恢复业务数据。

## Recoverable Scope

- Session identity
- Tunnel identity
- Runtime context attributes
- Subscription cursor
- Statistics

## Non-Recoverable Scope

- Business data
- Database state
- User payload
- External side effects

## Flow

```mermaid
flowchart TD
    Reconnected["ReconnectSucceeded"]
    Load["Load RecoveryContext"]
    HasContext{"Context exists?"}
    Session["Recover Session identity"]
    Tunnel["Recover Tunnel runtime state"]
    Stats["Recover Statistics"]
    Sub["Recover Subscription cursors"]
    Result["RecoveryResult"]
    Event["SessionRecovered event"]
    Sync["StateSyncManager.synchronize()"]
    Missing["RecoveryError::ContextNotFound"]

    Reconnected --> Load --> HasContext
    HasContext -- no --> Missing
    HasContext -- yes --> Session --> Tunnel --> Stats --> Sub --> Result --> Event --> Sync
```

## Result

`RecoveryResult` includes:

- `recovered_session`
- `recovered_tunnel`
- `recovered_statistics`
- `recovered_context`
- `recovered_subscription`
- `recovery_time_ms`
- `warnings`
