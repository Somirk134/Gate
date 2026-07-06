---
layout: home

hero:
  name: Gate
  text: Self-hosted tunnel runtime for teams
  tagline: Rust-first runtime, Tauri desktop client, reliable private service access, and a GitHub-ready open source ecosystem.
  image:
    src: /logo.svg
    alt: Gate logo
  actions:
    - theme: brand
      text: Quick Start
      link: /guide/quick-start
    - theme: alt
      text: GitHub
      link: https://github.com/lancemorii-git/gate

features:
  - title: Rust-first runtime
    details: Layered workspace for protocol, transport, communication, runtime, server, and integration testing.
  - title: Self-hosted operations
    details: Docker, deployment, reverse proxy, authentication, heartbeat, and monitoring documentation.
  - title: Open source ready
    details: CI, security policy, templates, release automation, roadmap, benchmark, and community guides.
---

## Architecture

```mermaid
flowchart LR
  Client["Desktop Client"] --> Runtime["Client Runtime"]
  Runtime --> Engine["Tunnel Engine"]
  Engine --> Transport["Transport"]
  Transport <--> Server["Gate Server"]
  Server --> Auth["Authentication"]
  Server --> Monitor["Monitoring"]
```
