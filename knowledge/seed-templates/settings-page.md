---
id: settings-page
title: Seed Template: Settings / Account Page
domain: seed-templates
category: settings-page.md
difficulty: intermediate
tags: [gates, page, quality, seed-templates, settings, structure]
quality_score: 70
last_updated: 2026-06-15
---
# Seed Template: Settings / Account Page

> Use for user profile, preferences, billing, team management.
> Recommended design direction: Modern Minimal.

## Page structure

```
┌──────┬──────────────────────────────────────────┐
│ Side │  Settings header: "Settings" + save btn   │
│ nav  ├──────────────────────────────────────────┤
│      │  Section: Profile                         │
│ tabs │  ┌ Avatar upload ┐ Name ┐ Email ┐ Bio    │
│      ├──────────────────────────────────────────┤
│      │  Section: Preferences                     │
│      │  Theme toggle · Language · Timezone        │
│      ├──────────────────────────────────────────┤
│      │  Section: Security                        │
│      │  Change password · MFA toggle · Sessions   │
│      ├──────────────────────────────────────────┤
│      │  Section: Billing (if applicable)          │
│      │  Current plan · Usage · Payment method     │
│      ├──────────────────────────────────────────┤
│      │  Danger zone: Delete account               │
└──────┴──────────────────────────────────────────┘
```

## Quality gates

### P0
- [ ] Side navigation highlights active section
- [ ] Form fields have labels and validation
- [ ] Save button shows loading + success feedback
- [ ] Danger zone is visually distinct (red border or background)
- [ ] Mobile: side nav becomes top tabs or accordion

### P1
- [ ] Avatar upload with preview
- [ ] Theme toggle instantly switches light/dark
- [ ] Password change requires current password
- [ ] Form dirty detection (unsaved changes warning)
