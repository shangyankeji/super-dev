---
id: methodology
title: Product Manager — Methodology
domain: experts
category: product-manager
difficulty: intermediate
tags: [experts, framework, methodology, writing]
quality_score: 70
last_updated: 2026-06-15
---
# Product Manager — Methodology

## PRD Writing Framework

### 1. Problem Statement First

Before any solution, define:
- Who has this problem? (persona, not demographics)
- How are they solving it today? (current workaround)
- Why is the current solution inadequate? (pain intensity 1-10)
- What evidence do we have? (user interviews, data, support tickets)

### 2. Requirements Prioritization

Use RICE scoring for feature prioritization:

| Factor | Definition | Scale |
|---|---|---|
| **R**each | How many users per quarter? | actual number |
| **I**mpact | How much does it move the needle? | 3=massive, 2=high, 1=medium, 0.5=low, 0.25=minimal |
| **C**onfidence | How sure are we? | 100%=high, 80%=medium, 50%=low |
| **E**ffort | Person-months to build | actual estimate |

RICE Score = (Reach × Impact × Confidence) / Effort

### 3. Acceptance Criteria Standard

Every AC must be:
- **Specific** — no ambiguous words ("fast", "nice", "easy")
- **Measurable** — has a number or binary check
- **Independent** — can be tested without other ACs
- **Format** — Given [precondition], When [action], Then [observable result]

Bad: "The page should load fast"
Good: "Given a user on 3G connection, when they open /dashboard, then First Contentful Paint < 2s"

Bad: "Login should be secure"
Good: "Given 5 failed login attempts from same IP in 10 minutes, when the 6th attempt is made, then return 429 and lock for 15 minutes"

### 4. Edge Cases Checklist

Every feature must consider:
- Empty state (no data yet)
- Error state (API failure, validation failure)
- Loading state (in-progress)
- Boundary values (0, 1, max, negative)
- Concurrent users (race conditions)
- Offline/slow network
- Permission denied
- Already deleted / stale data

### 5. Non-Functional Requirements Template

| Category | Requirement | Target | How to Measure |
|---|---|---|---|
| Performance | Page load time | FCP < 1.5s | Lighthouse CI |
| Performance | API response time | p95 < 200ms | Server metrics |
| Performance | Concurrent users | 1000 simultaneous | Load test |
| Security | Authentication | JWT with refresh | Manual audit |
| Security | Data encryption | TLS 1.3 + at-rest AES-256 | Security scan |
| Security | Input validation | All endpoints | Automated test |
| Accessibility | WCAG level | 2.1 AA | axe-core audit |
| Accessibility | Keyboard navigation | All interactive elements | Manual test |
| Reliability | Uptime | 99.9% | Monitoring |
| Reliability | Error rate | < 0.1% | Error tracking |

### 6. Success Metrics Framework

Use the HEART framework:
- **H**appiness — user satisfaction (NPS, CSAT)
- **E**ngagement — usage frequency, session duration
- **A**doption — new users, feature adoption rate
- **R**etention — day-1/7/30 retention
- **T**ask success — completion rate, time-to-complete

Each metric needs: baseline → target → measurement method → review cadence

### 7. Common PRD Mistakes to Avoid

1. **Solution before problem** — jumping to "we need a button" before defining the user need
2. **Vague acceptance criteria** — "should be intuitive" is not testable
3. **Missing edge cases** — happy path only, no error handling
4. **No success metrics** — shipping without knowing if it worked
5. **Scope creep built-in** — "and also it would be nice if..." without marking as out-of-scope
6. **Missing non-functional** — no performance targets, no security requirements
7. **No user flow** — feature list without showing how they connect
8. **Assuming implementation** — "use React" in a PRD (that's architecture, not product)
