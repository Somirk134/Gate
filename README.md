<p align="center">
  <img src="./assets/logo/logo.svg" alt="Gate Logo" width="200" />
</p>

<h1 align="center">Gate</h1>

<p align="center">
  <strong>现代化 · 轻量级 · 可视化 · 开源 · 自部署</strong>
  <br />
  <em>Modern, lightweight, visual, open-source, self-hosted NAT traversal solution.</em>
</p>

<p align="center">
  <a href="./LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License" /></a>
  <a href="https://github.com/your-org/gate/releases"><img src="https://img.shields.io/github/v/release/your-org/gate" alt="Release" /></a>
  <a href="https://github.com/your-org/gate/actions"><img src="https://img.shields.io/github/actions/workflow/status/your-org/gate/ci.yml?branch=main" alt="CI" /></a>
  <br />
  <img src="https://img.shields.io/badge/Rust-1.78+-orange" alt="Rust" />
  <img src="https://img.shields.io/badge/Node.js-20+-green" alt="Node.js" />
  <img src="https://img.shields.io/badge/Tauri-2.0-purple" alt="Tauri" />
  <img src="https://img.shields.io/badge/Vue-3.5-brightgreen" alt="Vue 3" />
</p>

---

## Overview / 概述

**Gate** is a next-generation intranet penetration tool built for the GUI era. Unlike traditional tools that rely on terminal commands and manual configuration files, Gate provides a **complete visual operation experience** — every function is accessible through the graphical interface.

**Gate** 是一款为 GUI 时代打造的下一代内网穿透工具。与传统依赖命令行和配置文件的工具不同，Gate 提供**完整的可视化操作体验**——所有功能均可通过图形界面完成。

Whether you need to expose a local web service, access your home NAS remotely, or set up a temporary multiplayer server with friends, Gate lets you do it all with a few clicks.

无论你是需要暴露本地 Web 服务、远程访问家庭 NAS，还是与朋友搭建临时多人游戏服务器，Gate 都能让你通过几次点击即可完成。

> 🔧 **Status / 状态:** Early development (pre-alpha) / 早期开发阶段

---

## Screenshots / 界面预览

> *Screenshots will be added once the UI is implemented.*
> *界面截图将在 UI 实现后补充。*

| Dashboard / 仪表盘 | Tunnels / 隧道管理 | Connections / 连接管理 |
|:---------:|:-------:|:-----------:|
| *Coming soon / 即将上线* | *Coming soon / 即将上线* | *Coming soon / 即将上线* |

---

## Key Features / 核心特性

| English | 中文 |
|---------|------|
| 🖥️ **GUI First** — Complete desktop application, no terminal needed | 🖥️ **图形界面优先** — 完整的桌面应用，无需使用终端 |
| 📦 **Project Management** — Organize tunnels into projects for teams | 📦 **项目管理** — 将隧道组织到项目中，适合团队协作 |
| 👁️ **Visual Operation** — Create, edit, delete tunnels with a few clicks | 👁️ **可视化操作** — 通过几次点击即可创建、编辑、删除隧道 |
| 🔌 **Multiple Protocols** — TCP, UDP, HTTP, WebSocket tunnel support | 🔌 **多协议支持** — 支持 TCP、UDP、HTTP、WebSocket 隧道 |
| 🔐 **Secure by Default** — JWT, TLS encryption, rate limiting | 🔐 **默认安全** — JWT 认证、TLS 加密、速率限制 |
| 📊 **Real-time Statistics** — Live metrics, traffic graphs, client status | 📊 **实时统计** — 实时连接指标、流量图表、客户端状态 |
| 🚀 **Self-hosted** — Deploy on your own server or VPS | 🚀 **自部署** — 部署在你自己的服务器或 VPS 上 |
| 🤝 **Friend Servers** — Share access with friends on their own servers | 🤝 **朋友服务器** — 与朋友共享访问，使用各自的服务器 |
| 🔄 **Auto-update** — Built-in update mechanism | 🔄 **自动更新** — 内置的更新机制 |
| 🌐 **International** — Multi-language support (English, Chinese) | 🌐 **国际化** — 多语言支持（英文、中文） |

---

## Architecture / 架构设计

```
┌──────────────────────────────────────────────────┐
│                   Gate Client                     │
│  ┌───────────────────────────────────────────┐   │
│  │         Tauri 2 (Rust Backend)            │   │
│  │  System Tray  ·  Window Manager           │   │
│  │  IPC Commands ·  Auto-updater             │   │
│  │  Platform Native · Config Storage         │   │
│  └──────────────┬────────────────────────────┘   │
│  ┌──────────────▼────────────────────────────┐   │
│  │          Vue 3 (Frontend)                 │   │
│  │  Views  ·  Layouts  ·  Components         │   │
│  │  Pinia  ·  Vue Router  ·  vue-i18n       │   │
│  │  Naive UI  ·  Axios  ·  @vueuse/core     │   │
│  └───────────────────────────────────────────┘   │
└──────────────────────┬───────────────────────────┘
                       │ WebSocket / TCP / REST
┌──────────────────────▼───────────────────────────┐
│                   Gate Server                     │
│  ┌───────────────────────────────────────────┐   │
│  │        Transport Layer (Axum)             │   │
│  │  HTTP REST API  ·  WebSocket  ·  TCP     │   │
│  └──────────────────┬────────────────────────┘   │
│  ┌──────────────────▼────────────────────────┐   │
│  │      Application Layer (Services)         │   │
│  │  Auth  ·  Tunnel  ·  Connection  ·  Stats │   │
│  └──────────────────┬────────────────────────┘   │
│  ┌──────────────────▼────────────────────────┐   │
│  │        Domain Layer (Business Logic)      │   │
│  │  Models  ·  Repositories  ·  Events       │   │
│  └──────────────────┬────────────────────────┘   │
│  ┌──────────────────▼────────────────────────┐   │
│  │     Infrastructure Layer (Data Access)    │   │
│  │  SQLite (dev)  ·  PostgreSQL (prod)       │   │
│  │  In-memory Cache  ·  File Storage         │   │
│  └───────────────────────────────────────────┘   │
└──────────────────────────────────────────────────┘
```

---

## Tech Stack / 技术栈

### Client / 客户端

| Category / 类别 | Technology / 技术 |
|----------|-----------|
| Desktop Framework / 桌面框架 | [Tauri 2](https://v2.tauri.app/) (Rust) |
| UI Framework / 界面框架 | [Vue 3](https://vuejs.org/) + [TypeScript](https://www.typescriptlang.org/) |
| Build Tool / 构建工具 | [Vite](https://vitejs.dev/) |
| Component Library / 组件库 | [Naive UI](https://www.naiveui.com/) |
| State Management / 状态管理 | [Pinia](https://pinia.vuejs.org/) |
| Router / 路由 | [Vue Router](https://router.vuejs.org/) |
| i18n / 国际化 | [vue-i18n](https://vue-i18n.intlify.dev/) |
| HTTP Client / HTTP 客户端 | [Axios](https://axios-http.com/) |

### Server / 服务端

| Category / 类别 | Technology / 技术 |
|----------|-----------|
| Runtime / 运行时 | [Tokio](https://tokio.rs/) (async Rust) |
| Web Framework / Web 框架 | [Axum](https://github.com/tokio-rs/axum) |
| Middleware / 中间件 | [Tower](https://github.com/tower-rs/tower) + [tower-http](https://github.com/tower-rs/tower-http) |
| Database / 数据库 | [SQLx](https://github.com/launchbadge/sqlx) (SQLite / PostgreSQL) |
| Auth / 认证 | [JWT](https://github.com/Keats/jsonwebtoken) + [Argon2](https://docs.rs/argon2/) |
| Serialization / 序列化 | [Serde](https://serde.rs/) |
| Logging / 日志 | [Tracing](https://github.com/tokio-rs/tracing) |

### Protocol / 协议

```
Custom binary protocol over TCP/WebSocket
┌────────┬──────────────────────────────────────┐
│ Length │           JSON Payload                │
│ 4 bytes│         (variable size)               │
└────────┴──────────────────────────────────────┘
```

---

## Getting Started / 快速开始

### Prerequisites / 环境要求

| Dependency / 依赖 | Version / 版本 | Purpose / 用途 |
|------------|---------|---------|
| [Rust](https://www.rust-lang.org/) | 1.78+ | Server + Tauri backend / 服务端 + Tauri 后端 |
| [Node.js](https://nodejs.org/) | 20+ | Client frontend / 客户端前端 |
| [pnpm](https://pnpm.io/) | 9+ | Package manager / 包管理器 |
| [Tauri prerequisites](https://v2.tauri.app/start/prerequisites/) | — | Platform-specific build tools / 平台构建工具 |

### Quick Start / 快速启动

```bash
# 1. Clone the repository / 克隆仓库
git clone https://github.com/your-org/gate.git
cd gate

# 2. Start the server / 启动服务端
cargo run -p gate-server

# 3. Start the client / 启动客户端（另开一个终端）
cd client
pnpm install
pnpm tauri dev
```

### Docker (Server / 服务端)

```bash
# Build and run with Docker / 使用 Docker 构建并运行
docker build -f Dockerfile.server -t gate-server .
docker run -d \
  --name gate-server \
  -p 5800:5800 \
  -p 10000-11000:10000-11000 \
  -v ./data:/app/data \
  gate-server
```

---

## Project Structure / 项目结构

```
Gate/
├── client/                # Desktop client / 桌面客户端 (Tauri 2 + Vue 3)
│   ├── src/               # Vue 3 frontend / 前端源码
│   └── src-tauri/         # Tauri Rust backend / Tauri 后端
├── crates/
│   ├── server/            # Gate server / 服务端 (Axum + Tokio)
│   └── shared/            # Shared library / 共享库 (protocol, types, utils)
├── docs/                  # Documentation / 文档
├── scripts/               # Development & CI scripts / 开发与 CI 脚本
├── assets/                # Branding & static assets / 品牌与静态资源
├── release/               # Platform packaging configs / 平台打包配置
└── .github/               # GitHub templates & workflows / GitHub 模板和工作流
```

---

## Documentation / 文档

| Directory / 目录 | Contents / 内容 |
|-----------|----------|
| [01-product](./docs/01-product/) | Product vision, personas, use cases / 产品愿景、用户画像、用例 |
| [02-domain](./docs/02-domain/) | Domain model, glossary / 领域模型、术语表 |
| [03-architecture](./docs/03-architecture/) | System architecture, ADRs / 系统架构、架构决策记录 |
| [04-protocol](./docs/04-protocol/) | Wire protocol spec / 线缆协议规范 |
| [05-ui](./docs/05-ui/) | Design system, wireframes / 设计系统、线框图 |
| [06-server](./docs/06-server/) | Server config, deployment / 服务端配置、部署 |
| [07-client](./docs/07-client/) | Client architecture, IPC / 客户端架构、IPC |
| [08-api](./docs/08-api/) | REST API & WebSocket reference / API 参考 |
| [09-deployment](./docs/09-deployment/) | Docker, K8s, systemd guides / 部署指南 |
| [10-roadmap](./docs/10-roadmap/) | Milestones, release plan / 里程碑、发布计划 |

---

## Roadmap / 路线图

| Version / 版本 | Focus / 重点 | Status / 状态 |
|---------|-------|--------|
| v0.1.0 | Project scaffolding / 项目骨架搭建 | ✅ Done / 完成 |
| v0.2.0 | Core protocol & server tunneling / 核心协议与服务端隧道 | 🔄 In progress / 进行中 |
| v0.3.0 | Client GUI & IPC / 客户端界面与 IPC | 📋 Planned / 计划中 |
| v0.4.0 | Authentication & security / 认证与安全 | 📋 Planned / 计划中 |
| v0.5.0 | Statistics & monitoring / 统计与监控 | 📋 Planned / 计划中 |
| v0.6.0 | Deployment & packaging / 部署与打包 | 📋 Planned / 计划中 |
| v1.0.0 | Production release / 正式发布 | 🎯 Target / 目标 |

---

## Contributing / 参与贡献

We welcome contributions from the community! Please see our [Contributing Guide](./CONTRIBUTING.md) to get started.

欢迎社区贡献！请查阅[贡献指南](./CONTRIBUTING.md)了解如何参与。

- 🐛 **Found a bug? / 发现 Bug？** Open an [issue](https://github.com/your-org/gate/issues/new?template=bug_report.md)
- 💡 **Have an idea? / 有 idea？** Start a [discussion](https://github.com/your-org/gate/discussions)
- 🔧 **Want to contribute? / 想贡献代码？** Check [good first issues](https://github.com/your-org/gate/issues?q=is%3Aissue+is%3Aopen+label%3A%22good+first+issue%22)

---

## Community / 社区

| Channel / 渠道 | Purpose / 用途 |
|---------|---------|
| [GitHub Discussions](https://github.com/your-org/gate/discussions) | Q&A, ideas, show & tell / 问答、想法、分享 |
| [GitHub Issues](https://github.com/your-org/gate/issues) | Bug reports & feature requests / 缺陷报告与功能请求 |

---

## License / 许可证

This project is licensed under the [MIT License](./LICENSE).

本项目采用 [MIT 许可证](./LICENSE)。

Copyright (c) 2026 Gate Contributors.

---

<p align="center">
  <strong>Gate</strong> — Break through the wall, simply. / 打破壁垒，如此简单。
</p>
