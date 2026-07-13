# Changelog

All notable changes to Gate are documented in this file.

## 0.9.2 - 2026-07-13

- Improve tunnel worker selection, relay buffering, stalled-write handling, and HTTP connection behavior.
- Improve local and exported log retention so stability incidents remain diagnosable.
- Make GitHub Release assets deterministic and validate all updater packages, signatures, and platform URLs before publishing.
- Align Cargo, npm, Tauri, Docker, desktop UI, and release documentation on `0.9.2`.

## 0.9.1 - 2026-07-10

### Security

- Require an explicit authentication token of at least 16 characters and reject known weak defaults.
- Remove authentication tokens from server startup output and structured logs.
- Require `GATE_AUTH_TOKEN` in Docker Compose deployments.

### Release hardening

- Align Cargo, npm, Tauri, Docker, desktop UI, and release documentation on `0.9.1`.
- Remove unreachable placeholder crates, modules, commands, components, and mock runtime exports.
- Normalize updater IPC failures to the structured Gate application error contract.
- Remove synthetic tunnel metadata and display only Runtime-sourced traffic fields.

## 0.9.0 - Release Candidate

### Release engineering

- Reorganized public documentation into user, development, and internal references.
- Rebuilt the English and Chinese README files for open-source release readiness.
- Unified Cargo, npm, and Tauri versions at `0.9.0`.
- Prepared Tauri bundle targets for Windows, macOS, and Linux packages.
- Expanded GitHub Actions release automation for server binaries, desktop installers, Docker images, and draft GitHub Releases.
- Added root npm proxy scripts for `npm run typecheck` and `npm run build`.

### Cleanup

- Removed local build output, dependency caches, IDE files, logs, and empty placeholder resources from the release tree.
- Removed beta sprint reports, future-plan drafts, and one-off validation reports from public documentation.
- Renamed non-English resource filenames to release-safe English names.

### Compatibility

- No tunnel data-plane, TCP/HTTP/HTTPS runtime, protocol, database, or business-logic changes are included in this release cleanup.
