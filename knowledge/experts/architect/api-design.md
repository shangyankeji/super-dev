---
id: api-design
title: Architect — RESTful API Design Standards
domain: experts
category: architect
difficulty: intermediate
tags: [api, authentication, cors, design, experts, filtering, limiting, pagination]
quality_score: 70
last_updated: 2026-06-15
---
# Architect — RESTful API Design Standards

## URL Design

### Naming Conventions
- Use nouns, not verbs: `/users` not `/getUsers`
- Plural for collections: `/users`, `/posts`, `/comments`
- Nested resources for relationships: `/users/{id}/posts`
- Max 2 levels of nesting: `/users/{id}/posts` (OK), `/users/{id}/posts/{id}/comments/{id}/likes` (too deep → flatten)
- Kebab-case for multi-word: `/user-profiles` not `/userProfiles`
- No trailing slashes: `/users` not `/users/`

### HTTP Methods
| Method | Use | Idempotent | Safe | Example |
|---|---|---|---|---|
| GET | Read | Yes | Yes | `GET /users/123` |
| POST | Create | No | No | `POST /users` |
| PUT | Full replace | Yes | No | `PUT /users/123` |
| PATCH | Partial update | Yes | No | `PATCH /users/123` |
| DELETE | Remove | Yes | No | `DELETE /users/123` |

### Versioning
- URL prefix: `/api/v1/users`
- Not headers (harder to test, cache, share)
- Increment on breaking changes only

## Request/Response Standards

### Request Body
```json
{
  "email": "user@example.com",
  "name": "Jane Doe",
  "role": "admin"
}
```
- camelCase for JSON fields
- Validate ALL fields server-side (never trust client)
- Return 422 for validation errors with field-level details

### Success Response
```json
{
  "data": { ... },
  "meta": {
    "requestId": "req_abc123",
    "timestamp": "2026-01-15T10:30:00Z"
  }
}
```

### Error Response
```json
{
  "error": {
    "code": "VALIDATION_ERROR",
    "message": "Invalid input",
    "details": [
      { "field": "email", "message": "Invalid email format" }
    ],
    "requestId": "req_abc123"
  }
}
```

### Status Codes
| Code | When | Response body |
|---|---|---|
| 200 | Success (with data) | `{ "data": ... }` |
| 201 | Created | `{ "data": newResource }` with `Location` header |
| 204 | Success (no content) | empty body (DELETE, some PUTs) |
| 400 | Malformed request | `{ "error": { "code": "BAD_REQUEST" } }` |
| 401 | Not authenticated | `{ "error": { "code": "UNAUTHORIZED" } }` |
| 403 | Authenticated but forbidden | `{ "error": { "code": "FORBIDDEN" } }` |
| 404 | Resource not found | `{ "error": { "code": "NOT_FOUND" } }` |
| 409 | Conflict (duplicate) | `{ "error": { "code": "CONFLICT" } }` |
| 422 | Validation error | `{ "error": { "code": "VALIDATION_ERROR", "details": [...] } }` |
| 429 | Rate limited | `{ "error": { "code": "RATE_LIMITED" } }` + `Retry-After` header |
| 500 | Server error | `{ "error": { "code": "INTERNAL_ERROR" } }` (no internal details!) |

## Pagination

### Cursor-based (recommended)
```
GET /posts?cursor=abc123&limit=20
```
Response:
```json
{
  "data": [...],
  "pagination": {
    "nextCursor": "def456",
    "hasMore": true,
    "limit": 20
  }
}
```

### Offset-based (simpler but less performant)
```
GET /posts?page=2&limit=20
```
Response:
```json
{
  "data": [...],
  "pagination": {
    "page": 2,
    "limit": 20,
    "total": 156,
    "totalPages": 8
  }
}
```

## Filtering & Sorting

```
GET /posts?status=published&author=123&sort=-createdAt&fields=id,title
```
- Filter by field: `?status=published`
- Multiple values: `?status=published,draft`
- Sort: `?sort=createdAt` (asc), `?sort=-createdAt` (desc)
- Field selection: `?fields=id,title,author`

## Authentication

### JWT Best Practices
- Short-lived access tokens (15 min)
- Long-lived refresh tokens (7 days, stored httpOnly cookie)
- Rotate refresh tokens on use (one-time use)
- Include minimal claims: `{ sub, role, iat, exp }`
- Never store secrets in JWT payload

### Authorization Header
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIs...
```

## Rate Limiting

- Return `429 Too Many Requests` with `Retry-After` header
- Common limits:
  - Auth endpoints: 5/min per IP
  - API endpoints: 100/min per user
  - Search: 30/min per user

## CORS

```
Access-Control-Allow-Origin: https://your-frontend.com
Access-Control-Allow-Methods: GET, POST, PUT, PATCH, DELETE, OPTIONS
Access-Control-Allow-Headers: Content-Type, Authorization
Access-Control-Max-Age: 86400
```
Never use `*` in production.
