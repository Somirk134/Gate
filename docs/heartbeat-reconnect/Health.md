# Health

`HealthManager` 聚合稳定性信号并输出统一健康状态。

## Targets

- Connection
- Heartbeat
- Authentication
- Runtime
- Tunnel
- Server

## Status

统一输出：

- `Healthy`
- `Warning`
- `Critical`
- `Offline`

## Report

`HealthReport` includes:

- `status`
- `score`
- `checked_at_millis`
- `components`

## Rules

- 任一组件 `Offline`，整体为 `Offline`。
- 任一组件 `Critical`，整体为 `Critical`。
- 任一组件 `Warning`，整体为 `Warning`。
- 全部健康时按平均分确认最终状态。

## Usage

Transport、Heartbeat、Reconnect、Runtime 等模块只上报信号，`HealthManager` 不直接执行网络探测。
