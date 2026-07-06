# 隧道

隧道将本地端点连接到 Gate 服务端暴露的远端端点。

```toml
[tunnel]
name = "local-web"
protocol = "tcp"
local = "127.0.0.1:3000"
remote = "0.0.0.0:8080"
```
