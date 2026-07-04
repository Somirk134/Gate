# Deployment Documentation

## Purpose

Deployment documentation provides guides for deploying Gate in various
environments.

## Contents

- **QuickStart.md** — One-command deployment guide
- **Docker.md** — Docker deployment guide
- **Kubernetes.md** — Kubernetes deployment guide
- **Systemd.md** — Linux systemd service setup
- **WindowsService.md** — Windows service setup
- **Networking.md** — Network configuration (firewall, DNS, reverse proxy)
- **TLS.md** — TLS certificate setup
- **Backup.md** — Backup and restore procedures

## Deployment Options

| Method | Use Case |
|--------|----------|
| Docker Compose | Self-hosted single server |
| Kubernetes | Production cluster deployment |
| Binary | Direct installation on VM/bare metal |

## Why This Design

Multiple deployment options ensure that Gate can be deployed in any
environment from a developer's laptop to a production Kubernetes cluster.
Docker is the recommended approach for most users.

## Extension

Add deployment guides for new environments as they are supported.
Maintain the Docker Compose and Helm chart configurations.
