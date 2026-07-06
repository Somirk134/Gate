# 心跳

心跳用于同步客户端和服务端会话状态。

```mermaid
sequenceDiagram
  participant C as 客户端运行时
  participant S as 服务端
  loop 活跃会话
    C->>S: heartbeat
    S-->>C: ack
  end
```
