---
id: security
title: Architect — Security Checklist (OWASP-based)
domain: experts
category: architect
difficulty: intermediate
tags: [authentication, authorization, encoding, experts, input, management, output, security]
quality_score: 70
last_updated: 2026-06-15
---
# Architect — Security Checklist (OWASP-based)

## Authentication & Session Management

- [ ] Passwords hashed with bcrypt (cost ≥ 12) or Argon2id
- [ ] JWT access tokens expire in ≤ 15 minutes
- [ ] Refresh tokens are single-use and stored server-side
- [ ] Session invalidation on password change
- [ ] Account lockout after 5 failed attempts (15 min cooldown)
- [ ] MFA option available for sensitive operations
- [ ] Logout invalidates all tokens server-side

## Input Validation

- [ ] ALL user input validated server-side (never trust client)
- [ ] Parameterized queries / ORM for database operations (no string concatenation)
- [ ] File uploads: validate type (magic bytes, not extension), limit size, store outside webroot
- [ ] Reject unexpected fields in request body (allowlist, not blocklist)
- [ ] Content-Type header checked on all endpoints accepting body

## Output Encoding

- [ ] HTML output encoded to prevent XSS (`<script>` → `&lt;script&gt;`)
- [ ] JSON responses use `Content-Type: application/json` (never `text/html`)
- [ ] User-generated content sanitized before rendering
- [ ] CSP headers set: `Content-Security-Policy: default-src 'self'`

## Authorization

- [ ] Every endpoint checks authorization (not just authentication)
- [ ] IDOR prevention: verify user owns the resource, not just that resource exists
- [ ] Admin endpoints on separate route group with role check middleware
- [ ] API keys scoped to minimum necessary permissions

## Data Protection

- [ ] HTTPS only (HSTS header with max-age ≥ 1 year)
- [ ] Sensitive data encrypted at rest (AES-256)
- [ ] PII not logged (mask email, phone in logs)
- [ ] Database credentials in environment variables, never in code
- [ ] `.env` in `.gitignore`

## Headers

```
Strict-Transport-Security: max-age=31536000; includeSubDomains
Content-Security-Policy: default-src 'self'; script-src 'self'
X-Content-Type-Options: nosniff
X-Frame-Options: DENY
Referrer-Policy: strict-origin-when-cross-origin
Permissions-Policy: camera=(), microphone=(), geolocation=()
```

## Dependency Security

- [ ] No known vulnerabilities (`npm audit` / `cargo audit` clean)
- [ ] Dependencies pinned to exact versions in lock file
- [ ] Automated dependency update checks (Dependabot / Renovate)

## Error Handling

- [ ] Internal errors return generic message to client (no stack traces)
- [ ] Errors logged with context on server (request ID, user ID, timestamp)
- [ ] 404 for missing resources (don't leak existence via different error codes)
- [ ] Rate limiting on all public endpoints
