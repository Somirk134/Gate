# 部署

```bash
docker compose -f docker/docker-compose.yml up -d
```

生产检查：

- 明确配置监听地址。
- 使用反向代理或运行时 TLS。
- Secret 不进入镜像和 git。
- 验证心跳与代理超时设置。
