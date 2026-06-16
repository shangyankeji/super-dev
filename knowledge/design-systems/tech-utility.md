---
id: tech-utility
title: Tech Utility
domain: design-systems
category: tech-utility.md
difficulty: intermediate
tags: [color, component, design-systems, motion, palette, patterns, spacing, tech]
quality_score: 70
last_updated: 2026-06-15
---
# Tech Utility

> Dense, monospace accents, dark-mode-native. Inspired by GitHub, Datadog, Grafana.

## When to use

CLI companions, code platforms, monitoring dashboards, data tools, developer-facing products. Products where information density is a feature, not a bug.

## Color palette

```css
:root {
  --color-bg: #0d1117;
  --color-surface: #161b22;
  --color-surface-elevated: #1c2128;
  --color-surface-sunken: #010409;
  --color-text: #e6edf3;
  --color-text-secondary: #8b949e;
  --color-text-tertiary: #6e7681;
  --color-primary: #58a6ff;
  --color-primary-hover: #79c0ff;
  --color-primary-muted: rgba(56, 139, 253, 0.15);
  --color-accent: #bc8cff;
  --color-success: #3fb950;
  --color-warning: #d29922;
  --color-error: #f85149;
  --color-border: #30363d;
  --color-border-hover: #484f58;
  --color-border-focus: #58a6ff;
  --shadow-sm: 0 0 0 1px var(--color-border);
  --shadow-md: 0 3px 12px rgb(1 4 9 / 0.4);
}

@media (prefers-color-scheme: light) {
  :root {
    --color-bg: #ffffff;
    --color-surface: #f6f8fa;
    --color-surface-elevated: #ffffff;
    --color-text: #1f2328;
    --color-text-secondary: #656d76;
    --color-primary: #0969da;
    --color-border: #d0d7de;
  }
}
```

## Typography

- **Headings**: `"Inter", -apple-system, sans-serif`, weight 600
- **Body**: `"Inter", -apple-system, sans-serif`, weight 400
- **Code / Data**: `"JetBrains Mono", "Fira Code", monospace`, weight 400

| Level | Size | Weight | Line-height | Use |
|---|---|---|---|---|
| h1 | 1.75rem (28px) | 600 | 1.25 | Page title (compact) |
| h2 | 1.25rem (20px) | 600 | 1.3 | Section header |
| h3 | 1rem (16px) | 600 | 1.4 | Panel title |
| body | 0.875rem (14px) | 400 | 1.5 | Default text |
| body-sm | 0.8125rem (13px) | 400 | 1.45 | Table cells, metadata |
| mono | 0.8125rem (13px) | 400 | 1.5 | Code, terminal output |
| caption | 0.75rem (12px) | 500 | 1.3 | Timestamps, status labels |

## Spacing

4px base: `4 / 8 / 12 / 16 / 24 / 32 / 48`

Tighter than other systems. Panels: 16px padding. Gap between panels: 8-12px.

## Component patterns

### Data table
- Monospace numbers right-aligned, text left-aligned
- Alternating row bg: transparent / surface-sunken
- Sticky header, sortable columns (chevron indicator)
- Compact row height: 36px

### Status badge
- Dot (8px circle) + label. Colors: success/warning/error/neutral
- No filled backgrounds — just colored dot + text

### Code block
- `bg-surface-sunken`, monospace, line numbers in text-tertiary
- Copy button top-right, language label top-left

## Motion

- `--transition-fast: 100ms ease` — everything. Tech UIs should feel instant.
- Minimal animation. No bounces, no spring physics.

## Do

- Smaller base font (14px). Dense but legible.
- Monospace for any data: timestamps, IDs, metrics, code.
- Dark mode as the PRIMARY mode (light is the override).
- Subtle borders over shadows. 1px borders everywhere.
- Tabular data in actual tables, not cards.

## Don't

- Large hero sections with marketing copy.
- Rounded card corners > 8px (keep it sharp).
- Colorful illustrations or decorative elements.
- More than 2 status colors visible at once per panel.
