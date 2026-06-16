---
id: development-scenarios-guide
title: Development Scenarios Guide - Comprehensive Decision Reference
domain: development
category: 12-scenarios
difficulty: intermediate
tags: [consumer, development, enterprise, framework, growth, guide, multi-tenant, pack]
quality_score: 70
last_updated: 2026-06-15
---
# Development Scenarios Guide - Comprehensive Decision Reference

> Consolidated reference covering 8 business scenario packs (B2B, B2C, Multi-tenant SaaS, Internationalization, Mobile Superapp, AI Application, Fintech Regulated, E-commerce Peak) plus scenario selection methodology.

---

## 1. Scenario Selection Framework

### 1.1 Decision Tree

Before selecting scenario packs, answer these gate questions:

```
START
  |
  +-- Is the product enterprise-facing (B2B)?
  |     YES -> Apply B2B Scenario Pack
  |     NO  -> Is it consumer growth-focused?
  |              YES -> Apply B2C Growth Scenario Pack
  |
  +-- Does the product require tenant isolation?
  |     YES -> Apply Multi-Tenant SaaS Scenario Pack
  |
  +-- Does the product serve multiple regions / languages?
  |     YES -> Apply Internationalization Scenario Pack
  |
  +-- Is the primary experience a mobile app?
  |     YES -> Apply Mobile Superapp Scenario Pack
  |
  +-- Does the product include AI / ML generation or inference?
  |     YES -> Apply AI Application Scenario Pack
  |
  +-- Is the product subject to financial regulation?
  |     YES -> Apply Fintech Regulated Scenario Pack
  |
  +-- Does the product face peak-traffic events (promotions, flash sales)?
        YES -> Apply E-commerce Peak Scenario Pack
```

Multiple packs can be stacked. For example, a B2B SaaS product serving international clients would apply: B2B + Multi-Tenant SaaS + Internationalization.

### 1.2 Selection Checklist

Before proceeding with development, confirm:

- [ ] The current business objective is identified (efficiency, growth, compliance, or stability).
- [ ] Multi-tenant isolation and billing requirements are assessed.
- [ ] Multi-region language and timezone requirements are assessed.
- [ ] Mobile-primary closed-loop and offline requirements are assessed.
- [ ] AI generation capability and safety boundary requirements are assessed.
- [ ] Peak traffic and promotional risk are assessed.
- [ ] Audit, traceability, and non-repudiation compliance requirements are assessed.
- [ ] All matched scenario packs are activated and their five asset types are populated.

### 1.3 Five Asset Types Per Scenario

Every scenario pack must contain these five asset categories:

| Asset Type | Purpose | Example |
|-----------|---------|---------|
| Standard | Non-negotiable rules for the domain | "Tenant isolation must cover DB, cache, and object storage" |
| Playbook | Step-by-step operational procedures | "New tenant provisioning and initialization steps" |
| Checklist | Pre-launch verification items | "Cross-tenant access prevention check" |
| Anti-Pattern | Known failure modes to avoid | "Cache keys without tenant dimension" |
| Case Study | Real-world implementation reference | "Single-DB multi-tenant to sharded evolution" |

---

## 2. B2B Enterprise Development Scenario Pack

### 2.1 Standards

- **Organizational Model**: Organization, role, and permission models must be designed with explicit layering (org -> team -> role -> permission).
- **Approval Workflows**: Approval flows must support configurable nodes, conditional branching, and full audit trail.
- **Data Isolation**: Customer data must be logically or physically isolated; cross-customer data access is a P0 defect.
- **Audit Trail**: Every state-changing operation must be logged with actor, timestamp, old value, new value, and IP.
- **SLA Commitments**: Enterprise customers expect contractual SLA; system must measure and report availability per tenant.

### 2.2 Playbook

**Enterprise SSO Integration**
1. Confirm identity provider (SAML 2.0, OIDC, or LDAP).
2. Register application in the customer's IdP; exchange metadata.
3. Implement attribute mapping (groups -> roles).
4. Test login flow with at least 3 role combinations.
5. Enable JIT (Just-In-Time) provisioning or require pre-provisioning.
6. Document logout flow and session timeout behavior.

**Custom Capability Boundary Management**
1. Define the extensibility surface: what can be customized vs. what is locked.
2. Use feature flags or configuration layers -- never branch the main codebase per customer.
3. Document customization contracts with versioning.
4. Customer-specific logic must be deployed without blocking trunk releases.

### 2.3 Checklist

- [ ] Pre-launch permission isolation verified for large enterprise tenants.
- [ ] Audit log completeness verified (actor, action, target, timestamp, result).
- [ ] Approval flow edge cases tested (timeout, rejection, delegation, escalation).
- [ ] SSO integration tested with customer's staging IdP.
- [ ] Data export / portability capability verified (contractual requirement in many B2B deals).
- [ ] Rate limiting and quota management per organization configured.

### 2.4 Anti-Patterns

| Anti-Pattern | Impact | Remedy |
|-------------|--------|--------|
| Hardcoding customer customizations in trunk | Merge conflicts, regression risk, unmaintainable | Use configuration layer or plugin architecture |
| Temporary scripts for long-term permission policies | Security drift, audit failure | Implement policy-as-code with version control |
| Single admin role for all operations | Excessive privilege, compliance violation | Implement RBAC with least-privilege principle |
| No data export capability | Customer lock-in complaints, contract risk | Build export API from day one |

### 2.5 Case Studies

**Multi-Level Approval Procurement Platform**
- Challenge: 5-level approval chain with conditional routing and delegation.
- Solution: Workflow engine with configurable rules, timeout auto-escalation, and full audit trail.
- Result: Approval cycle reduced from 5 days to 1.5 days; audit pass rate 100%.

**Enterprise Tenant Onboarding with Historical Data Migration**
- Challenge: Migrating 2M records from legacy system with schema differences.
- Solution: ETL pipeline with validation gates, rollback checkpoints, and reconciliation reports.
- Result: Zero data loss; migration completed in 4-hour maintenance window.

---

## 3. B2C Consumer Growth Scenario Pack

### 3.1 Standards

- **Conversion Funnel Tracking**: Event tracking must be consistent across the entire funnel (impression -> click -> action -> conversion -> retention).
- **High-Concurrency Protection**: Promotional events must have rate limiting, circuit breaking, and graceful degradation strategies.
- **Experiment Infrastructure**: A/B testing must support clean segmentation, statistical significance validation, and safe rollback.
- **Retention Metrics**: Beyond acquisition, the system must track D1/D7/D30 retention and support cohort analysis.

### 3.2 Playbook

**A/B Experiment Launch & Rollback**
1. Define hypothesis, primary metric, and guardrail metrics.
2. Calculate required sample size for statistical significance.
3. Implement feature flag with random user assignment (sticky bucketing).
4. Launch to 5% traffic; monitor guardrail metrics for 24 hours.
5. Ramp to target percentage if guardrails hold.
6. Run until significance reached; document and ship winner.
7. Rollback: flip feature flag; verify metrics within 1 hour.

**Promotional Traffic Protection**
1. Pre-event: capacity test at 2x expected peak; identify bottlenecks.
2. Configure tiered rate limiting (API gateway -> service -> database).
3. Prepare degradation switches for non-critical features (recommendations, reviews).
4. Event day: dedicated war room with real-time dashboards.
5. Post-event: analyze actual vs. predicted traffic; update capacity model.

### 3.3 Checklist

- [ ] Pre-promotion capacity stress test completed at 2x expected peak.
- [ ] Payment success rate and timeout metrics baselined.
- [ ] Funnel tracking verified end-to-end (no missing events).
- [ ] Degradation switches tested and documented.
- [ ] A/B experiment cleanup: remove losing variants within 1 sprint.
- [ ] Retention tracking configured for post-campaign cohorts.

### 3.4 Anti-Patterns

| Anti-Pattern | Impact | Remedy |
|-------------|--------|--------|
| Tracking only impressions, ignoring retention | Misleading growth metrics | Implement full-funnel tracking including D7/D30 retention |
| Promotional rules scattered across frontend and backend | Inconsistency, customer complaints | Centralize promotion rules in a single service |
| No experiment cleanup | Technical debt, conflicting flags | Enforce experiment TTL and automated cleanup |
| Launching at 100% without ramp | Undetected regressions at scale | Always start at 5% with guardrail monitoring |

### 3.5 Case Studies

**Flash Sale Concurrency Protection Overhaul**
- Challenge: Previous flash sale crashed at 50K QPS due to database hot-spot.
- Solution: Redis-based inventory pre-deduction + async order creation + database batch write.
- Result: Sustained 200K QPS with zero oversell; error rate < 0.01%.

**Recommendation A/B Experiment Driving Conversion**
- Challenge: Recommendation module showed low CTR.
- Solution: Personalized ranking model with A/B framework; tested 3 variants.
- Result: Winner variant increased CTR by 23% and order conversion by 8%.

---

## 4. Multi-Tenant SaaS Scenario Pack

### 4.1 Standards

- **Tenant Isolation**: Must cover all data layers -- database (row-level or schema-level or DB-level), cache (key prefix or separate instance), object storage (bucket or path prefix), and message queues (topic prefix or separate topic).
- **Quota & Rate Limiting**: Must support per-tenant configuration for API rate limits, storage quotas, compute limits, and concurrent user limits.
- **Billing Dimensions**: Usage metering must be accurate per tenant, supporting multiple billing models (seat-based, usage-based, tiered).
- **Tenant Lifecycle**: Provisioning, suspension, reactivation, and deletion must be automated with audit trail.

### 4.2 Playbook

**New Tenant Provisioning**
1. Validate tenant configuration (plan, region, admin user).
2. Create tenant record in control plane database.
3. Provision isolated resources (DB schema/namespace, cache prefix, storage path).
4. Seed default configuration and sample data.
5. Create admin user and send invitation.
6. Run smoke test against tenant-specific endpoints.
7. Enable billing metering.

**Tenant Migration & Consolidation**
1. Freeze source tenant (read-only mode).
2. Export all data with referential integrity preserved.
3. Transform data to target schema if needed.
4. Import to target tenant with validation checksums.
5. Run reconciliation report; resolve discrepancies.
6. Switch DNS / routing to target.
7. Maintain source in read-only for 30-day rollback window.

### 4.3 Checklist

- [ ] Cross-tenant access prevention verified (attempt access with Tenant-A token to Tenant-B data).
- [ ] Tenant-level backup and restore drill completed.
- [ ] Quota enforcement tested at limit boundaries.
- [ ] Billing metering accuracy verified (compare usage logs with billing records).
- [ ] Tenant deletion verified (all data purged, resources released, audit log retained).
- [ ] Noisy-neighbor protection tested (one tenant's spike does not degrade others).

### 4.4 Anti-Patterns

| Anti-Pattern | Impact | Remedy |
|-------------|--------|--------|
| Cache keys without tenant dimension | Cross-tenant data leakage | Enforce `tenant:{id}:` key prefix at SDK level |
| Super-admin operations without audit trail | Compliance violation, trust erosion | Log all admin operations with immutable audit trail |
| Shared connection pool without tenant limits | Noisy-neighbor resource exhaustion | Implement per-tenant connection pooling or priority queuing |
| Hardcoded single-tenant assumptions in ORM | Massive refactoring when adding multi-tenancy | Design tenant context from day one |

### 4.5 Case Studies

**Single-DB Multi-Tenant to Sharded Evolution**
- Challenge: 500+ tenants on single PostgreSQL; largest tenant causing lock contention.
- Solution: Introduced tenant-aware sharding with Citus; largest 10 tenants on dedicated shards.
- Result: P99 latency reduced from 2.3s to 180ms; zero cross-tenant data incidents.

**Dedicated Resource Pool for Enterprise Tenants**
- Challenge: Enterprise customers demanded guaranteed performance SLA.
- Solution: Kubernetes namespace per enterprise tenant with resource quotas and dedicated node pools.
- Result: Contractual SLA met (99.95%); premium pricing justified.

---

## 5. Internationalization Scenario Pack

### 5.1 Standards

- **String Management**: All user-facing text must use locale keys managed in a translation management system (TMS). No hardcoded strings in source code.
- **Format Localization**: Date, time, number, currency, address, and phone formats must use locale-aware formatters (ICU / Intl API).
- **Timezone Handling**: All server-side timestamps stored in UTC. Conversion to local timezone happens at the presentation layer.
- **RTL Support**: Layout must correctly mirror for right-to-left languages (Arabic, Hebrew).

### 5.2 Playbook

**Multi-Language Release & Translation Workflow**
1. Developers use locale keys; never write raw strings.
2. New keys are extracted and pushed to TMS automatically on merge.
3. Translators work in TMS with context screenshots.
4. Translated bundles are pulled into the build pipeline.
5. Pseudo-localization test run to catch truncation and layout issues.
6. Release with fallback chain: user locale -> region default -> English.

**Regional Compliance Onboarding**
1. Identify applicable regulations per region (GDPR, CCPA, PIPL, etc.).
2. Map regulation requirements to feature flags (cookie consent, data residency, right to deletion).
3. Implement consent collection and storage per regulation.
4. Test compliance flow per region with legal review sign-off.
5. Document regional compliance matrix and update quarterly.

### 5.3 Checklist

- [ ] Pseudo-localization test passed (verify no truncation, overlap, or hardcoded strings).
- [ ] RTL layout verified for Arabic/Hebrew locales.
- [ ] Daylight saving time (DST) logic verified for all target timezones.
- [ ] Cross-timezone scheduling logic verified (meeting invites, cron jobs, report generation).
- [ ] Currency conversion and display verified for all supported currencies.
- [ ] Regional compliance requirements mapped and feature flags configured.
- [ ] Fallback locale chain verified (missing translation does not show key name to user).

### 5.4 Anti-Patterns

| Anti-Pattern | Impact | Remedy |
|-------------|--------|--------|
| Hardcoded UI strings in frontend code | Untranslatable, blocks i18n expansion | Enforce lint rule: no string literals in JSX/template |
| Storing timestamps in local timezone on server | Incorrect calculations across timezones | Store UTC; convert at presentation layer |
| Concatenating translated strings | Grammatically incorrect in many languages | Use ICU MessageFormat with placeholders |
| Assuming left-to-right layout | Broken UI for RTL users | Use logical CSS properties (`margin-inline-start` instead of `margin-left`) |

### 5.5 Case Studies

**Multi-Currency Settlement & Reconciliation Overhaul**
- Challenge: 12 currencies with different rounding rules and settlement schedules.
- Solution: Currency service with per-currency rounding config, real-time exchange rate feed, and automated reconciliation.
- Result: Settlement accuracy 99.99%; reconciliation time reduced from 4 hours to 15 minutes.

**Regional Content Strategy Driving Conversion**
- Challenge: Global landing page had low conversion in APAC markets.
- Solution: Region-specific hero content, local testimonials, and locale-appropriate CTA language.
- Result: APAC conversion rate increased by 34%.

---

## 6. Mobile Superapp Scenario Pack

### 6.1 Standards

- **Performance Budgets**: Define and enforce budgets for cold start time (< 2s), memory usage (< 150MB baseline), crash rate (< 0.1%), and ANR rate (< 0.05%).
- **Offline & Retry**: Core flows must work offline with local queue and automatic retry on connectivity restoration.
- **Weak Network Resilience**: All network calls must have timeout, retry with exponential backoff, and graceful degradation.
- **Permission Minimization**: Request only necessary permissions; justify each in privacy manifest.

### 6.2 Playbook

**Staged Rollout & Hotfix**
1. Submit to app store with phased rollout (1% -> 5% -> 20% -> 50% -> 100%).
2. Monitor crash rate, ANR rate, and key metrics at each stage.
3. Hold at each stage for minimum 24 hours.
4. If regression detected: halt rollout and push hotfix.
5. Hotfix path: code-push for JS layer; expedited store review for native layer.
6. Emergency: server-side feature flag to disable problematic feature without app update.

**Client Emergency Rollback & Version Control**
1. Server-side minimum version enforcement: reject requests from deprecated versions.
2. Force-update dialog for critical security patches.
3. Maintain N-2 API version compatibility.
4. Feature flags for all major features; remotely disable without release.

### 6.3 Checklist

- [ ] Device compatibility matrix verified (top 20 devices by user base, min OS versions).
- [ ] Permission requests comply with platform guidelines and are minimized.
- [ ] Offline mode tested: complete core flow without network, sync on reconnect.
- [ ] Weak network tested: 2G simulation, high latency, intermittent connectivity.
- [ ] Cold start time measured on low-end reference device (< 2s target).
- [ ] Memory leak test: 30-minute usage session on low-end device.
- [ ] Deep link / universal link routing verified for all registered paths.
- [ ] Background / foreground transition state management verified.

### 6.4 Anti-Patterns

| Anti-Pattern | Impact | Remedy |
|-------------|--------|--------|
| Full release without staged rollout | Undetected crash affecting all users | Always use phased rollout with monitoring gates |
| No background/foreground state management | Data loss, stale UI, crash on resume | Implement lifecycle-aware state persistence |
| Requesting all permissions at launch | Low install-to-activation rate | Request permissions in context, at point of use |
| No offline capability for core flows | Unusable in elevators, subways, rural areas | Implement offline queue with sync-on-reconnect |

### 6.5 Case Studies

**Offline-First Field Service Application**
- Challenge: Field workers in areas with no connectivity needed to complete inspections.
- Solution: Local SQLite database with conflict-free replicated data types (CRDTs); background sync.
- Result: 100% task completion rate regardless of connectivity; sync conflicts < 0.01%.

**Deep Link Optimization Driving Conversion**
- Challenge: Marketing links opened the app but landed on home page, not target content.
- Solution: Deferred deep linking with attribution; fallback to web if app not installed.
- Result: Deep link conversion rate increased by 41%.

---

## 7. AI Application Scenario Pack

### 7.1 Standards

- **Trust Boundaries**: Model output must include confidence indicators or explicit uncertainty disclaimers. Never present AI output as verified fact.
- **RAG Data Governance**: Retrieval-Augmented Generation data sources must be traceable, updatable, and rollbackable. Source metadata must be attached to every retrieval result.
- **Prompt Versioning**: Prompts are code -- version controlled, reviewed, tested, and deployed through the standard release pipeline.
- **Safety Guardrails**: Input and output must be filtered for prompt injection, PII leakage, and harmful content.

### 7.2 Playbook

**Prompt Version Management & Deployment**
1. Store prompts in version control (alongside code or in dedicated prompt registry).
2. Each prompt change requires a PR with offline evaluation results.
3. Evaluation suite: accuracy, hallucination rate, latency, cost, and safety checks.
4. Deploy via feature flag; A/B test new prompt against baseline.
5. Monitor production metrics for 48 hours before full rollout.
6. Rollback: revert feature flag to previous prompt version.

**AI Incident Response & Human Fallback**
1. Define AI failure modes: hallucination, high latency, safety filter trigger, model unavailability.
2. For each failure mode, implement automated detection and human fallback path.
3. Hallucination: confidence below threshold triggers human review queue.
4. Latency: timeout triggers cached / static fallback response.
5. Safety filter: blocked output triggers human agent escalation.
6. Model down: circuit breaker routes to rule-based fallback.

### 7.3 Checklist

- [ ] Hallucination rate measured on representative test set (target: < 5% for factual queries).
- [ ] End-to-end latency measured (P50, P95, P99) and within budget.
- [ ] Cost per query calculated and within budget.
- [ ] User adoption rate and task completion rate tracked.
- [ ] Prompt injection attack tested (at least 10 known attack patterns).
- [ ] PII detection and redaction verified in both input and output.
- [ ] Human fallback path tested end-to-end.
- [ ] RAG source freshness verified (no stale data beyond defined TTL).

### 7.4 Anti-Patterns

| Anti-Pattern | Impact | Remedy |
|-------------|--------|--------|
| Treating the model as a factual database | Users trust incorrect output | Always show confidence and source attribution |
| Deploying without offline evaluation | Regressions discovered in production | Require evaluation suite pass before merge |
| No human fallback path | Users stuck when AI fails | Implement fallback for every AI-powered feature |
| RAG without source tracking | Unverifiable, non-updatable knowledge | Attach source metadata to every retrieval chunk |
| Prompt changes without version control | Unreproducible behavior, impossible rollback | Treat prompts as code with full CI/CD |

### 7.5 Case Studies

**Intelligent Customer Service with RAG Q&A Loop**
- Challenge: Customer service team overwhelmed; FAQ coverage insufficient.
- Solution: RAG-based Q&A with knowledge base, confidence scoring, and human escalation for low-confidence answers.
- Result: 72% of queries resolved automatically; CSAT maintained at 4.3/5.

**Code Assistant Integrated with Standards Review**
- Challenge: Code review bottleneck; style and security issues caught too late.
- Solution: AI code assistant that suggests fixes inline, referencing team coding standards and security rules.
- Result: PR review cycle reduced by 40%; security issue escape rate reduced by 65%.

---

## 8. Fintech Regulated Scenario Pack

### 8.1 Standards

- **Transaction Integrity**: Transaction chains must satisfy strong consistency and non-repudiation. Every transaction must have an immutable audit record.
- **Risk Control Traceability**: Risk control and anti-fraud strategies must be explainable and traceable. Decision logs must be retained for regulatory examination.
- **Data Classification**: All data fields must be classified (public, internal, confidential, restricted) with corresponding protection measures (encryption at rest, in transit, tokenization).
- **Regulatory Mapping**: Every applicable regulation must be mapped to specific technical controls with evidence of compliance.

### 8.2 Playbook

**Transaction Anomaly Rollback & Compensation**
1. Detect anomaly (timeout, inconsistency, fraud signal).
2. Halt affected transaction chain; log freeze point.
3. Determine compensation strategy: reverse transaction, credit adjustment, or manual review.
4. Execute compensation with separate audit trail.
5. Reconcile: verify source and destination balances match expected state.
6. Generate incident report with timeline and resolution evidence.

**Compliance Audit Evidence & Reporting**
1. Define audit scope (regulation, time period, data domain).
2. Extract relevant logs, transaction records, and decision trails.
3. Generate compliance report with required format and fields.
4. Internal review and sign-off before submission.
5. Archive evidence package with tamper-evident hash.

### 8.3 Checklist

- [ ] Sensitive data classification and masking/encryption verified for all fields.
- [ ] Key account reconciliation completed; discrepancies resolved within SLA.
- [ ] Transaction audit trail immutability verified (append-only, hash-chained).
- [ ] Risk control decision logs retained for required period (typically 5-7 years).
- [ ] Fraud detection rules tested with known attack patterns.
- [ ] Regulatory reporting capability verified (format, timing, content).
- [ ] Disaster recovery drill completed within target RTO/RPO.

### 8.4 Anti-Patterns

| Anti-Pattern | Impact | Remedy |
|-------------|--------|--------|
| No unified transaction ID across services | Impossible to trace end-to-end | Implement distributed tracing with correlation ID |
| Compliance logs missing critical fields | Audit failure, regulatory penalty | Define and enforce log schema per regulation |
| Manual reconciliation | Error-prone, slow, unscalable | Automate reconciliation with exception-based human review |
| Risk rules as hardcoded if-else | Inflexible, slow to update, impossible to explain | Use rules engine with version control and decision logging |

### 8.5 Case Studies

**Payment Timeout Compensation Mechanism**
- Challenge: 0.3% of payments timed out with inconsistent state between payment provider and internal ledger.
- Solution: Saga pattern with compensation steps; automated reconciliation job every 5 minutes.
- Result: Inconsistency resolution time reduced from 24 hours to 5 minutes; zero financial loss.

**Real-Time Risk Control Strategy Evolution**
- Challenge: Batch fraud detection had 6-hour delay; losses occurred before detection.
- Solution: Stream processing (Kafka + Flink) with real-time rule evaluation and ML scoring.
- Result: Fraud detection latency reduced from 6 hours to < 200ms; fraud loss reduced by 78%.

---

## 9. E-commerce Peak Scenario Pack

### 9.1 Standards

- **Consistency Strategy**: Inventory, order, and payment must have explicitly defined consistency levels (strong for inventory deduction, eventual for non-critical updates).
- **Tiered Rate Limiting**: Peak traffic must be managed with multi-layer rate limiting (CDN -> API Gateway -> Service -> Database).
- **Elastic Scaling**: Auto-scaling policies must be pre-configured and tested before peak events.
- **Degradation Switches**: Every non-critical feature must have a kill switch for graceful degradation during peak.

### 9.2 Playbook

**Peak Event War Room & Emergency Response**
1. T-7 days: capacity test at 2x projected peak; fix bottlenecks.
2. T-3 days: enable pre-warming for CDN, cache, and connection pools.
3. T-1 day: war room setup with dashboards, communication channels, and escalation roster.
4. T-0: real-time monitoring with 1-minute granularity; designated decision-maker for degradation switches.
5. T+1 hour: post-peak review; document actual vs. predicted metrics.
6. T+1 day: full post-mortem with improvement items.

**Inventory Hotspot Protection & Replenishment**
1. Identify hot-spot SKUs (top 0.1% by expected demand).
2. Pre-load hot-spot inventory into Redis with per-SKU rate limiting.
3. Implement two-phase inventory: Redis pre-deduction -> async DB confirmation.
4. Replenishment: monitor Redis inventory; trigger replenishment when below threshold.
5. Oversell protection: if DB confirmation fails, release Redis reservation and notify user.

### 9.3 Checklist

- [ ] Peak capacity stress test completed at 2x projected traffic.
- [ ] Capacity redundancy verified (compute, database connections, cache memory).
- [ ] Core flow degradation switches tested and documented.
- [ ] CDN and cache pre-warming completed for static and hot-spot content.
- [ ] Inventory hot-spot protection verified (no oversell under concurrent load).
- [ ] Payment timeout and retry logic verified under high latency.
- [ ] Auto-scaling policy tested and verified (scale-up time < 3 minutes).
- [ ] War room communication and escalation chain verified.

### 9.4 Anti-Patterns

| Anti-Pattern | Impact | Remedy |
|-------------|--------|--------|
| Promotional rules hardcoded and not hot-updatable | Cannot adjust during peak; wrong prices | Store rules in config service with hot-reload |
| Hot-spot cache key without protection | Cache avalanche / stampede | Implement per-key rate limiting and async refresh |
| No degradation switches for non-critical features | Everything fails together | Feature-flag every non-core feature; test kill switches |
| Testing at 1x expected peak | No headroom for traffic spikes | Always test at 2x minimum; 3x for critical events |

### 9.5 Case Studies

**Flash Sale Inventory Oversell Prevention**
- Challenge: Previous flash sale oversold 1,200 units due to database race condition.
- Solution: Redis Lua script for atomic inventory deduction; async order creation with compensation on failure.
- Result: Zero oversell across 5 subsequent flash sales handling 300K QPS.

**Peak Payment Success Rate Defense**
- Challenge: Payment success rate dropped to 89% during Double-11 due to upstream timeout.
- Solution: Circuit breaker per payment channel; automatic failover to secondary channel; increased timeout budget.
- Result: Payment success rate maintained at 99.7% during subsequent peak.

---

## Agent Checklist

- [ ] Identify all applicable scenario packs using the decision tree.
- [ ] Verify all five asset types (standard, playbook, checklist, anti-pattern, case) are populated for each selected pack.
- [ ] Apply standards from selected packs as hard constraints in architecture and implementation.
- [ ] Execute playbook steps for relevant operational procedures.
- [ ] Complete all checklist items before launch.
- [ ] Cross-reference anti-pattern catalog during code review.
- [ ] Reference case studies when evaluating solution approaches.
- [ ] Re-evaluate scenario selection when business requirements change.
