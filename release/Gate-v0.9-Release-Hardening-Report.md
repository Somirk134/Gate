# Gate v0.9 Release Hardening Report

- Report date: 2026-07-10
- Release: Gate v0.9.0 Beta candidate
- Decision: GO for Beta after final artifact signing/policy and native installer smoke gate
- Protected areas: Tunnel Data Plane, Protocol, Gateway core logic, and Runtime architecture were not changed

## 1. Cross-Platform Status

| Platform | Bundle | Status | Notes |
| --- | --- | --- | --- |
| Windows x64 | NSIS | PASS | Current-user install, Chinese/English selector, WebView2 bootstrapper, installer/uninstaller icon |
| macOS x64 | DMG | CONFIG PASS | macOS 10.15 minimum, DMG layout, x64 CI target, ICNS asset added |
| macOS arm64 | DMG | CONFIG PASS | arm64 CI target and DMG artifact path configured |
| Linux x64 | AppImage | CONFIG PASS | AppImage target and artifact path configured |
| Linux x64 | deb | CONFIG PASS | WebKit GTK, GTK3, AppIndicator, and librsvg dependencies declared |

The release workflow has explicit Windows, macOS x64, macOS arm64, and Linux jobs and uploads NSIS, DMG, AppImage, and deb artifacts. Windows release compilation passed locally with `tauri build --no-bundle`. macOS/Linux bundle configuration was statically validated on Windows; fresh native CI artifact generation and install smoke tests remain the publication gate.

Platform data paths are unified:

- Windows: `%APPDATA%\Gate` with `%LOCALAPPDATA%` fallback
- macOS: `~/Library/Application Support/Gate`
- Linux: `$XDG_DATA_HOME/Gate` or `~/.local/share/Gate`
- Config: `<data-dir>/client-config.json`
- Runtime/database: `<data-dir>/client-runtime.json`, `projects.sqlite3`, `domains.sqlite3`
- Certificates: `<data-dir>/certificates`, unless an explicit Gate certificate environment override is set
- Backup: user-selected `.gatebackup` path; default fallback is the application data directory

## 2. Security Scan Result

Status: PASS.

- No private key or credential artifact was found in the release worktree.
- Logger and global error notifications redact sensitive fields and private-key text.
- UI no longer embeds or reveals a shared default server token.
- Deployment examples use the real server variables `GATE_AUTH_TOKEN` and `GATE_SERVER_ADDR`.
- Tauri CSP is enabled.
- `.gatebackup` excludes server tokens, private keys, certificate PEM, secrets, passwords, logs, session IDs, secret project variables, and project notes.

Detailed results: `release/Gate-v0.9-Security-Audit-Report.md`.

## 3. Dependency Risk

- Production dependencies: PASS, 0 vulnerabilities.
- Fixed: `vue-i18n`/`@intlify` DOM-XSS advisory via patch upgrade to 9.14.5.
- Development dependencies: 1 high and 1 moderate advisory in Vite/esbuild development-server behavior.
- No automatic `npm audit fix` was run.
- Vite 8 migration is deferred because the available fix is a breaking major upgrade; packaged production assets are not affected.

## 4. Performance Optimization

Vite manual chunks now separate framework, UI, and other vendor dependencies.

| Chunk | Before | After |
| --- | ---: | ---: |
| Entry JS | 733.46 kB | 218.70 kB |
| Framework | Included in entry | 56.62 kB |
| UI | Included in entry | 249.75 kB |
| Vendor | Included in entry | 182.61 kB |

The entry chunk decreased by 514.76 kB (70.2%). The production build has no chunk-size warning. The split improves parallel loading and cache reuse without changing route behavior.

## 5. First-User Flow Test

No-server path:

- Application opens and renders the first-run guide.
- The user can choose "no server", read the public-server explanation, review provider options, and continue later.
- Docker guidance uses safe placeholders and the correct server environment variables.

Existing-server path:

- The user can choose a server environment, enter host, port, and a masked token, review a deployment command, and reach connection testing.
- Server management exposes a complete add/edit form and does not prefill a shared token.
- Dashboard and server pages provide reachable Tunnel creation and server connection actions.
- Browser UI validation stops before native IPC submission because a normal browser does not provide the Tauri Rust backend.
- Native success coverage passed for authentication, real protocol connection, bidirectional forwarding, server/client restart recovery, local HTTP runtime, and Tunnel listener creation.

Desktop browser inspection found no console error or incoherent layout overlap. Preview URL: `http://127.0.0.1:1420/`.

## 6. Final Verification

| Command | Result |
| --- | --- |
| `cargo check --workspace` | PASS |
| `cargo test --workspace` | PASS: 105 passed, 0 failed, 6 explicitly ignored long-running stress cases |
| `npm run typecheck` | PASS |
| `npm run build` | PASS |
| `npm audit --omit=dev` | PASS: 0 vulnerabilities |
| `tauri build --no-bundle` | PASS: optimized Windows executable built |

## Release Recommendation

Gate v0.9 meets the code, dependency, backup, performance, and automated functional quality bar for public Beta.

Before publishing artifacts:

1. Run the release workflow from the final commit and require all four platform artifact jobs to pass.
2. Smoke-install NSIS, both DMGs, AppImage, and deb on native clean machines/VMs.
3. Sign Windows artifacts and sign/notarize macOS artifacts, or explicitly document the unsigned Beta policy.
4. Publish SHA-256 checksums and keep the Vite development server inaccessible from untrusted networks.

After these operational gates, the release recommendation is GO.
