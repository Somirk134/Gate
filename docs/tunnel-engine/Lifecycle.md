# Lifecycle

Tunnel Engine uses one unified lifecycle state model.

## States

```text
Created
Initializing
Ready
Running
Paused
Stopping
Stopped
Restarting
Error
```

## State Diagram

```mermaid
stateDiagram-v2
    [*] --> Created
    Created --> Initializing
    Initializing --> Ready
    Initializing --> Error
    Ready --> Running
    Ready --> Stopping
    Running --> Paused
    Running --> Stopping
    Running --> Restarting
    Running --> Error
    Paused --> Running
    Paused --> Stopping
    Stopping --> Stopped
    Restarting --> Initializing
    Restarting --> Error
    Stopped --> Initializing
    Error --> Stopping
    Error --> Restarting
    Error --> Stopped
```

## Lifecycle Contract

`EngineLifecycle` defines:

- `state()`
- `initialize()`
- `start()`
- `pause()`
- `stop()`
- `restart()`

All transitions pass through `EngineState`, which rejects invalid transitions.
