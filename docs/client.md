# Client

The Gate desktop client is the operator workspace for servers, projects, tunnels, logs, and settings.

## Run In Development

```bash
cd client
npm install
npm run tauri dev
```

For frontend-only work:

```bash
npm run dev
```

## Build

```bash
npm run typecheck
npm run build
```

Tauri packaging:

```bash
npm run tauri build
```

## Main Areas

| Area | Purpose |
| --- | --- |
| Welcome Wizard | First-run setup and first tunnel creation |
| Dashboard | Runtime health, traffic, and quick actions |
| Projects | Group tunnels by product, environment, or team |
| Tunnels | Create, start, stop, inspect, and monitor tunnels |
| Servers | Manage self-hosted Gate endpoints |
| Log Center | Search, filter, inspect, and export logs |
| Settings | Appearance, network, security, update, and developer preferences |

## Client Configuration

The client keeps local workspace state and preferences on the desktop machine. Do not commit local app state, tokens, exported logs, or generated diagnostics.

## Expected First-Run Flow

1. Open Welcome Wizard.
2. Choose a use case.
3. Add a Gate server address and token.
4. Create a first tunnel.
5. Review Dashboard and Log Center.

## Maintainer Notes

- Keep UI text short and action-oriented.
- Keep mock data isolated from runtime services.
- Update screenshots when primary UI navigation changes.
- Document every user-visible workflow in this docs directory.
