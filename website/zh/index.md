---
layout: home

hero:
  name: Gate
  text: 面向团队的自托管隧道运行时
  tagline: Rust 优先运行时、Tauri 桌面客户端、可靠的私有服务访问，以及面向 GitHub 的开源生态。
  image:
    src: /logo.svg
    alt: Gate Logo
  actions:
    - theme: brand
      text: 快速开始
      link: /zh/guide/quick-start
    - theme: alt
      text: GitHub
      link: https://github.com/lancemorii-git/gate

features:
  - title: Rust 优先运行时
    details: 分层 workspace 覆盖协议、传输、通信、运行时、服务端和集成测试。
  - title: 自托管运维
    details: Docker、部署、反向代理、认证、心跳和监控文档统一组织。
  - title: 开源生态就绪
    details: CI、安全策略、模板、发布自动化、路线图、基准测试和社区指南。
---

## 架构

```mermaid
flowchart LR
  Client["桌面客户端"] --> Runtime["客户端运行时"]
  Runtime --> Engine["隧道引擎"]
  Engine --> Transport["传输层"]
  Transport <--> Server["Gate 服务端"]
  Server --> Auth["认证"]
  Server --> Monitor["监控"]
```
