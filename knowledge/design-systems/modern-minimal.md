---
id: modern-minimal
title: Modern Minimal
domain: design-systems
category: modern-minimal.md
difficulty: intermediate
tags: [borders, color, component, design-systems, minimal, modern, palette, patterns]
quality_score: 70
last_updated: 2026-06-15
---
# Modern Minimal

> Precise, geometric, whitespace-first. Inspired by Linear, Vercel, Raycast.

## When to use

SaaS products, developer tools, dashboards, productivity apps. Products where information density matters but visual noise must stay low.

## Color palette

```css
:root {
  /* Surface */
  --color-bg: #fafafa;
  --color-surface: #ffffff;
  --color-surface-elevated: #ffffff;
  --color-surface-sunken: #f4f4f5;

  /* Text */
  --color-text: #18181b;
  --color-text-secondary: #71717a;
  --color-text-tertiary: #a1a1aa;

  /* Brand */
  --color-primary: #2563eb;
  --color-primary-hover: #1d4ed8;
  --color-primary-muted: #dbeafe;

  /* Accent */
  --color-accent: #8b5cf6;

  /* Status */
  --color-success: #22c55e;
  --color-warning: #f59e0b;
  --color-error: #ef4444;

  /* Border */
  --color-border: #e4e4e7;
  --color-border-hover: #d4d4d8;
  --color-border-focus: #2563eb;

  /* Shadow */
  --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.05);
  --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.07), 0 2px 4px -2px rgb(0 0 0 / 0.05);
  --shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.08), 0 4px 6px -4px rgb(0 0 0 / 0.04);
}

@media (prefers-color-scheme: dark) {
  :root {
    --color-bg: #09090b;
    --color-surface: #18181b;
    --color-surface-elevated: #27272a;
    --color-surface-sunken: #09090b;
    --color-text: #fafafa;
    --color-text-secondary: #a1a1aa;
    --color-text-tertiary: #71717a;
    --color-primary-muted: #1e3a5f;
    --color-border: #27272a;
    --color-border-hover: #3f3f46;
    --shadow-sm: 0 1px 2px 0 rgb(0 0 0 / 0.3);
    --shadow-md: 0 4px 6px -1px rgb(0 0 0 / 0.4);
    --shadow-lg: 0 10px 15px -3px rgb(0 0 0 / 0.5);
  }
}
```

## Typography

- **Headings**: `Inter, -apple-system, BlinkMacSystemFont, sans-serif`, weight 600
- **Body**: `Inter, -apple-system, BlinkMacSystemFont, sans-serif`, weight 400
- **Code**: `JetBrains Mono, Menlo, monospace`, weight 400

| Level | Size | Weight | Line-height | Letter-spacing | Use |
|---|---|---|---|---|---|
| h1 | 2.25rem (36px) | 700 | 1.2 | -0.025em | Page title |
| h2 | 1.5rem (24px) | 600 | 1.3 | -0.02em | Section header |
| h3 | 1.25rem (20px) | 600 | 1.4 | -0.015em | Card title |
| body-lg | 1.125rem (18px) | 400 | 1.6 | 0 | Hero subtitle |
| body | 1rem (16px) | 400 | 1.5 | 0 | Default text |
| body-sm | 0.875rem (14px) | 400 | 1.5 | 0 | Secondary text |
| caption | 0.75rem (12px) | 500 | 1.4 | 0.02em | Labels, badges |

## Spacing

4px base grid: `4 / 8 / 12 / 16 / 20 / 24 / 32 / 40 / 48 / 64 / 80 / 96`

## Borders & radius

- Default radius: `8px`
- Small radius: `6px` (badges, tags)
- Large radius: `12px` (cards, modals)
- Full radius: `9999px` (pills, avatars)
- Border width: `1px`

## Component patterns

### Buttons
- Primary: `bg-primary text-white`, hover darkens 10%, active darkens 15%, disabled opacity 0.5
- Secondary: `bg-transparent border-default text-primary`, hover bg-surface-sunken
- Ghost: no border, hover bg-surface-sunken
- Height: 36px (sm), 40px (md), 44px (lg)
- Padding: 12px horizontal (sm), 16px (md), 20px (lg)
- Font: body-sm weight 500

### Cards
- `bg-surface border-default radius-lg shadow-sm`
- Hover: `shadow-md border-hover`
- Padding: 20px (compact), 24px (default)

### Inputs
- Height: 40px
- `bg-surface border-default radius-md`
- Focus: `border-focus shadow(0 0 0 3px primary-muted)`
- Error: `border-error`

## Motion

- `--transition-fast: 150ms cubic-bezier(0.4, 0, 0.2, 1)` — hover, focus
- `--transition-normal: 200ms cubic-bezier(0.4, 0, 0.2, 1)` — expand, collapse
- `--transition-slow: 300ms cubic-bezier(0.4, 0, 0.2, 1)` — modals, drawers

## Do

- Let whitespace do the work. 24-32px between sections minimum.
- One accent color, used at most 2x per screen.
- Subtle borders over heavy shadows.
- Monochrome icons (Lucide stroke width 1.5).

## Don't

- Purple/pink gradient hero backgrounds.
- More than 2 font weights per page.
- Shadows heavier than `shadow-md` on cards.
- Icon + text + icon on every row.
- Rounded everything (keep some sharp edges for contrast).
