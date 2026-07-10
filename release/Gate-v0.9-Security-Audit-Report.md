# Gate v0.9 Security Audit Report

- Audit date: 2026-07-10
- Scope: release worktree, desktop client, Tauri commands, backup, onboarding, logs, UI, npm dependencies
- Excluded generated trees: `.git`, `node_modules`, `target`, `dist`, Tauri generated schemas
- Result: PASS with accepted development-only dependency risk

## Sensitive Data Scan

Scanned `token`, `password`, `secret`, `private_key`, `pem`, and `certificate` references, hard-coded assignments, key files, logging calls, UI interpolation, and backup serialization paths.

- No committed private key, PEM key, P12, PFX, or `.gatebackup` file was found.
- No production credential or private-key block was found in the release worktree.
- Known `gate-alpha-token` values are confined to tests and the loopback-only development launcher. They are public test fixtures, not release credentials.
- The development launcher no longer prints the configured token value.
- Server token values are masked in server details and password inputs.
- Onboarding no longer embeds a shared default token. Deployment commands display `[hidden]` after a token is entered; the real value is used only by the explicit copy action.
- Frontend logger data, error messages, Bearer tokens, token/password/secret/private-key fields, and private-key PEM blocks are redacted before reaching a sink.
- Global error notifications apply the same text redaction before rendering in the UI.
- Tauri CSP is enabled. Scripts are restricted to `self`; IPC and required HTTPS connections are explicitly allowed.

## Backup Security

The `.gatebackup` contract was reviewed and hardened.

Included:

- Sanitized runtime snapshot
- Projects and sanitized project SQLite database
- Domain metadata/database
- Certificate metadata
- Non-sensitive settings/runtime configuration

Excluded or cleared:

- Server tokens
- Password and secret fields
- Private keys and key PEM
- Certificate PEM payloads
- Runtime logs and session IDs
- Project variables marked `secret`
- Project variables whose names contain token/password/secret/private-key patterns
- Free-text project notes

Certificate export reads only `metadata.json`; it never reads `private_key.pem` or `certificate.pem`. Four focused tests cover recursive runtime redaction, project JSON redaction, certificate metadata redaction, and decoded SQLite backup contents.

## Dependency Risk

- Production audit: 0 vulnerabilities (`npm audit --omit=dev`).
- `vue-i18n` and `@intlify` were upgraded from 9.14.4 to 9.14.5, resolving the production DOM-XSS advisory.
- Full audit: 1 high and 1 moderate vulnerability remain in Vite/esbuild development-server tooling.
- The remaining fix requires a Vite 5 to Vite 8 major upgrade. It is not applied in v0.9 because it is breaking and does not affect packaged production assets.
- Acceptance condition: do not expose the Vite development server to untrusted networks.

## Conclusion

No known production secret, UI/log credential disclosure, or backup credential leak remains in the reviewed release surface. Security status is suitable for public Beta, subject to normal artifact signing, checksum publication, and native-platform smoke tests.
