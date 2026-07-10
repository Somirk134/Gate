# Changelog

All notable changes to Gate are documented in this file.

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
