---
id: blog-content
title: Seed Template: Blog / Content Site
domain: seed-templates
category: blog-content.md
difficulty: intermediate
tags: [article, articles, blog, content, gates, page, quality, rules]
quality_score: 70
last_updated: 2026-06-15
---
# Seed Template: Blog / Content Site

> Use this structure when the product is a blog, documentation site, or content-heavy platform.
> Recommended design direction: Editorial Clean.

## Page structure

```
┌─────────────────────────────────────────────────┐
│  Nav: logo · category links · search · theme btn │
├─────────────────────────────────────────────────┤
│  Featured: hero article with large image         │
│  ┌────────────────────────────────────────────┐  │
│  │  Category tag · Read time                  │  │
│  │  Article Title (display font, large)       │  │
│  │  Excerpt (body-lg)                         │  │
│  │  Author avatar + name + date               │  │
│  └────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────┤
│  Article grid: 2-3 column cards                  │
│  Each: thumbnail · category · title · excerpt    │
├─────────────────────────────────────────────────┤
│  Newsletter CTA: simple email capture            │
├─────────────────────────────────────────────────┤
│  Footer: about · categories · social · copyright │
└─────────────────────────────────────────────────┘
```

## Article page structure

```
┌─────────────────────────────────────────────────┐
│  Nav (same as index)                             │
├─────────────────────────────────────────────────┤
│  Article header:                                 │
│    Category · Read time                          │
│    Title (display, 36-48px)                      │
│    Subtitle (body-lg)                            │
│    Author + date + share buttons                 │
├─────────────────────────────────────────────────┤
│  ┌─────────┐                                     │
│  │ Content │  max-width: 680px, centered          │
│  │ column  │  Serif body text, 18-20px            │
│  │         │  Line-height: 1.7                    │
│  │ h2/h3   │  Serif headings                      │
│  │ images  │  Full-width within column             │
│  │ code    │  Monospace, bg-surface-sunken         │
│  │ quotes  │  Left border + italic                 │
│  └─────────┘                                     │
├─────────────────────────────────────────────────┤
│  Related articles: 3-column card grid            │
├─────────────────────────────────────────────────┤
│  Footer                                          │
└─────────────────────────────────────────────────┘
```

## Typography rules for articles

- Body text: serif, 18-20px, line-height 1.7 — optimized for reading
- Headings: same serif family, weight 700
- Code blocks: monospace, 14px, surface-sunken background
- Links: underline on hover, primary color
- Blockquote: left border 3px primary, italic
- Max content width: 680px (optimal reading width)

## Quality gates

### P0
- [ ] Articles are readable (correct font size, line-height, max-width)
- [ ] Navigation works between index and article pages
- [ ] Images have aspect ratios maintained
- [ ] Mobile responsive (single column on mobile)

### P1
- [ ] Category filtering works on index page
- [ ] Article metadata (date, read time, author) is formatted
- [ ] Code blocks have syntax highlighting
- [ ] Newsletter form validates email

### P2
- [ ] Dark mode support
- [ ] Reading progress indicator
- [ ] Smooth scroll to headings
- [ ] Table of contents for long articles
