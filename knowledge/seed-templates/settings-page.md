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
