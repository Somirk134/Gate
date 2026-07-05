# Gate Error Handling

## Error Types

统一错误体系定义在 `gate-shared`：

- `AppError`
- `NetworkError`
- `ConfigError`
- `TunnelError`
- `InternalError`
- `ErrorCode`

所有错误类型通过 `thiserror` 实现 `std::error::Error`。

## ErrorCode

`ErrorCode` 是稳定分类，不直接绑定 HTTP status、CLI exit code 或日志格式：

| Code | 含义 |
| --- | --- |
| `UNKNOWN` | 未分类错误预留 |
| `CONFIG` | 配置来源、配置值、配置优先级相关错误 |
| `NETWORK` | 网络组件边界错误 |
| `TUNNEL` | Tunnel 能力预留错误 |
| `INTERNAL` | 组件缺失、运行期不变量失败 |

## AppError

`AppError` 是跨层返回的统一错误类型：

```rust
pub enum AppError {
    Config(ConfigError),
    Network(NetworkError),
    Tunnel(TunnelError),
    Internal(InternalError),
}
```

应用层、基础设施端口、传输层端口都应该返回 `AppError` 或更局部的错误，然后在边界处转换为 `AppError`。

## Rules

- 不使用字符串作为长期错误协议。
- 不把 SQLx、Redis、Axum、Tower 的具体错误泄漏到领域层。
- 不在错误类型里携带认证、Token、密钥、连接负载等敏感内容。
- `TunnelError` 当前只表示未来能力预留，不代表已经实现 Tunnel。
- `InternalError` 只描述基础设施或运行期不变量，不承载业务失败。

