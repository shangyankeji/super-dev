---
id: 00-craft-rules
title: Craft Rules — Universal Visual Quality Standards
domain: design-systems
category: 00-craft-rules.md
difficulty: intermediate
tags: [auto-checked, cardinal, catch, craft, design-systems, governance, reviewer, rules]
quality_score: 70
last_updated: 2026-06-15
---
# Craft Rules — Universal Visual Quality Standards

> These rules apply to ALL frontend output regardless of design direction.
> P0 = hard blocker (must fix before preview gate).
> P1 = should fix (quality suffers noticeably).
> P2 = polish (nice to have).

## P0 — Cardinal Sins (auto-checked by governance)

1. **No emoji as functional icons.** Use Lucide / Heroicons / Tabler only.
2. **No hardcoded colors.** Every `color`, `background`, `border-color`, `box-shadow` must use a CSS var.
3. **No purple-to-pink gradient hero backgrounds.** This is the #1 "AI made this" tell.
4. **No Lorem ipsum / dolor sit amet.** Use realistic placeholder content.
5. **No "Welcome to [App]" hero headings.** Write a specific value proposition.
6. **No filler metrics.** Don't invent "10x faster" or "99.9% uptime" without a source.
7. **No emoji-icon hybrid.** Don't mix emoji and icon-library icons in the same UI.

## P1 — Soft Tells (reviewer should catch)

1. **Template skeleton without variation.** If every section follows the same card-grid pattern, add rhythm (alternate layouts, vary section heights).
2. **Accent color overuse.** Max 2 accent-colored elements visible per viewport. More than that = visual noise.
3. **Placeholder CDN images.** `placehold.co` or `unsplash.com/random` are acceptable in dev, but mark them `<!-- TODO: replace -->`.
4. **Too many raw hex values outside `:root`.** If you see 12+ unique hex values scattered in components, refactor into tokens.
5. **Sans-serif display font when the design system specifies serif.** Read the UIUX doc.
6. **Identical card content.** 3+ cards with the same placeholder text is obvious AI output. Vary the content.
7. **Missing component states.** Every button needs hover + focus + disabled. Every input needs focus + error.

## P2 — Polish

1. **Dark mode.** If the UIUX doc defines dark tokens, wire `prefers-color-scheme`.
2. **Loading states.** Skeleton screens for async content, not blank white space.
3. **Empty states.** "No items yet" with an illustration or CTA, not a blank table.
4. **Micro-interactions.** Button press feedback, card hover lift, toggle animation.
5. **Scroll-triggered reveals.** Subtle fade-up on section enter (not bouncy/spring).
6. **Focus ring.** 2px solid primary, 2px offset. Visible on keyboard navigation.

## Typography Craft

- **Letter-spacing rules:**
  - ALL CAPS text: >= 0.06em spacing (prevents cramped look)
  - Display text (>= 32px): -0.01 to -0.02em (tightens for visual weight)
  - Body text: 0 (default)
  - Caption/label text: 0.02-0.03em (opens up for legibility at small sizes)

- **Weight system:** use at most 3 weights per page (typically 400 / 500 / 700).

- **Line length:** 50-75 characters for body text. Enforce via `max-width` on text containers.

- **Max typefaces:** 2 (one for headings, one for body). Code blocks can use a third (monospace).

## Color Craft

- **Palette distribution:** neutrals 70-90%, accent 5-10%, semantic (success/warning/error) 0-5%.
- **Contrast minimums:** 4.5:1 for body text (WCAG AA), 3:1 for large text (>=24px or bold >=18.5px).
- **One decisive accent.** If you have both `--color-primary` and `--color-accent`, only one should be visible per viewport. The other is for a different screen/section.

## Layout Craft

- **Rhythm alternation.** Don't stack 3 identical-layout sections. Alternate: full-width → constrained, image-left → image-right, light-bg → dark-bg.
- **Vertical spacing progression.** Sections: 80-120px. Within sections: 32-48px between groups. Within groups: 16-24px.
- **One bold move per section.** Oversized type OR dramatic image OR striking color. Pick one. Three competing flourishes = noise.
