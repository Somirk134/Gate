# Coding Style

Gate follows the existing style of each workspace area.

## Rust

- Use `cargo fmt` for formatting.
- Prefer small modules with explicit ownership boundaries.
- Return typed errors where callers can act on them.
- Keep protocol and runtime changes isolated from release engineering changes.

## TypeScript and Vue

- Use TypeScript for service and state code.
- Keep UI components focused on presentation and user interaction.
- Prefer composables for shared UI behavior.
- Run `npm --prefix client run typecheck` before publishing a pull request.

## Documentation

- Use one H1 per page.
- Prefer task-oriented pages for users.
- Keep development notes under `docs/development/`.
- Keep protocol/runtime internals under `docs/internals/`.
- Do not include secrets, private hostnames, or local-only paths in examples.
