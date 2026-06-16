---
id: e-commerce
title: Seed Template: E-Commerce Product Page
domain: seed-templates
category: e-commerce.md
difficulty: intermediate
tags: [commerce, gates, page, quality, seed-templates, structure]
quality_score: 70
last_updated: 2026-06-15
---
# Seed Template: E-Commerce Product Page

> Use for online shops, marketplaces, product catalogs.
> Recommended design direction: Modern Minimal or Soft Warm.

## Page structure

```
┌─────────────────────────────────────────────────┐
│  Nav: logo · search · cart(badge) · account      │
├─────────────────────────────────────────────────┤
│  Breadcrumb: Home > Category > Product           │
├──────────────────────┬──────────────────────────┤
│  Image gallery       │  Product info             │
│  (main + thumbnails) │  Title · Price · Rating   │
│  zoom on hover       │  Variant selector          │
│                      │  Add to Cart button        │
│                      │  Shipping info             │
├──────────────────────┴──────────────────────────┤
│  Tabs: Description · Specs · Reviews              │
├─────────────────────────────────────────────────┤
│  Related products: 4-column card grid             │
├─────────────────────────────────────────────────┤
│  Footer: links · payment icons · copyright        │
└─────────────────────────────────────────────────┘
```

## Quality gates

### P0
- [ ] Add to Cart button is prominent and always visible
- [ ] Price displayed clearly with currency
- [ ] Image gallery loads and thumbnails switch main image
- [ ] Mobile: single column, sticky "Add to Cart" bottom bar

### P1
- [ ] Variant selector (size/color) updates price and image
- [ ] Rating stars rendered as SVG (not emoji)
- [ ] Related products section populated
- [ ] Quantity selector with +/- buttons
