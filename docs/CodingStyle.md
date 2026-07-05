# Gate Rust Coding Style

## 基本原则

- Rust Stable only。
- 所有 crate 职责单一。
- 依赖方向必须遵守 Clean Architecture，不允许跨层捷径。
- 不使用全局可变状态；运行期依赖通过 `AppContext`、Registry、Provider、Factory 注入。
- 默认不添加抽象，除非它代表清晰边界或已有局部模式。

## Module Convention

- `mod.rs` 只声明模块和必要 re-export。
- 业务模块统一包含 `service.rs`、`repository.rs`、`entity.rs`、`error.rs`、`event.rs`、`handler.rs`、`types.rs`。
- 不在 `shared` 放领域实体。
- 不在 `domain` 引入 Axum、Tower、SQLx、Redis、Tokio 网络类型。
- 不在 `transport` 写业务决策。

## Error Convention

- 跨层错误统一进入 `gate_shared::error`。
- 对外暴露 `AppError`。
- 具体类别使用 `ConfigError`、`NetworkError`、`TunnelError`、`InternalError`。
- 新错误必须提供稳定 `ErrorCode` 映射。
- 使用 `thiserror` 实现 Error Trait。

## Async Convention

- Tokio 作为异步运行时。
- 当前 Trait 先保持同步抽象；需要异步时先评估对象安全、生命周期和调用边界。
- 不在本阶段启动后台任务、监听端口或 spawn 长生命周期任务。

## Logging Convention

- 统一使用 `tracing`。
- 日志级别为 `Trace`、`Debug`、`Info`、`Warn`、`Error`。
- Console、File、JSON 输出当前只建配置与接口，具体 sink 后续再接入。

## CI Reservation

CI 阶段预留：

- `fmt`
- `clippy`
- `test`
- `build`
- `release`

当前 Rust workspace 的 default members 聚焦服务端基础架构，避免客户端 Tauri 构建影响服务端骨架迭代。

