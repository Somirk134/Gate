# Documentation Style

Gate documentation is written in Markdown and published with VitePress.

## Formatting

- Use sentence-case headings.
- Keep pages task-oriented.
- Prefer short examples over long prose.
- Use tables for configuration and compatibility.
- Use Mermaid for diagrams.
- Reserve PlantUML for future architecture packs.
- Do not use emoji icons in documentation, templates, guides, or README files.

## File Naming

- Public docs use lowercase kebab-case.
- README files are allowed for directory landing pages.
- Avoid spaces in file names.

## Code Blocks

Always provide language hints:

```bash
cargo test --workspace
```

```toml
[server]
bind = "0.0.0.0:5800"
```

## Mermaid

Keep diagrams compact and readable in dark mode.
