---
id: methodology
title: Architect — System Architecture Methodology
domain: experts
category: architect
difficulty: intermediate
tags: [architecture, data, decision, design, experts, framework, infrastructure, methodology]
quality_score: 70
last_updated: 2026-06-15
---
# Architect — System Architecture Methodology

## Architecture Decision Framework

### Technology Selection
- Evaluate based on: team expertise, community size, long-term viability, license
- Document decision rationale in ADR (Architecture Decision Record) format
- Prefer boring technology for core infrastructure; innovate at edges
- Prototype risky integrations before committing

### Scalability Planning
- Design for 10x current load; plan for 100x
- Identify bottlenecks early: database reads, API fanout, file I/O
- Separate read/write paths when read:write ratio exceeds 10:1
- Use connection pooling for all database and HTTP client connections

## System Design Patterns

### Layered Architecture
```
┌─────────────────────────┐
│  Presentation (UI/API)  │
├─────────────────────────┤
│  Application (Use Cases)│
├─────────────────────────┤
│  Domain (Business Logic)│
├─────────────────────────┤
│  Infrastructure (DB/IO) │
└─────────────────────────┘
```
- Each layer only depends on the layer below
- Domain layer has zero external dependencies
- Infrastructure implements interfaces defined by domain

### API Gateway Pattern
- Single entry point for all client requests
- Rate limiting, auth, logging at gateway level
- Request routing to appropriate microservice/handler
- Response aggregation for composite endpoints

### Event-Driven Architecture
- Use events for cross-domain communication
- Event store for audit trail and replay capability
- Idempotent event handlers (at-least-once delivery)
- Dead letter queue for failed event processing

## Data Architecture

### Database Selection Matrix
| Requirement | Relational (PostgreSQL) | Document (MongoDB) | KV (Redis) |
|---|---|---|---|
| Complex queries | Best | Poor | N/A |
| Schema flexibility | Moderate | Best | N/A |
| Transactions | Best | Limited | Limited |
| Caching | Moderate | Moderate | Best |
| Full-text search | Good (w/ extensions) | Good | N/A |

### Schema Design Principles
- Normalize to 3NF by default; denormalize with measurement justification
- Every table needs: `id` (UUID/ULID), `created_at`, `updated_at`
- Soft-delete with `deleted_at` for recoverable entities
- Index foreign keys and frequently queried columns
- Use enums/check constraints at DB level, not just application

### Migration Strategy
- One migration per logical change (not per table)
- Migrations must be reversible (up + down)
- Zero-downtime migrations: add column → backfill → add constraint → remove old
- Never rename columns in production; add new, migrate, drop old

## Infrastructure Patterns

### Deployment Architecture
- Container-first: Dockerfile for every deployable service
- Environment parity: dev ≈ staging ≈ production
- Configuration via environment variables (12-factor)
- Health check endpoints: `/health` (liveness), `/ready` (readiness)

### Caching Strategy
- Cache hierarchy: browser → CDN → application → database
- Cache invalidation: TTL for reads, explicit invalidation on writes
- Cache key design: `{entity}:{id}:{version}` for granular invalidation
- Never cache authenticated/personalized responses at CDN level

### Observability
- Structured logging (JSON) with correlation IDs
- Metrics: RED method (Rate, Errors, Duration) for services
- Distributed tracing for cross-service request flows
- Alert on symptoms (error rate, latency), not causes

## Security Architecture

### Defense in Depth
1. Network: firewall rules, VPC isolation, TLS everywhere
2. Application: input validation, output encoding, CSRF tokens
3. Data: encryption at rest, field-level encryption for PII
4. Access: RBAC/ABAC, principle of least privilege
5. Audit: immutable log of all access and mutations

### Auth Architecture
- Prefer JWT for stateless API auth; sessions for server-rendered
- Access token: short-lived (15m), refresh token: longer (7d), rotate on use
- Store refresh tokens in httpOnly secure cookies, not localStorage
- Permission model: User → Role → Permission (many-to-many)

## Architecture Review Checklist
- [ ] Single responsibility: each service/module does one thing
- [ ] Failure isolation: one component failure doesn't cascade
- [ ] Data ownership: each domain owns its data store
- [ ] API contract: OpenAPI/GraphQL schema defined before implementation
- [ ] Security: auth, authz, input validation, secrets management
- [ ] Observability: logging, metrics, tracing, alerting
- [ ] Scalability: identified bottlenecks and horizontal scaling path
- [ ] Recovery: backup strategy, disaster recovery, rollback plan
