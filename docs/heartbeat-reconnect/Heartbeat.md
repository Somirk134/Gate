# Heartbeat

`HeartbeatManager` 是异步心跳状态机，支持 `Start`、`Stop`、`Pause`、`Resume`、`Tick`、`Ping`、`Pong`、`Timeout`。

## State

```text
Idle -> Running -> WaitingPong -> Running
WaitingPong -> Timeout -> Retrying -> WaitingPong
Running/WaitingPong/Timeout/Retrying -> Stopped
Running -> Idle -> Running
```

## Config

`HeartbeatConfig` 支持 Builder Pattern：

- `interval`
- `timeout`
- `retry_count`
- `retry_delay`
- `max_missed_heartbeat`

## Sequence

```mermaid
sequenceDiagram
    participant Scheduler
    participant HeartbeatManager
    participant Transport
    participant EventBus

    Scheduler->>HeartbeatManager: start(tunnel_id)
    HeartbeatManager-->>EventBus: HeartbeatStarted
    loop every interval
        Scheduler->>HeartbeatManager: tick(tunnel_id)
        HeartbeatManager->>HeartbeatManager: state Running -> WaitingPong
        HeartbeatManager-->>Transport: Ping snapshot
        Transport-->>HeartbeatManager: pong(tunnel_id, sequence)
        HeartbeatManager->>HeartbeatManager: update RTT and metrics
    end
    Scheduler->>HeartbeatManager: timeout(tunnel_id)
    HeartbeatManager->>HeartbeatManager: WaitingPong -> Timeout
    HeartbeatManager-->>EventBus: HeartbeatTimeout
```

## Metrics

心跳指标包括：

- `ping_count`
- `pong_count`
- `timeout_count`
- `retry_count`
- `heartbeat_count`
- `average_rtt_ms`
- `last_rtt_ms`

## Boundary

`HeartbeatManager` 不发送真实 ping 包。真实 transport 应在发送前调用 `ping()`，收到 pong 后调用 `pong()`。
