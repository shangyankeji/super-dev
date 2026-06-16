---
id: editorial-clean
title: Editorial Clean
domain: design-systems
category: editorial-clean.md
difficulty: intermediate
tags: [clean, color, component, design-systems, editorial, motion, palette, patterns]
quality_score: 70
last_updated: 2026-06-15
---
# Editorial Clean

> Magazine-like, serif-accent headings, photography-driven. Inspired by Medium, Substack, The Verge.

## When to use

Content sites, blogs, portfolios, documentation, news/media products. Products where reading experience is the primary value.

## Color palette

```css
:root {
  --color-bg: #fefdfb;
  --color-surface: #ffffff;
  --color-surface-elevated: #ffffff;
  --color-surface-sunken: #f7f5f0;
  --color-text: #1a1a1a;
  --color-text-secondary: #666666;
  --color-text-tertiary: #999999;
  --color-primary: #c0392b;
  --color-primary-hover: #a93226;
  --color-primary-muted: #fce4e1;
  --color-accent: #2c3e50;
  --color-success: #27ae60;
  --color-warning: #f39c12;
  --color-error: #e74c3c;
  --color-border: #e8e4dd;
  --color-border-hover: #d5d0c8;
  --color-border-focus: #c0392b;
  --shadow-sm: 0 1px 3px rgb(0 0 0 / 0.04);
  --shadow-md: 0 4px 12px rgb(0 0 0 / 0.06);
}

@media (prefers-color-scheme: dark) {
  :root {
    --color-bg: #1a1a1a;
    --color-surface: #242424;
    --color-surface-sunken: #141414;
    --color-text: #eeeeee;
    --color-text-secondary: #aaaaaa;
    --color-border: #333333;
  }
}
```

## Typography

- **Headings**: `"Playfair Display", "Georgia", serif`, weight 700
- **Body**: `"Source Serif 4", "Georgia", serif`, weight 400
- **UI labels**: `"Inter", system-ui, sans-serif`, weight 500

| Level | Size | Weight | Line-height | Letter-spacing | Use |
|---|---|---|---|---|---|
| display | 3rem (48px) | 700 | 1.15 | -0.02em | Hero headline |
| h1 | 2.25rem (36px) | 700 | 1.2 | -0.015em | Article title |
| h2 | 1.75rem (28px) | 700 | 1.25 | -0.01em | Section header |
| h3 | 1.25rem (20px) | 600 | 1.35 | 0 | Subheading |
| body-lg | 1.25rem (20px) | 400 | 1.7 | 0 | Article body (long-form) |
| body | 1rem (16px) | 400 | 1.6 | 0 | Default text |
| caption | 0.8125rem (13px) | 500 | 1.4 | 0.03em | Bylines, dates, tags |

## Spacing

8px base: `8 / 16 / 24 / 32 / 48 / 64 / 80 / 120`

Content column: max-width 680px, centered. Sidebars: 280px.

## Component patterns

### Article card
- Large featured image (aspect 16:9), bottom text block
- Title in serif h3, author/date in caption sans-serif
- No border, just spacing separation. Hover: slight shadow lift.

### Pull quote
- Left border 3px `--color-primary`, italic serif, 1.25rem

### Navigation
- Minimal top bar, logo left, text links right
- Active state: underline 2px `--color-primary`, offset 4px

## Motion

- `--transition-fast: 200ms ease` — link hovers, underlines
- `--transition-normal: 300ms ease-in-out` — card hover lift

## Do

- Large reading font (18-20px body) with generous line-height (1.6-1.7).
- One serif for headings, one serif (or sans) for body. Never 3+ fonts.
- Photography over illustration. Real images over stock.
- Generous top/bottom padding on sections (80-120px).

## Don't

- Rainbow-colored category tags.
- Sidebar clutter (ads, widgets, social buttons).
- Sans-serif headings (defeats the editorial feel).
- Cards with identical thumbnail sizes in a rigid grid (vary the layout).
