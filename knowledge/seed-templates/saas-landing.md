---
id: saas-landing
title: Seed Template: SaaS Landing Page
domain: seed-templates
category: saas-landing.md
difficulty: intermediate
tags: [bottom, gates, into, landing, page, paste, quality, root]
quality_score: 70
last_updated: 2026-06-15
---
# Seed Template: SaaS Landing Page

> Use this structure when the product is a SaaS landing / marketing page.
> The worker should follow this section order and fill each slot.

## Page structure (top → bottom)

```
┌─────────────────────────────────────────────────┐
│  Nav: logo (left) · links (center) · CTA (right)│
├─────────────────────────────────────────────────┤
│  Hero: headline + subtitle + primary CTA         │
│  Optional: product screenshot or animation       │
├─────────────────────────────────────────────────┤
│  Trust bar: 4-6 monochrome logos, single row     │
├─────────────────────────────────────────────────┤
│  Features: 3-column icon + title + description   │
│  Use declared icon library, NOT emoji            │
├─────────────────────────────────────────────────┤
│  Product showcase: screenshot + feature callouts │
│  Alternate left/right for visual rhythm          │
├─────────────────────────────────────────────────┤
│  Testimonials: 2-3 real quotes + avatar + name   │
├─────────────────────────────────────────────────┤
│  Pricing: 2-3 tier cards with toggle (if needed) │
├─────────────────────────────────────────────────┤
│  FAQ: accordion, 5-8 questions                   │
├─────────────────────────────────────────────────┤
│  CTA footer: headline + button + trust note      │
├─────────────────────────────────────────────────┤
│  Footer: links · social icons · copyright        │
└─────────────────────────────────────────────────┘
```

## CSS skeleton (paste into :root)

```css
:root {
  /* Bind these from the UIUX doc's design tokens */
  --font-heading: /* from uiux */;
  --font-body: /* from uiux */;
  --max-width: 1200px;
  --section-padding-y: 80px;
  --section-padding-y-mobile: 48px;
}

.container {
  max-width: var(--max-width);
  margin: 0 auto;
  padding: 0 24px;
}

section {
  padding: var(--section-padding-y) 0;
}

@media (max-width: 768px) {
  section { padding: var(--section-padding-y-mobile) 0; }
}
```

## Quality gates

### P0 (must pass before preview gate)
- [ ] Page loads without JS errors
- [ ] Navigation links scroll to correct sections
- [ ] CTA buttons have hover + focus states
- [ ] Mobile responsive (test at 360px width)
- [ ] No emoji as icons anywhere

### P1 (should pass)
- [ ] Trust bar logos are monochrome (not colorful)
- [ ] Feature icons from declared library
- [ ] Testimonial avatars are properly sized circles
- [ ] Pricing cards have visual hierarchy (recommended plan highlighted)
- [ ] FAQ accordion works (open/close)

### P2 (polish)
- [ ] Dark mode works via prefers-color-scheme
- [ ] Scroll-triggered section reveal animation
- [ ] Smooth scroll between sections
- [ ] Loading skeleton for any async content
