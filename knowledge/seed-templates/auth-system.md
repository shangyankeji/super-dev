---
id: auth-system
title: Seed Template: Authentication System
domain: seed-templates
category: auth-system.md
difficulty: intermediate
tags: [auth, component, gates, page, patterns, quality, seed-templates, structure]
quality_score: 70
last_updated: 2026-06-15
---
# Seed Template: Authentication System

> Use for login/signup/MFA/password-reset flows.
> Recommended design direction: Modern Minimal.

## Page structure

```
Auth flow pages (each a separate route):

/login          — email + password + "Remember me" + "Forgot?" link + OAuth buttons
/signup         — name + email + password + confirm + terms checkbox + OAuth
/forgot         — email input + "Send reset link" button
/reset/:token   — new password + confirm + "Reset" button
/verify-email   — success message + "Resend" link
/mfa            — 6-digit TOTP input + "Use backup code" link
/mfa/setup      — QR code + manual key + verify input
```

## Component patterns

### Auth card (centered on page)
- max-width: 420px, centered vertically + horizontally
- bg-surface, radius-lg, shadow-md, padding 32-40px
- Logo at top, heading below, form fields, primary CTA full-width

### OAuth buttons
- Full-width, secondary style, icon left-aligned
- "Continue with Google" / "Continue with GitHub"
- Separated from email form by "or" divider

### Password input
- Show/hide toggle (eye icon from icon library)
- Strength indicator bar below (4 segments: weak/fair/good/strong)

## Quality gates

### P0
- [ ] Login form submits (even if to /api/auth/login stub)
- [ ] Password visibility toggle works
- [ ] OAuth buttons are styled (not unstyled links)
- [ ] Mobile responsive (card shrinks, no horizontal scroll)
- [ ] Error states shown inline (red border + message below field)

### P1
- [ ] MFA TOTP input auto-advances between 6 digit boxes
- [ ] Password strength indicator animates on typing
- [ ] "Remember me" checkbox styled with custom checkmark
- [ ] Loading spinner on form submit
