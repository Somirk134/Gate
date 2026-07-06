# Naming Conventions

## Directories

| Area | Convention | Example |
| --- | --- | --- |
| Top-level ecosystem directories | lowercase words | `docs`, `docker`, `community` |
| Example directories | lowercase kebab-case | `multi-tunnel` |
| Documentation directories | lowercase kebab-case | `getting-started` |
| Rust crates | lowercase kebab-case package names | `gate-server` |
| TypeScript modules | camelCase or PascalCase by local pattern | `registerApplicationServices.ts` |

## Markdown

- Public docs use lowercase kebab-case: `quick-start.md`.
- Root governance files use GitHub standard uppercase names: `README.md`, `SECURITY.md`.
- Directory landing pages use `README.md`.

## GitHub

| Asset | Convention |
| --- | --- |
| Workflows | lowercase kebab-case, e.g. `markdown-lint.yml` |
| Issue templates | lowercase snake-case or kebab-case forms |
| Labels | lowercase words, e.g. `needs-triage`, `good first issue` |
| Branches | `codex/<topic>` for automation work, feature branches by maintainer preference |
| Tags | `vMAJOR.MINOR.PATCH`, e.g. `v1.0.0` |

## Releases

- Pre-1.0 releases may use `v0.x.y`.
- Stable releases use semantic versioning.
- Release titles use `Gate vX.Y.Z`.

## Diagrams

- Mermaid is the default.
- PlantUML is reserved for future architecture packs.
- Diagram labels should be short and readable in dark mode.
