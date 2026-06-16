---
id: methodology
title: UI/UX Designer — Methodology
domain: experts
category: uiux-designer
difficulty: intermediate
tags: [component, construction, design, experts, interaction, methodology, principles, responsive]
quality_score: 70
last_updated: 2026-06-15
---
# UI/UX Designer — Methodology

## Design System Construction

### Token Architecture (3 layers)
```
Primitive tokens:  blue-600 = #2563EB    (raw values)
Semantic tokens:   color-primary = {blue-600}    (intent)
Component tokens:  button-bg = {color-primary}   (usage)
```

Rule: components reference semantic tokens, never primitives.
Changing blue-600 updates every component that uses color-primary.

### Color System
- **6 semantic roles minimum**: bg, surface, text, primary, border, error
- **Each role needs**: default + hover + active states
- **Neutral scale**: 10 steps (50-950) for backgrounds, borders, text
- **Semantic colors**: success (green), warning (amber), error (red), info (blue)
- **Dark mode**: override semantic tokens, not primitives

### Typography System
- **2 font families max**: one display/heading, one body
- **7-step scale**: xs(12) sm(14) base(16) lg(18) xl(20) 2xl(24) 3xl(30)
- **3 weights**: regular(400), medium(500), bold(700)
- **Line-height**: tight(1.2) for headings, normal(1.5) for body, relaxed(1.7) for long-form

### Spacing System
- **Base unit**: 4px
- **Scale**: 1(4) 2(8) 3(12) 4(16) 5(20) 6(24) 8(32) 10(40) 12(48) 16(64)
- **Rule**: never use arbitrary values. Every margin/padding is a scale step.

## Interaction Design Principles

### Fitts's Law
- Larger targets are easier to click
- Closer targets are easier to reach
- **Practical**: primary CTA should be the largest button, positioned near the user's attention

### Hick's Law
- More choices = more time to decide
- **Practical**: limit navigation to 5-7 items. Use progressive disclosure (show less, reveal more on demand)

### Jakob's Law
- Users spend most of their time on OTHER sites
- **Practical**: follow established patterns (top nav, left sidebar, bottom tabs on mobile)

## Component Design Standards

### Every component needs:
1. **Default state** — what it looks like normally
2. **Hover state** — cursor enters (desktop only)
3. **Focus state** — keyboard navigation (visible focus ring, 2px solid primary, 2px offset)
4. **Active state** — being clicked/tapped
5. **Disabled state** — not interactive (opacity 0.5, cursor not-allowed)
6. **Loading state** — waiting for async (spinner or skeleton)
7. **Error state** — validation failed (red border, error message below)

### Form Design
- Labels ABOVE inputs (not inside as placeholder)
- Error messages BELOW the field (not as tooltip)
- Inline validation on blur, not on every keystroke
- Show password toggle on password fields
- Autofocus first field on page load
- Tab order matches visual order

### Empty States
Every list/table/grid needs 3 empty states:
1. **First-time**: "No items yet. Create your first X." + CTA button
2. **Filter empty**: "No results match your filters." + clear filters link
3. **Error**: "Failed to load. Please try again." + retry button

### Loading States
- **Skeleton screens** for initial page load (not spinner)
- **Inline spinner** for button actions ("Saving..." with spinner)
- **Progress bar** for known-duration operations (file upload)
- **Optimistic UI** for instant-feeling mutations (update UI, then sync)

## Responsive Design

### Breakpoints
| Name | Min width | Typical device |
|---|---|---|
| mobile | 0 | phones (portrait) |
| tablet | 640px | phones (landscape), small tablets |
| desktop | 1024px | laptops, desktops |
| wide | 1280px | large monitors |

### Mobile-first Rules
- Start with mobile layout, add complexity for larger screens
- Touch targets: minimum 44×44px
- No hover-dependent functionality on mobile
- Single-column layout on mobile, multi-column on desktop
- Bottom navigation on mobile, top/side navigation on desktop

## Accessibility (WCAG 2.1 AA)

### Color Contrast
- Body text: 4.5:1 ratio minimum
- Large text (≥24px or bold ≥18.5px): 3:1 ratio minimum
- UI controls (borders, icons): 3:1 ratio minimum

### Keyboard
- Every interactive element reachable via Tab
- Focus order matches visual order
- Escape closes modals/drawers
- Enter/Space activates buttons
- Arrow keys navigate within components (tabs, menus, radio groups)

### Screen Readers
- All images have alt text (decorative: alt="")
- Headings form a logical hierarchy (h1→h2→h3, no skipping)
- Form inputs have associated labels
- Dynamic content updates use aria-live="polite"
- Modals trap focus and have aria-modal="true"
