# Client Documentation

## Purpose

Client documentation describes the Gate desktop client architecture,
development workflow, and building/packaging process.

## Contents

- **Architecture.md** — Tauri + Vue 3 architecture overview
- **Development.md** — Development setup and workflow
- **IPC.md** — Frontend-backend IPC protocol
- **Packaging.md** — Building and packaging for all platforms
- **AutoUpdate.md** — Update mechanism
- **Testing.md** — Client testing strategy

## Tech Stack

| Concern | Technology |
|---------|-----------|
| Desktop Shell | Tauri 2 (Rust) |
| UI Framework | Vue 3 + TypeScript |
| Component Library | Naive UI |
| State Management | Pinia |
| Routing | Vue Router |
| i18n | vue-i18n |
| HTTP Client | Axios |

## Why This Design

Tauri 2 provides a minimal, secure, and performant desktop shell. Vue 3's
Composition API pairs well with Pinia for state management. Naive UI offers
a complete enterprise component library with TypeScript support.

## Extension

Add new views in `src/views`, new components in `src/components`, and new
stores in `src/stores`. Keep IPC commands organized by domain in `src-tauri/src/commands/`.
