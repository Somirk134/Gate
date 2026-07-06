# Screenshot Guidelines

This guide keeps every public Gate screenshot visually consistent.

## Required Screenshots

| Screenshot | File |
| --- | --- |
| Home Hero | `assets/screenshots/hero.svg` |
| Dashboard | `assets/screenshots/dashboard.svg` |
| Tunnel | `assets/screenshots/tunnel.svg` |
| Project | `assets/screenshots/project.svg` |
| Server | `assets/screenshots/server.svg` |
| Log Center | `assets/screenshots/log-center.svg` |
| Settings | `assets/screenshots/settings.svg` |
| Welcome Wizard | `assets/screenshots/welcome-wizard.svg` |
| Connection Wizard | `assets/screenshots/connection-wizard.svg` |

## Frame

- Export size: `1440x900`.
- Keep the app frame centered.
- Use `28px` outer radius.
- Use consistent shadow and background.
- Avoid cropped text.
- Avoid showing secrets, tokens, private IPs, customer names, or real payloads.

## Content Rules

- Use realistic but fake data.
- Prefer `127.0.0.1`, `gate.example.com`, and `example` domains.
- Keep labels short.
- Show one primary workflow per screenshot.
- Do not mix languages in one screenshot unless demonstrating i18n.

## Replacement Process

1. Capture the real app at `1440x900`.
2. Redact secrets and private values.
3. Place it in the same screenshot frame.
4. Replace the matching file in `assets/screenshots`.
5. Update README if the navigation or layout changed.

## Quality Checklist

- Text is readable at GitHub README size.
- Main UI state is obvious in under five seconds.
- No overlapping labels.
- No placeholder tokens or real secrets.
- Screenshot file name matches the documented surface.
