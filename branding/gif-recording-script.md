# GIF Recording Script

The launch GIF should show the full first-run path in 30 seconds or less.

## Output

| Item | Value |
| --- | --- |
| Duration | 25 to 30 seconds |
| Size | 1440x900 source, optimized for README |
| Format | GIF for README, MP4/WebM for website |
| File | `assets/demo/gate-first-tunnel.gif` |

## Timeline

| Time | Action | Screen |
| --- | --- | --- |
| 0s-3s | Download or clone repository | Terminal |
| 3s-6s | Start `gate-server` | Terminal |
| 6s-9s | Open desktop client | Welcome Wizard |
| 9s-12s | Connect to server | Connection Wizard |
| 12s-16s | Create Tunnel | Tunnel Wizard |
| 16s-20s | Start Tunnel | Tunnel Workspace |
| 20s-24s | View realtime traffic | Dashboard |
| 24s-27s | Inspect logs | Log Center |
| 27s-30s | Stop Tunnel | Tunnel Workspace |

## Shot List

### 1. Download And Start

```bash
git clone https://github.com/Somirk134/Gate.git
cd gate
GATE_SERVER_ADDR=127.0.0.1:7000 GATE_AUTH_TOKEN=gate-alpha-token cargo run -p gate-server
```

### 2. Connect Server

- Open Welcome Wizard.
- Choose `Local web app`.
- Add server `127.0.0.1:7000`.
- Enter token.
- Show success state.

### 3. Create Tunnel

- Name: `local-web`.
- Protocol: `TCP`.
- Local: `127.0.0.1:3000`.
- Remote: `18080`.
- Project: `Platform Lab`.

### 4. Start And Observe

- Click start.
- Show status changing to `Running`.
- Switch to Dashboard.
- Show traffic chart moving.
- Open Log Center.
- Filter by `local-web`.

### 5. Stop

- Return to Tunnel Workspace.
- Stop `local-web`.
- End on a clean running/stopped state.

## Editing Notes

- No narration text over the app.
- Cursor movement should be deliberate and slow enough to follow.
- Crop nothing important.
- Keep terminal text large.
- Do not record real tokens or private hostnames.
