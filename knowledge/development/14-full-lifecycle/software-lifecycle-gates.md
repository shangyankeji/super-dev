---
id: software-lifecycle-gates
title: Software Lifecycle Gates - Comprehensive Quality Gate Reference
domain: development
category: 14-full-lifecycle
difficulty: intermediate
tags: [architecture, decision, design, development, discovery, end-to-end, gate, gates]
quality_score: 70
last_updated: 2026-06-15
---
# Software Lifecycle Gates - Comprehensive Quality Gate Reference

> Consolidated reference covering the end-to-end software development lifecycle: requirement discovery, product-design handoff, architecture decision, implementation execution, testing verification, security compliance, release management, operations observability, incident postmortem, and stage exit criteria.

---

## 1. Lifecycle End-to-End Map

### 1.1 Stages Overview

The full software development lifecycle consists of 9 stages, each with defined inputs, outputs, gates, and responsible roles:

```
Requirement    Product &     Architecture   Implementation   Testing
Discovery  ->  Design    ->  Decision    ->  Execution    ->  Verification
   |              |              |               |               |
   v              v              v               v               v
[Scope Doc]  [UI States]    [ADR]          [Merged Code]   [Test Report]
[Acceptance] [Tracking]     [Scale Plan]   [Test Evidence] [Perf Result]
[Risk Reg.]  [Handoff]      [Rollback]     [PR Review]     [Bug Closure]
                                                                |
                                                                v
Security       Release &      Operations     Incident
Compliance ->  Change Mgmt -> Observability -> Postmortem
   |              |              |               |
   v              v              v               v
[Vuln Scan]  [Change Ticket] [SLO Dashboard] [Postmortem]
[Perm Audit] [Rollout Record][Alert Policy]  [Action Items]
[Compliance] [Verification]  [Runbook]       [Prevention]
```

### 1.2 Governing Principles

1. **Stage Gate Enforcement**: Each stage has explicit exit criteria. The next stage must not begin until the current stage's gate is passed.
2. **Traceability**: Every decision must be traceable and replayable. Link requirements to code, code to tests, tests to release.
3. **Responsibility Assignment**: Every stage has a designated owner. Ownership must be documented and acknowledged.
4. **Continuous Feedback**: Later stages feed improvements back to earlier stages (postmortem -> requirement standards, operations -> architecture).

---

## 2. Stage 1: Requirement Discovery

### 2.1 Inputs

- Business objectives with measurable targets (revenue, efficiency, compliance, etc.).
- Constraint conditions (budget, timeline, team capacity, technology stack).
- Target users and their primary contexts.
- Key success metrics and how they will be measured.

### 2.2 Process

**Step 1: User Task Decomposition**
- Identify the primary user roles and their goals.
- Map each goal to a task flow: trigger -> steps -> outcome.
- Identify the critical path (the shortest flow to core value).
- Document alternative paths and edge cases.

**Step 2: Non-Functional Requirement Identification**
- Performance: response time budgets, throughput targets, concurrency limits.
- Reliability: availability target (e.g., 99.9%), RTO, RPO.
- Security: data classification, authentication requirements, compliance regulations.
- Scalability: expected growth trajectory and scaling strategy.
- Accessibility: WCAG compliance level, supported assistive technologies.

**Step 3: Acceptance Criteria Definition**
- Each requirement must have testable acceptance criteria using Given-When-Then or equivalent format.
- Acceptance criteria must cover both happy path and failure paths.
- Non-functional requirements must have measurable thresholds.

**Step 4: Risk Identification**
- Technical risks: unfamiliar technology, integration complexity, performance uncertainty.
- Business risks: market timing, regulatory changes, dependency on third parties.
- Each risk must have: likelihood, impact, mitigation plan, and owner.

### 2.3 Outputs

| Output | Description | Quality Standard |
|--------|-------------|-----------------|
| Scope Document | Business goals, user roles, task flows, boundaries | Reviewed by PM, Tech Lead, and stakeholder |
| Acceptance Criteria | Testable conditions for every requirement | Mapped 1:1 to requirement items |
| Risk Register | Identified risks with mitigation plans | Each risk has owner and review date |

### 2.4 Exit Criteria

- [ ] Scope document reviewed and signed off by all stakeholders.
- [ ] All requirements have testable acceptance criteria.
- [ ] Risk register contains all identified risks with mitigation plans.
- [ ] Non-functional requirements have measurable thresholds.
- [ ] Dependencies on external teams or systems are documented and acknowledged.

---

## 3. Stage 2: Product & Design Handoff

### 3.1 Handoff Checklist

The design-to-engineering handoff must include all of the following:

**Interaction Completeness**
- [ ] User flow diagrams cover all primary and exception paths.
- [ ] Edge cases documented: empty state, error state, loading state, permission-denied state.
- [ ] State transitions defined with trigger conditions.
- [ ] Responsive behavior specified for all target breakpoints.

**Visual Completeness**
- [ ] Visual designs cover all states: default, hover, focus, active, disabled, error, empty, loading, success.
- [ ] Dark mode / theme variants included if applicable.
- [ ] Component-to-token mapping documented (which token drives which visual property).

**Content & Tracking**
- [ ] All UI copy finalized and reviewed.
- [ ] Analytics event plan defined (event name, properties, trigger condition).
- [ ] Permission rules documented (who sees what, conditional visibility).

**Engineering Alignment**
- [ ] Component reuse identified (which existing components to use, which to create).
- [ ] Token references verified (all visual values trace to design tokens).
- [ ] Design acceptance criteria defined and testable.
- [ ] Change impact assessment completed (which pages / flows are affected).
- [ ] Version / release strategy confirmed.

### 3.2 Handoff Quality Standard

- No design file should be "handed off" without a 30-minute walkthrough with the implementing engineer.
- Engineer must confirm understanding by restating the critical path and key edge cases.
- Open questions must be logged and resolved within 24 hours.

---

## 4. Stage 3: Architecture Decision Gate

### 4.1 Mandatory Review Items

Every architecture decision must address these four dimensions:

**Scalability & Performance**
- What is the expected load in 6 months? 12 months?
- What is the scaling strategy (horizontal, vertical, sharding)?
- What are the performance budgets (latency P50/P95/P99, throughput)?
- Where are the bottleneck risks and what are the mitigations?

**Availability & Disaster Recovery**
- What is the availability target and corresponding error budget?
- What is the disaster recovery strategy (active-active, active-passive, cold standby)?
- What are the RTO and RPO targets?
- How is data replicated and what is the consistency model?

**Security & Access Control**
- What is the authentication mechanism?
- What is the authorization model (RBAC, ABAC, policy-based)?
- How are secrets managed?
- What data needs encryption at rest and in transit?

**Observability & Alerting**
- What metrics, logs, and traces are collected?
- What are the key SLIs and SLOs?
- What is the alerting hierarchy (P0 -> immediate page, P1 -> 15 min, P2 -> next business day)?
- What dashboards are required?

### 4.2 Decision Artifacts

Every architecture decision must produce:

| Artifact | Content | Retention |
|----------|---------|-----------|
| ADR (Architecture Decision Record) | Context, options considered, decision rationale, consequences | Permanent (version controlled) |
| Trade-off Analysis | Comparison matrix with weighted criteria | Attached to ADR |
| Rollback / Migration Plan | Steps to revert or migrate if the decision proves wrong | Attached to ADR |
| Dependency Map | Upstream and downstream system dependencies | Updated per release |

### 4.3 Exit Criteria

- [ ] ADR written, reviewed by architecture review board, and merged.
- [ ] Scalability plan documented with growth projections.
- [ ] Rollback plan documented and feasible.
- [ ] Security model reviewed by security team.
- [ ] Observability plan reviewed by operations team.

---

## 5. Stage 4: Implementation Execution

### 5.1 Execution Rules

**Task Management**
- Tasks must be decomposed to 1-3 day units, each with a clear definition of done.
- Branch strategy must be documented and traceable (branch name maps to task/ticket ID).
- Work-in-progress (WIP) limits must be enforced (max 2 active tasks per developer).

**Code Quality**
- All production code must be covered by automated tests before merge.
- Main branch code must always be in a shippable state.
- Static analysis (lint, type check) must pass before PR review.

**Pull Request Standards**
- Every PR must include:
  - Link to the requirement / task.
  - Summary of what changed and why.
  - Test evidence (screenshots, test output, or coverage report).
  - Risk assessment (what could go wrong, what was tested).
  - Rollback instructions if the change needs to be reverted.

### 5.2 Quality Actions

| Action | Timing | Gate |
|--------|--------|------|
| Static analysis (lint + type check) | Pre-commit / CI | Must pass |
| Unit tests | Pre-commit / CI | Must pass, coverage >= threshold |
| Integration tests | CI | Must pass for affected modules |
| Code review | Before merge | At least 1 approval from qualified reviewer |
| Regression test | Before release | All regression suites pass |
| Security scan | CI | No critical / high vulnerabilities |

### 5.3 Critical Logic Requirements

- Business-critical logic (payment, authorization, data mutation) must have:
  - Dedicated regression test cases covering success, failure, and edge paths.
  - Explicit error handling with recovery or compensation.
  - Audit logging for all state changes.
  - Code review by a senior engineer or domain expert.

### 5.4 Exit Criteria

- [ ] All task code merged to main branch.
- [ ] All automated tests pass (unit, integration, regression).
- [ ] PR reviews completed with all comments resolved.
- [ ] Static analysis and security scan pass.
- [ ] Test evidence archived as build artifacts.

---

## 6. Stage 5: Testing & Verification Gate

### 6.1 Coverage Scope

Testing must cover five dimensions:

| Dimension | Focus | Minimum Requirement |
|-----------|-------|-------------------|
| Functional | Feature correctness per acceptance criteria | All acceptance criteria have corresponding test cases |
| Regression | No existing functionality broken | Full regression suite pass |
| Performance | Meets latency, throughput, and resource budgets | Load test at 2x expected peak |
| Security | No exploitable vulnerabilities | DAST/SAST scan pass, penetration test for critical flows |
| Compatibility | Works on target platforms / browsers / devices | Matrix verification for top 80% user agents |

### 6.2 Test Path Coverage

- Every critical user flow must have test cases covering:
  - Success path (happy path).
  - Failure path (invalid input, network error, timeout).
  - Edge path (boundary values, concurrent access, resource exhaustion).

### 6.3 Exit Criteria

- [ ] Zero blocking (P0) defects open.
- [ ] High-risk test cases: 100% pass rate.
- [ ] Smoke test suite: 100% pass.
- [ ] Regression test suite: 100% pass.
- [ ] Performance test results within budget.
- [ ] Staged verification (if applicable): canary / gray release verification pass.
- [ ] Test report generated and archived.

---

## 7. Stage 6: Security & Compliance Gate

### 7.1 Mandatory Checks

| Check Area | Requirement | Evidence |
|-----------|-------------|---------|
| Data Classification | All data fields classified (public, internal, confidential, restricted) | Classification matrix document |
| Data Protection | Confidential/restricted data encrypted at rest and in transit | Encryption configuration verification |
| Masking / Tokenization | PII masked in logs, test environments, and non-production displays | Log sampling verification |
| Permission Model | Least-privilege principle enforced; no excessive permissions | Permission audit report |
| Audit Logging | All state-changing operations logged with immutable trail | Audit log completeness check |
| Dependency Security | No known critical/high CVEs in production dependencies | Dependency scan report (Trivy, npm audit, etc.) |
| Compliance Mapping | Applicable regulations mapped to technical controls | Compliance matrix with evidence links |

### 7.2 Exit Criteria

- [ ] Zero critical (CVSS >= 9.0) vulnerabilities.
- [ ] Zero high (CVSS >= 7.0) vulnerabilities without approved mitigation plan.
- [ ] All mitigation plans have owner and deadline (max 30 days for high).
- [ ] Permission audit completed and signed off.
- [ ] Compliance mapping reviewed by legal / compliance team.
- [ ] Security scan report archived as release artifact.

---

## 8. Stage 7: Release & Change Management

### 8.1 Release Strategy

**Principles**
- Small batches, frequent releases, with gradual rollout.
- Every release must have a rollback plan that can execute in < 15 minutes.
- Critical feature flags must support instant kill-switch.

**Rollout Pattern**
1. Canary: 1-2% of traffic for initial validation (minimum 1 hour).
2. Early adopter: 5-10% for broader signal (minimum 4 hours).
3. Partial: 25-50% for confidence building (minimum 24 hours).
4. Full: 100% with enhanced monitoring for 48 hours.

### 8.2 Change Control

- Every production change must have a change ticket containing:
  - Change description and business justification.
  - Impact assessment (systems, users, data).
  - Rollback procedure with step-by-step instructions.
  - Approval from change manager and tech lead.
- Release windows must have:
  - On-call engineer assigned.
  - Emergency communication channel established.
  - Escalation path documented.

### 8.3 Post-Release Verification

- Within 30 minutes of full rollout:
  - [ ] Core business metrics stable (within +/- 5% of baseline).
  - [ ] Error rates within normal bounds.
  - [ ] No new alerts triggered.
  - [ ] Latency P95/P99 within budget.
- Enhanced monitoring period: 48 hours with lowered alert thresholds.

### 8.4 Exit Criteria

- [ ] Change ticket approved and linked to release.
- [ ] Staged rollout completed per pattern.
- [ ] Post-release verification passed.
- [ ] Rollback plan verified (tested in staging or documented from previous rollback).
- [ ] Release record archived with rollout timeline and verification results.

---

## 9. Stage 8: Operations & Observability

### 9.1 Observability Stack

Three pillars of observability must be implemented:

| Pillar | Purpose | Implementation |
|--------|---------|---------------|
| Metrics | Quantitative measurement of system health | Prometheus / CloudWatch / Datadog with SLI definitions |
| Logs | Detailed event records for debugging | Structured JSON logs with correlation IDs, shipped to central log system |
| Traces | Request flow across services | Distributed tracing (OpenTelemetry / Jaeger / X-Ray) |

### 9.2 SLO & Error Budget

- Define SLOs for each critical service:
  - Availability: e.g., 99.95% measured over 30-day rolling window.
  - Latency: e.g., P99 < 500ms for API endpoints.
  - Error rate: e.g., < 0.1% 5xx responses.
- Error budget = 100% - SLO target. When error budget is exhausted:
  - Freeze non-critical deployments.
  - Prioritize reliability work until budget recovers.

### 9.3 Alerting Strategy

| Severity | Response Time | Channel | Example |
|----------|--------------|---------|---------|
| P0 - Critical | Immediate (< 5 min) | Phone + PagerDuty | Service down, data loss, security breach |
| P1 - High | < 15 min | Slack + PagerDuty | Degraded performance, elevated error rate |
| P2 - Medium | < 4 hours | Slack alert channel | Non-critical feature failure, approaching capacity |
| P3 - Low | Next business day | Email / ticket | Cosmetic issue, minor log anomaly |

Rules:
- Alert on symptoms (user-facing impact), not causes.
- Every alert must have a runbook link.
- Alert fatigue review: monthly audit of alert volume and signal-to-noise ratio.

### 9.4 Runbook Standards

Every production service must have a runbook containing:
- Service overview: purpose, dependencies, SLOs.
- Health check endpoints and expected responses.
- Common failure modes and resolution steps.
- Scaling procedures (manual and automated).
- Restart / recovery procedures.
- Contact list and escalation path.

### 9.5 Post-Change Observation

- After any production change, enhanced observation for 24 hours:
  - Lower alert thresholds by 20%.
  - Monitor new-code-path metrics specifically.
  - On-call engineer must acknowledge the change and confirm observation setup.

### 9.6 Exit Criteria

- [ ] SLO dashboard operational for all critical services.
- [ ] Alert policies configured and tested.
- [ ] Runbooks documented for all production services.
- [ ] On-call rotation established and acknowledged.
- [ ] Log and trace retention meets compliance requirements.

---

## 10. Stage 9: Incident Postmortem & Learning Loop

### 10.1 Postmortem Structure

Every significant incident (P0 or P1) must produce a postmortem within 5 business days:

**Section 1: Event Timeline**
- Detection time and method (alert, user report, monitoring).
- First response time and responder.
- Key decision points during incident.
- Resolution time and method.
- Communication timeline (internal and external).

**Section 2: Impact Assessment**
- User impact: number of affected users, duration, severity.
- Business impact: revenue loss, SLA breach, reputation damage.
- Data impact: any data loss or corruption.

**Section 3: Root Cause Chain**
- Direct cause: the specific failure that triggered the incident.
- Contributing causes: conditions that allowed the direct cause to have impact.
- Systemic cause: organizational or process gaps that created the contributing conditions.

### 10.2 Action Items

Every postmortem must produce categorized action items:

| Category | Timeline | Example |
|----------|----------|---------|
| Immediate Fix | 1-3 days | Patch the specific bug, restore data |
| Short-term Prevention | 1-2 weeks | Add monitoring, improve alert, add test case |
| Long-term Prevention | 1-3 months | Architecture improvement, process change, training |

Rules:
- Every action item has an owner and a deadline.
- Prevention items should be fed back into standards or gate rules (e.g., a new checklist item, a new anti-pattern entry).
- Action items are tracked in the issue system and reviewed weekly until closed.

### 10.3 Learning Loop

- Monthly: review incident trends (frequency, severity, category, MTTR).
- Quarterly: aggregate learnings into knowledge base updates.
- Annually: review systemic patterns and invest in structural improvements.
- Blameless culture: focus on systems and processes, not individuals.

### 10.4 Exit Criteria

- [ ] Postmortem document completed within 5 business days.
- [ ] All action items logged with owner and deadline.
- [ ] Prevention items mapped to standards, gates, or checklists.
- [ ] Monthly trend review conducted.
- [ ] Quarterly knowledge base update completed.

---

## 11. Stage Exit Criteria Summary (YAML Reference)

```yaml
stage_exit_criteria:
  requirement:
    required_outputs: [scope_doc, acceptance_criteria, risk_register]
    gate_owner: Product Manager
  design:
    required_outputs: [user_flow, ui_states, tracking_plan]
    gate_owner: Design Lead
  architecture:
    required_outputs: [adr, scalability_plan, rollback_plan]
    gate_owner: Tech Lead / Architect
  implementation:
    required_outputs: [merged_code, test_evidence, pr_review]
    gate_owner: Tech Lead
  testing:
    required_outputs: [regression_report, performance_result, bug_closure]
    gate_owner: QA Lead
  security:
    required_outputs: [vulnerability_scan, permission_audit, compliance_check]
    gate_owner: Security Engineer
  release:
    required_outputs: [change_ticket, rollout_record, verification_result]
    gate_owner: Release Manager
  operations:
    required_outputs: [slo_dashboard, alert_policy, runbook]
    gate_owner: SRE / DevOps Lead
  incident_learning:
    required_outputs: [postmortem, action_items, prevention_updates]
    gate_owner: Incident Commander
```

---

## Agent Checklist

- [ ] Verify current lifecycle stage and confirm all prior stage exit criteria are met.
- [ ] For requirement stage: confirm scope doc, acceptance criteria, and risk register exist.
- [ ] For design handoff: walk through the handoff checklist with the implementing engineer.
- [ ] For architecture: verify ADR exists with scalability plan, rollback plan, and security review.
- [ ] For implementation: verify PR standards (link, summary, test evidence, risk, rollback).
- [ ] For testing: verify zero blocking defects and 100% high-risk test pass rate.
- [ ] For security: verify zero critical vulnerabilities and compliance mapping complete.
- [ ] For release: verify change ticket, staged rollout, and post-release verification.
- [ ] For operations: verify SLO dashboard, alert policies, and runbooks are in place.
- [ ] For postmortem: verify action items are logged, owned, and tracked to closure.
- [ ] Cross-reference stage exit criteria YAML when validating gate passage.
