# Monitoring Internals

Gate exposes operational state through dashboard data, logs, health checks, and runtime statistics.

## Areas

- Server health check used by Docker and CI smoke checks.
- Desktop dashboard and log center views.
- Runtime statistics for tunnel status and traffic visibility.

## Maintainer guidance

Keep monitoring data stable for users. When adding new metrics later, prefer additive fields and document them in user-facing release notes.
