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

Most day-to-day development entry points are exposed as root npm scripts:

```bash
npm run dev:server
npm run dev:client
npm run dev:desktop
npm run build:client
npm run check:server
```

Windows local server helper:

```powershell
$env:GATE_AUTH_TOKEN = [Convert]::ToHexString([Security.Cryptography.RandomNumberGenerator]::GetBytes(32)).ToLower()
npm run dev:server:local
npm run dev:server:local -- -Addr "127.0.0.1:7001"
```

JetBrains IDEs can use the shared `Gate Server Local` run configuration from `.run/`.

Other useful scripts:

```bash
# Linux install helper
bash scripts/install.sh

# Windows icon generation
powershell -ExecutionPolicy Bypass -File scripts/build/icons.ps1
```
