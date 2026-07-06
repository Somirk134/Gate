# FAQ

## Why choose Gate?

Choose Gate when you want a self-hosted tunnel foundation with a desktop operator workflow, Rust codebase, Docker deployment path, and documentation-first project maintenance.

## How is Gate different from FRP?

FRP is a mature and widely used tunnel proxy. Gate is younger and focuses on:

- Desktop-first management.
- Rust workspace architecture.
- Project, server, tunnel, dashboard, log, and settings surfaces.
- Open source maintainability assets from the start.

Gate is not yet a production-grade drop-in replacement for FRP.

## Does Gate support Docker?

Yes. Use [Docker](./docker.md) or [examples/docker](../examples/docker).

## Does Gate support HTTPS?

Use a reverse proxy for HTTPS termination today. Native HTTPS behavior should be documented before it is presented as stable.

## Does Gate support self-hosting?

Yes. Self-hosting is a primary project goal. Run `gate-server` on your own machine, VPS, homelab, or private cloud.

## Is Gate free?

Yes. Gate is open source under the MIT License.

## How do I upgrade?

Read [Upgrade](./upgrade.md) and the release note for the version you are installing. Back up configuration and tokens first.

## How do I back up Gate?

For alpha deployments, back up:

- Server environment variables.
- Compose files or process manager files.
- Client workspace exports when available.
- Tokens in your secret manager.

Do not store tokens in the repository.

## How do I migrate?

For now, migration means recreating server configuration and tunnel definitions in the target environment. A structured export/import workflow should be documented before 1.0.

## Can I use Gate for payment callbacks?

Yes, as a development or controlled testing workflow. See the webhook and framework examples in [examples](../examples/README.md).

## Is Gate production-ready?

Gate is alpha. Use it carefully in controlled environments until v1 stability criteria are complete.
