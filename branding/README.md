# Gate Brand Guidelines

Gate should feel reliable, technical, calm, and operator-friendly. The brand is built for open source developers who want to understand the project quickly and trust its maintenance quality.

## Brand Promise

Self-hosted tunnel infrastructure for teams that want private service access without giving up operational control.

## Logo

| Asset | Path | Usage |
| --- | --- | --- |
| Primary logo | `assets/logo/logo.svg` | README, documentation, website, app about page |
| App icon | `assets/branding/app-icon.svg` | Desktop app icon source |
| Small icon | `assets/branding/icon.svg` | Favicons, small cards, social snippets |
| Banner | `assets/branding/banner.svg` | GitHub README sections and docs landing |
| Social card | `assets/branding/social-card.svg` | GitHub social preview |
| Open Graph | `assets/branding/open-graph.svg` | Website metadata |

## Usage Rules

- Keep the logo on high-contrast backgrounds.
- Do not stretch, rotate, skew, recolor, or add extra effects to the mark.
- Use the full logo for README and website hero areas.
- Use the icon only when space is constrained.
- Keep at least one icon width of clear space around the mark.

## Color Palette

| Token | Hex | Usage |
| --- | --- | --- |
| Gate Ink | `#0B1020` | App chrome, hero dark surface, terminal blocks |
| Gate Surface | `#F8FAFC` | Main app surface |
| Gate Canvas | `#EEF3F8` | Screenshot background |
| Gate Sky | `#0EA5E9` | Primary action, links, active nav |
| Gate Mint | `#10B981` | Healthy state, success |
| Gate Amber | `#F59E0B` | Warning, pending, attention |
| Gate Red | `#EF4444` | Error and destructive state |
| Gate Slate | `#64748B` | Secondary text |
| Gate Border | `#E2E8F0` | Borders and separators |

## Typography

Use this stack for web, docs, and screenshots:

```css
font-family: Inter, "Segoe UI", Arial, sans-serif;
```

Use this stack for terminal and logs:

```css
font-family: Consolas, "SFMono-Regular", "Liberation Mono", monospace;
```

## Screenshot Style

Screenshots live in `assets/screenshots` and follow:

- Size: `1440x900`.
- Outer background: `#EEF3F8`.
- App frame radius: `28`.
- Panel radius: `14` to `18`.
- Soft neutral shadow.
- Light product UI with dark app chrome.
- No random decorative blobs or unrelated illustrations.

See [screenshot-guidelines.md](./screenshot-guidelines.md).

## GitHub Style

GitHub surfaces should be consistent:

- Repository description: short and concrete.
- Topics: tunnel, self-hosted, rust, tauri, docker, networking, developer-tools.
- README first viewport: logo, one sentence, hero screenshot, badges, key links.
- Social Preview: `assets/branding/social-card.svg`.
- Release notes: use `.github/RELEASE_TEMPLATE.md`.

See [github-homepage.md](./github-homepage.md).

## Voice

- Clear before clever.
- Concrete before aspirational.
- State alpha limitations honestly.
- Prefer short operational sentences.
- Avoid unexplained acronyms and hype.
