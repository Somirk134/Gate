# Contributing

Thanks for contributing to Gate. Start with the root [CONTRIBUTING.md](../../CONTRIBUTING.md), then use this page for local workflow details.

## Local setup

```bash
git clone https://github.com/Somirk134/Gate.git
cd Gate
npm --prefix client ci
cargo check --workspace
npm run typecheck
```

## Development commands

| Command | Purpose |
| --- | --- |
| `npm run dev:server` | Start the local server. |
| `npm run dev:client` | Start the frontend dev server. |
| `npm run dev:desktop` | Start the Tauri desktop app. |
| `cargo test --workspace` | Run Rust tests. |
| `npm run typecheck` | Run client type checking from the repository root. |
| `npm run build` | Build the client from the repository root. |

## Pull request checklist

- Keep changes scoped and easy to review.
- Do not mix release cleanup with runtime behavior changes.
- Add or update documentation when user-facing behavior changes.
- Run the relevant checks before opening a pull request.
- Remove local logs, build output, and secrets.
