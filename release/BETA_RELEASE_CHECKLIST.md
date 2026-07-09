# Gate Beta Release Checklist

目标：Beta 用户可以下载安装 Gate，并完成第一次内网穿透。

## 1. 安装

- [ ] Windows 安装包可启动，主窗口、托盘、设置页正常显示。
- [ ] macOS 安装包可启动，首次打开无崩溃、无权限阻塞。
- [ ] Linux 包可启动，数据目录可写。
- [ ] 首次启动显示 Onboarding，支持跳过。
- [ ] 应用内无测试数据、Mock 文案、假 API 提示。

## 2. 配置

- [ ] 新用户选择“我已有服务器”后进入部署向导。
- [ ] 新用户选择“我没有服务器”后看到服务器准备说明。
- [ ] Linux VPS 部署命令可复制。
- [ ] Docker 部署命令可复制。
- [ ] Token、端口配置可填写。
- [ ] 连接测试能返回成功或可执行的错误建议。

## 3. 备份

- [ ] 设置页可导出 `gate-backup.zip`。
- [ ] 备份包含 `backup.json`。
- [ ] 备份包含 `database/client-runtime.json`。
- [ ] 备份包含 `database/projects.sqlite3`。
- [ ] 备份包含 `database/domains.sqlite3`。
- [ ] 备份包含 `certificate-metadata/certificates.json`。
- [ ] 备份不包含证书私钥或 PEM 内容。

## 4. 恢复

- [ ] 设置页可选择 `gate-backup.zip`。
- [ ] 恢复前展示 Projects、Servers、Tunnels、Domains、Certificates、Settings 数量。
- [ ] 版本不兼容时阻止恢复并显示错误。
- [ ] 确认恢复后先停止 Runtime。
- [ ] 恢复失败时回滚数据文件。
- [ ] 恢复成功后重新加载配置，服务器和 Tunnel 保持停止，等待用户手动连接。

## 5. Tunnel 测试

- [ ] 添加服务器并连接成功。
- [ ] 创建 TCP Tunnel，远端端口可访问本地 TCP 服务。
- [ ] 创建 HTTP Tunnel，公网 HTTP 地址可访问本地 Web 服务。
- [ ] 停止 Tunnel 后公网访问中断。
- [ ] 删除 Tunnel 后列表和 Runtime Store 同步更新。

## 6. HTTPS 测试

- [ ] 创建 HTTPS Tunnel 并绑定域名。
- [ ] ACME metadata 正常进入证书页。
- [ ] HTTPS 地址可访问本地 Web 服务。
- [ ] 证书页显示签发者、到期时间、续期状态。
- [ ] 备份恢复后证书 metadata 可见，但私钥/PEM 不被导出。

## 7. 发布前复核

- [ ] `client/src` 无 Mock、fake、TODO 用户路径文案。
- [ ] `client/src-tauri/src` 无 panic、unwrap、expect、TODO、mock、fake 命中。
- [ ] Dashboard、Projects、Servers、Tunnels、Certificates、Logs、Connections 均有空状态。
- [ ] 不新增协议。
- [ ] 不重构 Runtime。
- [ ] TCP / HTTP / HTTPS 数据面行为未改动。
