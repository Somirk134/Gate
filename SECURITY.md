# Security Policy

Gate is a tunnel and remote access project. Security reports are handled with higher urgency than
ordinary bugs.

## Supported Versions

See [SUPPORTED_VERSIONS.md](./SUPPORTED_VERSIONS.md).

## Responsible Disclosure

Please do not open public issues for vulnerabilities.

Report privately through one of these channels:

- GitHub Security Advisories for this repository.
- Email: [security@example.com](mailto:security@example.com)

Include:

- Affected version or commit.
- Reproduction steps.
- Impact assessment.
- Logs, packet captures, or proof of concept when safe to share.
- Whether the report is already disclosed elsewhere.

## Response Targets

| Stage | Target |
| --- | --- |
| Initial acknowledgement | 3 business days |
| Triage result | 7 business days |
| Fix plan for confirmed high severity issues | 14 business days |
| Coordinated disclosure | After patched release or agreed date |

## Dependency Policy

- Dependabot monitors Cargo, npm, GitHub Actions, and Docker dependencies.
- Security updates are prioritized above ordinary dependency upgrades.
- Runtime dependency changes should include a short risk note in the pull request.
- New dependencies must have an active upstream, clear license, and reasonable maintenance history.

## Cryptography Policy

Cryptography policy is reserved. Until published:

- Prefer well-reviewed libraries over custom cryptographic code.
- Do not implement custom token signing, password hashing, or encryption primitives.
- Document key and token handling for any security-sensitive change.

## Security Baseline

- CodeQL runs on Rust, TypeScript, and GitHub Actions.
- CI enforces formatting, linting, type checks, and tests.
- Docker images should avoid embedding secrets and should use explicit runtime configuration.
