# Beta Sprint 2: Deployment and Onboarding UX

目标：开发者第一次下载 Gate 后，不阅读文档，也能在 5 分钟内完成部署、连接服务器并创建第一个 Tunnel。

本 Sprint 不新增核心能力，不新增 Tunnel 协议，不新增 P2P，不新增新的 HTTP 能力。所有改动围绕部署流程、配置流程、首次使用体验、错误提示和诊断工具。

## 新手体验方案

1. 首次启动不直接暴露 Dashboard，而是显示 Welcome Wizard。
2. Welcome Wizard 按 `Welcome -> 检查环境 -> 配置服务器 -> 测试连接 -> 创建第一个 Tunnel -> 完成` 推进。
3. Server Connection Wizard 要求用户只填写服务器地址、端口和 Token，并提供实时状态反馈。
4. 连接测试必须给出结构化错误：错误原因、可能原因、解决方案、查看日志和复制错误。
5. Quick Start 提供本地开发、支付回调、Webhook、SSH、数据库、NAS、Docker 等场景。
6. Tunnel Template 只生成推荐配置，不改变协议层。HTTP 作为预留模板展示。
7. Diagnostics Center 聚合 Connection Diagnostics、Deployment Checker、Version Checker、System Info、Recent Server 和 Connection History。
8. Settings 增加恢复默认设置、导出配置、导入配置、备份配置、重置缓存、清理日志。
9. Feedback 页面支持复制调试信息、打开 GitHub Issue、查看日志目录、查看配置目录、生成诊断信息。

## 首次使用流程图

```mermaid
flowchart TD
    A["第一次打开 Gate"] --> B["Welcome Wizard"]
    B --> C["检查环境"]
    C --> D{是否有阻塞错误}
    D -- "有" --> E["展示原因、可能原因、解决方案"]
    E --> C
    D -- "无" --> F["配置服务器地址、端口、Token"]
    F --> G["测试连接"]
    G --> H{连接是否通过}
    H -- "否" --> I["分类错误：DNS / Token / 未启动 / 端口 / 超时"]
    I --> J["查看日志或复制错误"]
    J --> F
    H -- "是" --> K["选择 Quick Start 场景"]
    K --> L["选择 Tunnel Template"]
    L --> M["自动填充推荐配置"]
    M --> N["创建第一个 Tunnel"]
    N --> O["完成并进入 Tunnels"]
```

## 部署流程图

```mermaid
flowchart TD
    A["启动前自动检查"] --> B["检查 Rust Server 是否运行"]
    B --> C["检查配置文件是否存在"]
    C --> D["检查日志目录"]
    D --> E["检查写入权限"]
    E --> F["检查监听端口"]
    F --> G["检查配置是否合法"]
    G --> H{检查结果}
    H -- "通过" --> I["继续连接服务器"]
    H -- "警告" --> J["允许继续，但显示修复建议"]
    H -- "错误" --> K["阻止继续并展示解决方案"]
    K --> L["用户修复部署问题"]
    L --> A
```

## 连接流程图

```mermaid
flowchart TD
    A["用户填写服务器地址、端口、Token"] --> B["格式校验"]
    B --> C{格式正确}
    C -- "否" --> D["提示地址或端口格式错误"]
    C -- "是" --> E["DNS 解析"]
    E --> F{DNS 成功}
    F -- "否" --> G["DNS 错误"]
    F -- "是" --> H["TCP 端口连接"]
    H --> I{端口可达}
    I -- "拒绝连接" --> J["服务器未启动"]
    I -- "不可达" --> K["端口无法访问"]
    I -- "超时" --> L["连接超时"]
    I -- "可达" --> M["Gate 协议认证"]
    M --> N{Token 合法}
    N -- "否" --> O["Token 错误"]
    N -- "是" --> P["记录 Recent Server 和 Connection History"]
    P --> Q["允许创建 Tunnel"]
```

## 验收标准

- 首次启动向导覆盖首屏，用户不需要先理解 Dashboard。
- 服务器连接失败不再显示泛化的 `Connection Failed`。
- 诊断页能输出部署检查、连接检查、版本信息和系统信息。
- 设置页所有维护操作都有可视化入口。
- 反馈页能一键复制可提交的调试信息。
- Tunnel 创建向导支持 Quick Start 和 Template 自动填充。
- 不引入新协议，不新增 HTTP/P2P 等核心能力。
