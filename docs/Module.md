# Gate Server Module Guide

## Domain Modules

领域层已预留以下模块：

- `Project`
- `Tunnel`
- `Server`
- `Connection`
- `Statistics`
- `Log`
- `Settings`
- `Session`
- `Health`
- `Event`

每个业务模块保持同一文件结构：

```text
module/
├── mod.rs
├── service.rs
├── repository.rs
├── entity.rs
├── error.rs
├── event.rs
├── handler.rs
└── types.rs
```

## 文件职责

| 文件 | 职责 |
| --- | --- |
| `entity.rs` | 未来放聚合根、实体和值对象；当前只占位 |
| `repository.rs` | 未来放领域仓储 Trait；当前只定义指定 Repository 空 Trait |
| `service.rs` | 未来放领域服务 Trait |
| `error.rs` | 未来放领域局部错误 |
| `event.rs` | 未来放领域事件 |
| `handler.rs` | 未来放领域事件处理边界 |
| `types.rs` | 未来放类型别名和值对象 |
| `mod.rs` | 只做模块声明，不放逻辑 |

## Repository Trait

当前已建立：

- `ProjectRepository`
- `TunnelRepository`
- `ServerRepository`
- `SettingsRepository`
- `LogRepository`

这些 Trait 暂时不包含方法。后续添加方法时，必须先从用例需求反推，不允许为了某个数据库表提前设计 CRUD。

## Application Traits

应用层当前只提供抽象：

- `Service`
- `UseCase`
- `Command`
- `Query`
- `Handler`
- `CommandDispatcher`
- `QueryDispatcher`
- `EventDispatcher`
- `EventHandler`

应用层负责表达“系统要做什么”，不关心“用什么协议进来”或“数据存在哪里”。

## Infrastructure Traits

基础设施层当前只提供端口：

- `Storage`
- `ConfigProvider`
- `LoggerProvider`
- `Cache`
- `Network`
- `RuntimeProvider`
- `Scheduler`

未来 SQLx、Redis、TLS、文件日志、JSON 日志、MessagePack 都必须作为实现接入端口，不能把实现类型泄漏回领域层。

## Transport Traits

传输层当前只有接口：

- `Transport`
- `TcpTransport`
- `TcpEndpoint`
- `HttpTransport`
- `IpcTransport`
- `WebSocketTransport`

TCP 是本阶段唯一预留的当前传输模块，但不监听端口、不解析协议、不转发数据。

