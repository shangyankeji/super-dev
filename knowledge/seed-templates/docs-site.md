---
id: docs-site
title: Seed Template: Documentation Site
domain: seed-templates
category: docs-site.md
difficulty: intermediate
tags: [component, docs, gates, page, patterns, quality, seed-templates, site]
quality_score: 70
last_updated: 2026-06-15
---
# Seed Template: Documentation Site

> Use for API docs, developer guides, knowledge bases.
> Recommended design direction: Modern Minimal or Tech Utility.

## Page structure

```
┌─────────────────────────────────────────────────┐
│  Top nav: logo · search (cmd+K) · version · GitHub│
├──────┬──────────────────────────────────────────┤
│ Left │  Content area (max-width 780px)           │
│ side │                                            │
│ bar  │  # Page title                              │
│ 240px│  > Callout box                             │
│      │  Body text (16px, 1.6 line-height)         │
│ tree │  ## Section heading                        │
│ nav  │  Code block (syntax highlighted)           │
│      │  ### Subsection                            │
│      │  Table (API parameters)                    │
│      │                                            │
│      ├──────────────────────────────────────────┤
│      │  Prev/Next navigation                      │
│      │  "Was this helpful?" feedback               │
└──────┴──────────────────────────────────────────┘
```

## Component patterns

### Sidebar navigation
- Collapsible tree with section > page > anchor structure
- Active page highlighted, active section expanded
- Mobile: hamburger menu slides from left

### Code block
- Syntax highlighting (language label top-right)
- Copy button (click → "Copied!")
- Tab group for multi-language examples (curl / JS / Python)

### Callout boxes (4 types)
- Info: blue-left-border + info icon
- Warning: yellow-left-border + warning icon
- Tip: green-left-border + lightbulb icon
- Danger: red-left-border + alert icon

### Search (Cmd+K)
- Modal overlay with input + instant results
- Keyboard navigable (↑↓ + Enter)
- Results grouped by section

## Quality gates

### P0
- [ ] Navigation tree works (expand/collapse, active state)
- [ ] Code blocks have copy button
- [ ] Mobile sidebar collapses to hamburger
- [ ] Content area has appropriate max-width for reading

### P1
- [ ] Search modal opens on Cmd+K
- [ ] Syntax highlighting renders correctly
- [ ] Table of contents on right side (for long pages)
- [ ] Prev/Next links navigate between pages
