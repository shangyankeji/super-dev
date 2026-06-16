---
id: methodology
title: QA Lead — Quality Assurance Methodology
domain: experts
category: qa-lead
difficulty: intermediate
tags: [categories, experts, framework, gates, methodology, process, quality, strategy]
quality_score: 70
last_updated: 2026-06-15
---
# QA Lead — Quality Assurance Methodology

## Test Strategy Framework

### Test Pyramid
```
        ╱  E2E  ╲          ~5%  — critical user journeys only
       ╱ Integr. ╲         ~15% — API contracts, DB, external services
      ╱   Unit    ╲        ~80% — business logic, pure functions
     ╱─────────────╲
```
- Unit tests: fast, isolated, test one behavior per test
- Integration tests: verify real interactions (DB, cache, APIs)
- E2E tests: cover the golden path + top 3 error scenarios per feature
- Never mock what you don't own; use fakes/containers instead

### Test Design Principles
- Arrange-Act-Assert (AAA) pattern for every test
- One assertion per test (logical assertion, not literally one `assert`)
- Test behavior, not implementation — don't test private methods
- Use descriptive test names: `should_reject_login_when_password_expired`
- Tests must be deterministic: no date/time, no random, no network

### Acceptance Criteria → Test Mapping
| AC Pattern | Test Type | Example |
|---|---|---|
| "User can..." | E2E + Integration | Login flow → API → DB |
| "System must..." | Integration + Unit | Rate limiting, validation |
| "When X, then Y" | Unit | Business rule logic |
| "Never/always..." | Unit + Property | Invariant tests |
| "Within N ms" | Performance | Load test, benchmark |

## Test Categories

### Functional Testing
- **Smoke tests**: top 5 critical paths pass? Deploy gate.
- **Regression tests**: previously broken scenarios stay fixed
- **Boundary tests**: min/max/empty/null/unicode/overflow
- **Negative tests**: invalid input, unauthorized access, rate limits

### Non-Functional Testing
- **Performance**: response time P50/P95/P99 under expected load
- **Load**: sustained traffic at 2x expected peak
- **Security**: OWASP Top 10 checklist, dependency audit
- **Accessibility**: WCAG 2.1 AA automated checks + manual screen reader

### Test Data Management
- Factory pattern for test data creation (not raw SQL inserts)
- Each test creates its own data; no shared fixtures between tests
- Use database transactions with rollback for test isolation
- Sensitive test data: use faker/fabricator, never real PII

## Quality Gates

### Pre-Merge Gate
- [ ] All unit tests pass
- [ ] All integration tests pass
- [ ] Code coverage ≥ 80% (new code ≥ 90%)
- [ ] No new linting errors
- [ ] Security scan clean (Snyk/Trivy/Dependabot)
- [ ] Type check passes

### Pre-Release Gate
- [ ] Smoke tests pass against staging
- [ ] E2E suite green (≤ 2% flaky tolerance)
- [ ] Performance benchmarks within baseline ± 10%
- [ ] Accessibility audit passes
- [ ] Manual QA sign-off on new features
- [ ] Rollback procedure verified

### Post-Release Verification
- [ ] Health checks green for 15 minutes
- [ ] Error rate ≤ baseline + 0.1%
- [ ] P95 latency ≤ baseline + 20%
- [ ] Key business metrics trending normally
- [ ] No new error patterns in log aggregation

## Bug Triage Process

### Severity Classification
| Severity | Impact | Response Time | Example |
|---|---|---|---|
| P0 - Critical | Service down, data loss | Immediate | Auth broken, DB corruption |
| P1 - High | Major feature broken | < 4 hours | Checkout fails for 20% users |
| P2 - Medium | Feature degraded | < 24 hours | Search returns stale results |
| P3 - Low | Minor issue | Next sprint | UI alignment off on Safari |

### Root Cause Analysis
1. Reproduce the bug with minimum steps
2. Identify the root cause (not just the symptom)
3. Write a failing test that catches the bug
4. Fix the code
5. Verify the test passes
6. Check for similar patterns elsewhere in codebase

## CI/CD Quality Integration

### Pipeline Quality Checks
```
commit → lint → type-check → unit → integration → build → deploy(staging) → smoke → deploy(prod) → verify
```

### Flaky Test Management
- Quarantine flaky tests (don't delete, don't block pipeline)
- Maximum 2% flaky rate; above this → halt new features until fixed
- Track flaky tests with retry count; >3 retries = quarantine
- Root-cause every flaky test: timing, ordering, shared state, network

### Test Coverage Policy
- Coverage is a floor, not a ceiling — high coverage ≠ good tests
- Focus coverage on: business logic, error handling, state transitions
- Exempt from coverage: generated code, configuration, type definitions
- Use mutation testing quarterly to verify test quality
