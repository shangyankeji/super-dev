---
id: bold-geometric
title: Bold Geometric
domain: design-systems
category: bold-geometric.md
difficulty: intermediate
tags: [bold, color, component, design-systems, geometric, layout, palette, patterns]
quality_score: 70
last_updated: 2026-06-15
---
# Bold Geometric

> High contrast, oversized type, asymmetric layouts. Inspired by Figma marketing, Stripe keynotes, Apple product pages.

## When to use

Creative agencies, product launches, marketing landing pages, portfolios, brand showcase sites. Products where visual impact matters more than utility density.

## Color palette

```css
:root {
  --color-bg: #000000;
  --color-surface: #111111;
  --color-surface-elevated: #1a1a1a;
  --color-surface-sunken: #000000;
  --color-text: #ffffff;
  --color-text-secondary: #999999;
  --color-text-tertiary: #666666;
  --color-primary: #ff6b35;
  --color-primary-hover: #ff8555;
  --color-primary-muted: rgba(255, 107, 53, 0.15);
  --color-accent: #00d4aa;
  --color-success: #00d4aa;
  --color-warning: #ffb800;
  --color-error: #ff4444;
  --color-border: #222222;
  --color-border-hover: #444444;
  --color-border-focus: #ff6b35;
  --shadow-glow: 0 0 40px rgba(255, 107, 53, 0.15);
}

@media (prefers-color-scheme: light) {
  :root {
    --color-bg: #ffffff;
    --color-surface: #f5f5f5;
    --color-text: #000000;
    --color-text-secondary: #555555;
    --color-border: #e0e0e0;
    --shadow-glow: none;
  }
}
```

## Typography

- **Display / Headlines**: `"Clash Display", "Space Grotesk", system-ui, sans-serif`, weight 700
- **Body**: `"Space Grotesk", "DM Sans", system-ui, sans-serif`, weight 400

| Level | Size | Weight | Line-height | Letter-spacing | Use |
|---|---|---|---|---|---|
| display | 5rem (80px) | 700 | 0.95 | -0.04em | Hero headline (one line) |
| h1 | 3rem (48px) | 700 | 1.05 | -0.03em | Section headline |
| h2 | 2rem (32px) | 600 | 1.15 | -0.02em | Subsection |
| h3 | 1.25rem (20px) | 600 | 1.3 | -0.01em | Card title |
| body-lg | 1.25rem (20px) | 400 | 1.6 | 0 | Lead paragraph |
| body | 1rem (16px) | 400 | 1.6 | 0 | Default text |
| overline | 0.75rem (12px) | 700 | 1.2 | 0.15em | Section label, ALL CAPS |

## Spacing

8px base: `8 / 16 / 32 / 48 / 64 / 96 / 128 / 160`

Dramatic spacing. Hero sections: 160px+ vertical padding. Section gaps: 96-128px.

## Layout

- Asymmetric grids (60/40 or 70/30 splits, not 50/50)
- Full-bleed sections alternating with constrained content
- Max content width: 1200px, but hero/feature sections go edge-to-edge
- Stagger elements vertically for visual tension

## Component patterns

### Hero
- Oversized headline (80px+), short subtitle, single CTA
- Dark bg with subtle gradient or texture (NOT purple/pink)
- CTA: pill button with glow shadow on hover

### Feature section
- Large visual (mockup/screenshot) on one side, text on the other
- Asymmetric split. Text side has overline label + h1 + body paragraph
- Alternate left/right for rhythm

### Stats / social proof
- Large numbers (48px+ mono or display font)
- Minimal labels below (caption weight)
- 3-column grid, generous gap

## Motion

- `--transition-fast: 200ms cubic-bezier(0.16, 1, 0.3, 1)` — hover
- `--transition-reveal: 600ms cubic-bezier(0.16, 1, 0.3, 1)` — scroll-triggered entrance
- Scroll-triggered fade-up for sections (offset: 40px, staggered 100ms)

## Do

- One BOLD move per section (oversized type OR dramatic image OR striking color — pick one).
- Negative space as a power tool. Let elements breathe.
- Dark mode as primary. Light as secondary.
- Monochrome palette + ONE accent color (max 2 accent appearances per screen).
- Overline labels in ALL CAPS with wide letter-spacing.

## Don't

- Multiple competing focal points in one viewport.
- Gradient backgrounds (use solid darks or subtle textures).
- Small, timid typography. If you're not going big, use Modern Minimal instead.
- Centered text blocks wider than 600px (they become hard to read).
- More than 3 type sizes visible at once per viewport.
