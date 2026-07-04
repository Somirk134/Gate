# Scripts

Development, build, release, and automation scripts for the Gate project.

## Directories

| Directory | Purpose |
|-----------|---------|
| `dev/` | Development environment setup and local dev scripts |
| `build/` | Build scripts for server and client |
| `release/` | Release packaging and signing scripts |
| `docker/` | Docker image build and management scripts |
| `ci/` | CI helper scripts (called by GitHub Actions) |
| `init/` | Project initialization scripts |

## Usage

```bash
# Development
./scripts/dev/setup.sh
./scripts/dev/start-server.sh
./scripts/dev/start-client.sh

# Build
./scripts/build/server.sh
./scripts/build/client.sh

# Docker
./scripts/docker/build.sh
./scripts/docker/run.sh

# Release
./scripts/release/tag.sh v0.1.0
```
