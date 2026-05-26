# Backend Lead — Development Methodology

## API Implementation Pattern

### Controller/Handler Structure
Every API endpoint follows:
1. **Parse** — validate request body/params
2. **Authorize** — check user has permission
3. **Execute** — call business logic
4. **Respond** — format and return result

```
Request → Middleware(auth, rate-limit) → Handler(parse, authorize, execute, respond) → Response
```

### Input Validation Rules
- Validate at the API boundary, not in business logic
- Use schema validation library (Zod, Joi, Pydantic, serde)
- Return 422 with field-level errors:
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "details": [
      { "field": "email", "message": "Must be a valid email address" },
      { "field": "name", "message": "Must be between 2 and 100 characters" }
    ]
  }
}
```
- Never trust frontend validation alone

### Error Handling Pattern
```
try {
  // business logic
} catch (NotFoundError) {
  return 404 with generic message
} catch (ForbiddenError) {
  return 403
} catch (ConflictError) {
  return 409 with "resource already exists"
} catch (ValidationError) {
  return 422 with field details
} catch (unknown) {
  log error with full context (requestId, userId, stack trace)
  return 500 with "Internal server error" (NO details to client)
}
```

## Database Best Practices

### Migration Standards
- Every schema change is a migration file with up + down
- Never modify a deployed migration — create a new one
- Migrations run automatically on deploy
- Test migrations against production-like data volume

### Query Patterns
- Use parameterized queries (NEVER string concatenation)
- Add indexes for: foreign keys, frequently queried fields, sort columns
- Pagination: cursor-based for infinite scroll, offset for page numbers
- N+1 prevention: eager load relationships or use DataLoader pattern

### Seed Data
- `seeds/development.ts` — realistic test data for local dev
- `seeds/test.ts` — minimal data for automated tests
- Never seed production directly

## Authentication Implementation

### JWT Flow
```
1. POST /auth/login { email, password }
   → Verify credentials
   → Generate access token (15 min, signed)
   → Generate refresh token (7 days, stored in DB + httpOnly cookie)
   → Return { accessToken, user }

2. Authenticated request:
   → Client sends: Authorization: Bearer <accessToken>
   → Middleware verifies signature + expiration
   → Extracts user from claims

3. Token refresh:
   → POST /auth/refresh (cookie has refresh token)
   → Verify refresh token exists in DB + not expired
   → Invalidate old refresh token (single-use)
   → Issue new access + refresh tokens

4. Logout:
   → POST /auth/logout
   → Delete refresh token from DB
   → Clear httpOnly cookie
```

### Password Rules
- Hash with bcrypt (cost 12) or Argon2id
- Minimum 8 characters
- Check against breached password list (haveibeenpwned API)
- Never log plaintext passwords
- Never return password hash in API responses

## Logging Standards

### What to Log
| Level | When | Example |
|---|---|---|
| ERROR | Operation failed, needs attention | Database connection lost, payment failed |
| WARN | Degraded but functioning | Cache miss, retry succeeded, rate limit approaching |
| INFO | Significant events | User signed up, order placed, deploy completed |
| DEBUG | Development troubleshooting | Query executed, cache hit, request parsed |

### Log Format
```json
{
  "level": "error",
  "timestamp": "2026-01-15T10:30:00Z",
  "requestId": "req_abc123",
  "userId": "usr_456",
  "message": "Payment processing failed",
  "error": { "code": "STRIPE_DECLINED", "message": "Card declined" },
  "context": { "orderId": "ord_789", "amount": 9900, "currency": "usd" }
}
```

### What NOT to Log
- Passwords, tokens, API keys
- Full credit card numbers (last 4 only)
- Personal health information
- Full request/response bodies (summarize instead)

## Testing Standards

### Unit Tests
- Test business logic functions in isolation
- Mock external dependencies (database, APIs, email)
- Naming: `describe('createUser')` → `it('should hash password before saving')`
- Assert both success path and error paths

### Integration Tests
- Test API endpoints with real database (test instance)
- Reset database between tests (transaction rollback or truncate)
- Test auth: verify 401 without token, 403 with wrong role
- Test validation: verify 422 for each invalid field

### Test Coverage Targets
| Layer | Target |
|---|---|
| Business logic | ≥ 90% |
| API handlers | ≥ 80% |
| Middleware | ≥ 80% |
| Utilities | ≥ 95% |
| Overall | ≥ 80% |

## Environment Variables

### Naming Convention
```
# Database
DATABASE_URL=postgres://user:pass@host:5432/dbname
DATABASE_POOL_SIZE=10

# Auth
JWT_SECRET=<random 256-bit key>
JWT_ACCESS_TTL=900     # 15 minutes in seconds
JWT_REFRESH_TTL=604800 # 7 days in seconds

# External services
STRIPE_SECRET_KEY=sk_live_...
SMTP_HOST=smtp.example.com
SMTP_PORT=587

# App
NODE_ENV=production
PORT=3001
CORS_ORIGIN=https://your-frontend.com
LOG_LEVEL=info
```

### Rules
- NEVER commit `.env` files (add to `.gitignore`)
- Provide `.env.example` with placeholder values
- Validate all required env vars on startup (fail fast if missing)
- Different values per environment (dev/staging/prod)
