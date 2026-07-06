# Gate Client Components

This document defines the component organization and usage rules for the Gate desktop client.

## Goals

- Keep component responsibilities clear.
- Reuse base components before adding page-specific UI.
- Keep visual tokens centralized.
- Keep business data loading outside presentational components.
- Keep icon usage consistent through the icon registry.

## Directory Structure

```text
client/src/components/
  base/
    GButton.vue
    GIconButton.vue
    GCard.vue
    GBadge.vue
  icons/
    GIcon.vue
    registry.ts
  form/
    GInput.vue
    GPasswordInput.vue
    GNumberInput.vue
    GPortInput.vue
    GHostInput.vue
    GTokenInput.vue
    GSearchInput.vue
    GTextarea.vue
    GLabel.vue
    GFormField.vue
  feedback/
    GSpinner.vue
    GSkeleton.vue
    GProgress.vue
    GCircleProgress.vue
    GEmptyState.vue
    GErrorState.vue
  status/
    GStatusDot.vue
    GStatusBadge.vue
  layout/
    GPageContainer.vue
    GPageHeader.vue
    GSectionHeader.vue
  cards/
    GStatCard.vue
    GActionCard.vue
  business/
    ProjectCard.vue
    ServerCard.vue
    TunnelCard.vue
    StatisticsCard.vue
```

## Design Tokens

Use variables from `client/src/styles/tokens.css`.

| Area | Rule |
| --- | --- |
| Color | Use semantic tokens such as primary, success, warning, danger, and muted |
| Typography | Use shared UI and mono font tokens |
| Spacing | Use the shared spacing scale |
| Radius | Use the shared radius scale |
| Shadow | Use tokenized surface shadows |
| Motion | Use shared duration and easing tokens |
| Z index | Use shell, overlay, popover, modal, and toast layers |

Do not hard-code colors, shadows, spacing, or animation durations in page components.

## Base Components

| Component | Usage | Reuse Priority |
| --- | --- | --- |
| `GButton` | Standard text buttons with variants, sizes, loading, disabled, and icon states | High |
| `GIconButton` | Square icon-only actions for toolbars and inline controls | High |
| `GCard` | Reusable card foundation | High |
| `GBadge` | Status, protocol, count, and metadata labels | High |

## Icon Components

| Component | Usage |
| --- | --- |
| `GIcon` | Unified icon wrapper for size, color, spin, and disabled states |
| `registry.ts` | Central registry for icon names and Lucide components |

Rules:

- Add new icons through `registry.ts`.
- Do not inline SVG in page components.
- Prefer clear tooltips for icon-only actions.

## Form Components

| Component | Usage |
| --- | --- |
| `GInput` | Text input foundation |
| `GPasswordInput` | Password input |
| `GNumberInput` | Number input |
| `GPortInput` | Port input |
| `GHostInput` | Host input |
| `GTokenInput` | Token input |
| `GSearchInput` | Search input |
| `GTextarea` | Multi-line text input |
| `GLabel` | Form label |
| `GFormField` | Field layout, hint, and error boundary |

## Feedback Components

| Component | Usage |
| --- | --- |
| `GSpinner` | Loading indicator |
| `GSkeleton` | Loading placeholder |
| `GProgress` | Linear progress |
| `GCircleProgress` | Circular progress |
| `GEmptyState` | Empty state |
| `GErrorState` | Error state |

## Status Components

| Component | Usage |
| --- | --- |
| `GStatusDot` | Compact status indicator |
| `GStatusBadge` | Text status badge |

## Layout Components

| Component | Usage |
| --- | --- |
| `GPageContainer` | Page width and spacing shell |
| `GPageHeader` | Page title, description, and actions |
| `GSectionHeader` | Section title and actions |

## Card Variants

| Component | Usage |
| --- | --- |
| `GStatCard` | Numeric statistic card |
| `GActionCard` | Shortcut action card |

## Business Components

Business components compose base components and receive data from parent views. They should not own
fetching, persistence, or global state.

| Component | Composition | Usage |
| --- | --- | --- |
| `ProjectCard` | `GCard`, `GIcon`, `GStatusBadge`, `GStatusDot`, `GIconButton` | Project overview |
| `ServerCard` | `GCard`, `GStatusBadge`, `GProgress`, `GIconButton` | Server overview |
| `TunnelCard` | `GCard`, `GBadge`, `GStatusBadge`, `GIcon`, `GIconButton` | Tunnel overview |
| `StatisticsCard` | `GStatCard`, chart slot | Dashboard statistics |

## Usage Example

```vue
<template>
  <GCard variant="outlined">
    <GSectionHeader title="Tunnel" />
    <GStatusBadge status="online">Online</GStatusBadge>
    <GButton variant="primary" :loading="saving">Save</GButton>
  </GCard>
</template>
```

## Rules

1. Use token variables for colors, typography, spacing, radius, shadows, and motion.
2. Use `GIcon` and `registry.ts` for icons.
3. Use `GButton` and `GIconButton` for actions.
4. Use feedback composables for notification and confirmation flows.
5. Build business cards from base components before adding new containers.
6. Keep presentational components stateless whenever possible.
7. Document new reusable components in this file.
