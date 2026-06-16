---
id: dashboard
title: Seed Template: Dashboard
domain: seed-templates
category: dashboard.md
difficulty: intermediate
tags: [component, dashboard, gates, layout, page, patterns, quality, rules]
quality_score: 70
last_updated: 2026-06-15
---
# Seed Template: Dashboard

> Use this structure when the product is a data dashboard, admin panel, or monitoring tool.
> Recommended design direction: Tech Utility or Modern Minimal.

## Page structure

```
┌──────┬──────────────────────────────────────────┐
│ Side │  Top bar: breadcrumb · search · avatar   │
│ bar  ├──────────────────────────────────────────┤
│      │  KPI row: 3-4 stat cards                  │
│ Nav  │  ┌──────┐ ┌──────┐ ┌──────┐ ┌──────┐    │
│ 220px│  │ Stat │ │ Stat │ │ Stat │ │ Stat │    │
│      │  └──────┘ └──────┘ └──────┘ └──────┘    │
│      ├──────────────────────────────────────────┤
│      │  Primary chart: full-width, 300px height  │
│      ├───────────────────┬──────────────────────┤
│      │  Data table       │  Secondary chart     │
│      │  (sortable, paged)│  or activity feed    │
│      │                   │                      │
└──────┴───────────────────┴──────────────────────┘
```

## Layout rules

- Sidebar: fixed 220px width, collapsible to 64px (icons only) on mobile
- Content area: fluid, min-width 640px
- KPI cards: equal width, flexbox row with 16px gap
- Primary chart: full content width, 280-320px height
- Bottom row: 60/40 or 50/50 split

## Component patterns

### Stat card
```
┌─────────────────────┐
│  Label (caption)     │
│  123,456 (num, 28px) │
│  ↑ 12.3% (trend)    │
└─────────────────────┘
```
- Number right-aligned or left-aligned with trend badge
- Trend: green up-arrow for positive, red down-arrow for negative
- Use monospace for numbers

### Data table
- Sticky header row
- Alternating row colors (transparent / surface-sunken)
- Sortable columns with chevron indicator
- Pagination: "1-20 of 156" + prev/next
- Row height: 44px for comfortable clicking
- Monospace for IDs, timestamps, numeric columns

### Sidebar nav
- Active item: bg-primary-muted + text-primary + left border 3px
- Hover: bg-surface-sunken
- Section headers: caption weight, uppercase, 0.08em letter-spacing
- Icons: 20px Lucide, stroke-width 1.5

## Quality gates

### P0
- [ ] Sidebar navigation works (active state tracks route)
- [ ] KPI cards show realistic numbers (not "123" or "N/A")
- [ ] Table sorts on column click
- [ ] Responsive: sidebar collapses on mobile

### P1
- [ ] Chart renders (even if placeholder SVG)
- [ ] Loading skeletons for KPI cards and table
- [ ] Empty state for table when no data
- [ ] Keyboard navigable (Tab through nav items)

### P2
- [ ] Transition animations on stat card number changes
- [ ] Table row hover highlight
- [ ] Chart tooltip on hover
- [ ] Dark mode via design tokens
