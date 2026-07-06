# Documentation Style

This page defines the Markdown style for Gate.

## Titles

- Use one `#` H1 per file.
- Use title case for page titles.
- Use sentence case for section headings when it reads more naturally.
- Do not skip heading levels.

## Emoji

- Do not use emoji in documentation headings.
- Avoid emoji in body text unless the page is explicitly community-facing and the tone benefits from it.
- Never use emoji to communicate status that is not also written in text.

## Images

- Store public screenshots in `assets/screenshots`.
- Use descriptive alt text.
- Keep screenshot dimensions consistent.
- Do not include secrets, private IPs, real tokens, or customer data.

## Code Blocks

- Always include a language label.
- Use `bash` for Unix shell commands.
- Use `powershell` for Windows PowerShell commands.
- Use `toml`, `json`, `yaml`, or `text` for config and output.

## Commands

- Prefer commands that can be copied directly.
- Do not chain unrelated commands in one block.
- Keep destructive commands out of quick-start docs.
- Explain environment variables before using them.

## English And Chinese Spacing

- Add spaces around English words inside Chinese text.
- Add spaces around inline code in Chinese text.
- Keep product names unchanged.
- Prefer English as the canonical public docs language and keep Chinese README concise.

## Links

- Use relative links inside the repository.
- Keep public docs filenames lowercase kebab-case.
- Use stable link text such as `Quick Start`, `Docker`, and `Troubleshooting`.

## Tables

- Use tables for short comparisons and configuration matrices.
- Keep cells concise.
- Avoid large prose paragraphs inside tables.
