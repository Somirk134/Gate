# Real World Tunnel Validation Report

日期：2026-07-09

## 测试环境

- 测试入口：`integration/e2e/phase3_real_world_validation.rs`
- 真实 Server 进程：`gate-e2e-server`，内部调用 `gate_server::ServerBootstrap`
- 真实 Client 进程：`gate-e2e-client`，内部调用 `ClientRuntimeState`
- 真实 Local Service 进程：`gate-e2e-local-service`
- 网络：本机 loopback，Server 控制端口、Tunnel 公网端口、Local Service 端口均使用真实 TCP socket
- 协议：不新增协议命令，恢复流程复用 `AuthLogin`、`TunnelRegister/TunnelStart`、`TunnelRelayConnect`

## 覆盖场景

| 场景 | 状态 | 说明 |
| --- | --- | --- |
| TCP Tunnel 正常链路 | 通过 | Server 启动、Client 连接、Tunnel 注册、公网 TCP 访问、Relay 成功 |
| HTTP Tunnel 正常链路 | 通过 | 公网 HTTP 请求经 Tunnel 转发到真实本地 HTTP 服务 |
| Server Restart Recovery | 通过 | Client 保持运行，Server 停止后重启，自动重连、重新认证、重新注册 Tunnel，公网访问恢复 |
| Client Restart Recovery | 通过 | Client 强制关闭后用同一运行时目录重启，无需重新创建配置即可恢复 Tunnel |
| Network Interruption | 通过 | 默认快速场景与 10/30/60 秒完整矩阵均已通过 |
| Stress Test | 通过 | 100/500/1000 并发连接矩阵已通过 |

## 本轮发现与修复

- Client 异常退出后，运行中的 Tunnel 不能被当成普通 stopped 状态处理。已改为保存 `recovering` 意图，并在启动后恢复 active server 与 running Tunnel。
- Server 重启后内存态 Session/Tunnel 会丢失。Client 心跳失败后现在会自动重连、复用旧 session id 重新认证，并重新注册 Tunnel。
- Relay worker 默认数量较小，不适合 100/500/1000 并发压测。新增 `GATE_RELAY_WORKERS_PER_TUNNEL` 作为运行时压测配置，默认仍为 4。
- 增加恢复指标：reconnect count、recovery success/failure、last/average recovery time、tunnel uptime。
- Server Gateway 增加 active/recovered/failed session 指标，以及每个 Tunnel 的 uptime。

## 验证结果

已执行：

```powershell
cargo check -p gate-integration --tests --bins
cargo test -p gate-integration --test phase3_real_world_validation -- --nocapture
cargo test -p gate-integration --test phase3_real_world_validation stress_matrix_100_500_1000_connections -- --ignored --nocapture
cargo test -p gate-integration --test phase3_real_world_validation network_interruption_matrix_10_30_60_seconds -- --ignored --nocapture
cargo test -p gate-server gateway::tests
cargo test -p gate-client runtime::tests::client_reconnect_restores_session_and_registers_running_tunnel
```

结果：

- `phase3_real_world_validation`：4 passed，2 ignored（完整中断矩阵、完整压力矩阵）
- `stress_matrix_100_500_1000_connections`：1 passed
- `network_interruption_matrix_10_30_60_seconds`：1 passed
- `gate-server gateway::tests`：5 passed
- `gate-client runtime::tests::client_reconnect_restores_session_and_registers_running_tunnel`：1 passed

## 全量验证命令

运行 10/30/60 秒中断矩阵与 100/500/1000 并发压测：

```powershell
cargo test -p gate-integration --test phase3_real_world_validation -- --ignored --nocapture
```

## 结论

当前 Gate TCP Tunnel 已具备真实进程级端到端验证能力，并通过默认恢复场景、10/30/60 秒中断矩阵、100/500/1000 并发压测证明：Server 重启、Client 重启、网络中断后，Tunnel 可以自动恢复公网访问。完整长期运行结论仍建议在独立环境定期重复执行中断矩阵和压力矩阵，并持续观察 CPU、Memory、Connection、Latency、Error 与恢复指标。
