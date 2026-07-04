# Architecture Documentation

## Purpose

Architecture documentation describes the high-level system architecture,
component relationships, and key architectural decisions.

## Contents

- **Overview.md** — System context and architecture diagram
- **ClientArchitecture.md** — Client-side architecture (Tauri + Vue 3)
- **ServerArchitecture.md** — Server-side architecture (DDD layers)
- **NetworkArchitecture.md** — Network topology and data flow
- **SecurityArchitecture.md** — Security model and threat analysis
- **Decisions.md** — Architecture Decision Records (ADRs)

## Principles

1. **Separation of Concerns** — Each layer has a distinct responsibility
2. **Dependency Inversion** — Domain layer depends on abstractions, not infrastructure
3. **Observability** — All components emit structured telemetry
4. **Secure by Default** — Security is built in, not bolted on
5. **Fail Closed** — On error, deny access rather than allow

## Why This Design

The hexagonal architecture (ports and adapters) pattern is used to keep the
domain core isolated from frameworks, databases, and transport protocols.
This enables the system to evolve independently in each concern.

## Extension

Architecture documentation should be updated whenever a new component or
significant integration is added. Each ADR should document a significant
architectural decision with context, options considered, and rationale.
