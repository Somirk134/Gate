# Security Policy

## Supported Versions

| Version | Supported          |
|---------|-------------------|
| 0.1.x   | :white_check_mark: |

## Reporting a Vulnerability

Gate is a network tunneling tool that handles sensitive connection data. Security is our top priority.

**Please do not report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to **security@gate-project.dev** (placeholder — will be updated before public release).

You should receive a response within 48 hours. If you don't, please follow up.

### What to include

- Type of vulnerability
- Full reproduction steps
- Impact assessment
- Suggested fix (if any)

### Scope

- The Gate server and client binaries
- The tunnel protocol implementation
- Authentication and authorization mechanisms
- Configuration and secret storage

## Disclosure Policy

We follow coordinated disclosure:

1. Report received and acknowledged
2. Investigation and fix development
3. Release and public disclosure

## Security Considerations

- Always use TLS in production deployments
- Rotate authentication tokens regularly
- Gate does not log tunnel payload content by default
- Network traffic through tunnels is unencrypted by default; use application-level encryption for sensitive data
