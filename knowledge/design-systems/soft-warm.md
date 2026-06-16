---
id: soft-warm
title: Soft Warm
domain: design-systems
category: soft-warm.md
difficulty: intermediate
tags: [borders, color, component, design-systems, palette, patterns, radius, soft]
quality_score: 70
last_updated: 2026-06-15
---
# Soft Warm

> Rounded, approachable, warm tones. Inspired by Notion, Headspace, Duolingo.

## When to use

Consumer apps, education, wellness, onboarding flows, community products. Products where friendliness and accessibility matter more than information density.

## Color palette

```css
:root {
  --color-bg: #fffbf5;
  --color-surface: #ffffff;
  --color-surface-elevated: #ffffff;
  --color-surface-sunken: #faf5ee;
  --color-text: #37352f;
  --color-text-secondary: #787774;
  --color-text-tertiary: #b4b4b0;
  --color-primary: #eb5757;
  --color-primary-hover: #d94444;
  --color-primary-muted: #fce8e8;
  --color-accent: #4ea8de;
  --color-success: #4dab6f;
  --color-warning: #e9b949;
  --color-error: #eb5757;
  --color-border: #e9e5df;
  --color-border-hover: #ddd9d3;
  --color-border-focus: #eb5757;
  --shadow-sm: 0 1px 4px rgb(55 53 47 / 0.06);
  --shadow-md: 0 4px 16px rgb(55 53 47 / 0.08);
}

@media (prefers-color-scheme: dark) {
  :root {
    --color-bg: #191919;
    --color-surface: #202020;
    --color-surface-sunken: #141414;
    --color-text: #ffffffcf;
    --color-text-secondary: #ffffff80;
    --color-border: #ffffff14;
  }
}
```

## Typography

- **Headings**: `"DM Sans", "Nunito", system-ui, sans-serif`, weight 700
- **Body**: `"DM Sans", "Nunito", system-ui, sans-serif`, weight 400

| Level | Size | Weight | Line-height | Use |
|---|---|---|---|---|
| h1 | 2rem (32px) | 700 | 1.25 | Page title |
| h2 | 1.5rem (24px) | 700 | 1.3 | Section header |
| h3 | 1.125rem (18px) | 600 | 1.4 | Card title |
| body | 1rem (16px) | 400 | 1.6 | Default text |
| body-sm | 0.875rem (14px) | 400 | 1.5 | Secondary text |
| caption | 0.75rem (12px) | 600 | 1.4 | Labels |

## Spacing

4px base: `4 / 8 / 12 / 16 / 24 / 32 / 48 / 64`

Generous padding. Cards: 24px minimum. Sections: 48-64px vertical.

## Borders & radius

- Default radius: `12px`
- Small radius: `8px`
- Large radius: `16px`
- Full radius: `9999px` (avatars, pills, floating action buttons)

## Component patterns

### Card
- `bg-surface radius-lg shadow-sm`, 24px padding
- Hover: translate-y -2px + shadow-md (gentle lift)
- Colorful left accent stripe (4px, rounded) optional

### Button
- Primary: `bg-primary text-white radius-full`, 44px height
- Hover: scale 1.02 + darken 5%
- Pill-shaped for primary CTAs

### Avatar
- Circular, border 2px white, pastel background for initials
- Size: 32px (sm), 40px (md), 56px (lg)

### Toast/notification
- Rounded, gentle shadow, slide-in from bottom
- Icon + text, dismiss on swipe

## Motion

- `--transition-fast: 180ms cubic-bezier(0.2, 0, 0, 1)` — hover
- `--transition-normal: 300ms cubic-bezier(0.2, 0, 0, 1)` — expand
- `--transition-bounce: 500ms cubic-bezier(0.34, 1.56, 0.64, 1)` — celebratory moments

## Do

- Rounded everything. 12px+ radius gives warmth.
- Pastel accent colors for backgrounds (muted tints of primary/accent).
- Playful micro-interactions (button press scale, check animation).
- Friendly copy ("You're all set!" not "Operation successful").
- Illustration style over photography when possible.

## Don't

- Sharp corners on interactive elements.
- Dense data tables (use cards or lists instead).
- Dark, moody color schemes.
- Corporate / formal tone in UI copy.
- Monospace fonts anywhere in the UI.
