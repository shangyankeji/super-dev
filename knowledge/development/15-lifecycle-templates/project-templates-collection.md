---
id: project-templates-collection
title: Project Templates Collection - Comprehensive Lifecycle Templates
domain: development
category: 15-lifecycle-templates
difficulty: intermediate
tags: [architecture, catalog, collection, design, development, handoff, project, requirement]
quality_score: 70
last_updated: 2026-06-15
---
# Project Templates Collection - Comprehensive Lifecycle Templates

> Consolidated collection of all lifecycle stage templates: requirement, design handoff, architecture ADR, implementation PR, testing report, security compliance, release change, operations runbook, postmortem, and lifecycle review board. Each template is ready-to-use with structured sections and guidance notes.

---

## Template Catalog

| ID | Template | Lifecycle Stage | Purpose |
|----|----------|----------------|---------|
| T-01 | Requirement Template | Requirement Discovery | Capture business goals, user scenarios, acceptance criteria, and risks |
| T-02 | Design Handoff Template | Product & Design | Ensure complete design-to-engineering transfer |
| T-03 | Architecture ADR Template | Architecture Decision | Record architecture decisions with context and trade-offs |
| T-04 | Implementation PR Template | Implementation Execution | Standardize pull request content for quality review |
| T-05 | Testing Report Template | Testing & Verification | Document test coverage, results, and release recommendation |
| T-06 | Security Compliance Template | Security & Compliance | Record security assessment and compliance status |
| T-07 | Release Change Template | Release & Change Management | Document release plan, rollout strategy, and verification |
| T-08 | Operations Runbook Template | Operations & Observability | Provide operational reference for production services |
| T-09 | Postmortem Template | Incident Learning | Structure incident review and improvement actions |
| T-10 | Lifecycle Review Board Template | Cross-Stage Governance | Track overall project progress across all lifecycle stages |

---

## T-01: Requirement Template

### Document Header

```
Project:        [Project Name]
Version:        [Document Version]
Author:         [Author Name]
Date:           [YYYY-MM-DD]
Status:         [Draft | Review | Approved]
Stakeholders:   [List of stakeholders]
```

### 1. Business Objective

**Primary Goal**
- Describe the business outcome this requirement serves.
- Include measurable target: [Metric] from [Current] to [Target] by [Date].

**Business Context**
- Why now? What market condition, user feedback, or strategic priority drives this?
- What is the cost of not doing this?

**Success Metrics**
| Metric | Current Baseline | Target | Measurement Method |
|--------|-----------------|--------|-------------------|
| [e.g., Conversion Rate] | [e.g., 3.2%] | [e.g., 5.0%] | [e.g., Analytics funnel] |

**Business Boundary**
- What is explicitly out of scope?
- What constraints exist (budget, timeline, technical, legal)?

### 2. User Scenarios

**Target Users**
| User Role | Description | Primary Goal | Frequency |
|-----------|-------------|-------------|-----------|
| [e.g., Admin] | [Description] | [Goal] | [Daily/Weekly] |

**Primary Path (Happy Path)**
```
Trigger: [What initiates the flow]
Step 1: [User action] -> [System response]
Step 2: [User action] -> [System response]
...
Outcome: [Expected end state]
```

**Alternative Paths**
- Path A: [Description of alternative flow and when it applies]
- Path B: [Description of alternative flow and when it applies]

**Exception Paths**
- Exception 1: [Error condition] -> [Expected system behavior] -> [User recovery path]
- Exception 2: [Error condition] -> [Expected system behavior] -> [User recovery path]

### 3. Acceptance Criteria

| ID | Scenario | Given | When | Then | Priority |
|----|----------|-------|------|------|----------|
| AC-01 | [Scenario name] | [Precondition] | [Action] | [Expected result] | MUST |
| AC-02 | [Scenario name] | [Precondition] | [Action] | [Expected result] | MUST |
| AC-03 | [Scenario name] | [Precondition] | [Action] | [Expected result] | SHOULD |

**Non-Functional Acceptance Criteria**
| Category | Requirement | Threshold | Measurement |
|----------|-------------|-----------|-------------|
| Performance | Page load time | < 2s P95 | Lighthouse / RUM |
| Availability | Uptime | 99.9% | Monitoring system |
| Accessibility | WCAG compliance | Level AA | axe-core scan |
| Security | Vulnerability scan | Zero critical/high | SAST/DAST |

### 4. Risk Register

| ID | Risk | Likelihood | Impact | Mitigation | Owner | Review Date |
|----|------|-----------|--------|------------|-------|-------------|
| R-01 | [Risk description] | [H/M/L] | [H/M/L] | [Mitigation plan] | [Name] | [Date] |

**Dependencies**
| Dependency | Type | Owner | Status | Risk if Delayed |
|-----------|------|-------|--------|----------------|
| [External API] | External | [Team] | [Confirmed] | [Impact description] |

---

## T-02: Design Handoff Template

### Document Header

```
Feature:        [Feature Name]
Designer:       [Designer Name]
Engineer:       [Assigned Engineer]
Date:           [YYYY-MM-DD]
Design File:    [Link to Figma/Sketch/etc.]
```

### 1. Interaction Scope

**Page / Screen List**
| Page | New / Modified | Complexity | Notes |
|------|---------------|------------|-------|
| [Page name] | [New] | [High/Med/Low] | [Special considerations] |

**State Coverage Matrix**
| Page / Component | Default | Hover | Focus | Active | Disabled | Loading | Error | Empty | Success |
|-----------------|---------|-------|-------|--------|----------|---------|-------|-------|---------|
| [Component A] | Y | Y | Y | Y | Y | Y | Y | N/A | Y |

**Flow Diagram Reference**
- Primary flow: [Link or description]
- Exception flows: [Link or description]

### 2. Visual Specification

**Component & Token Mapping**
| Component | Token Reference | Custom Styling | Notes |
|-----------|----------------|---------------|-------|
| [Button CTA] | `button-primary` | None | Standard token |
| [Status Badge] | `badge-success` | Custom radius | See design note |

**Responsive Breakpoints**
| Breakpoint | Width | Layout Change |
|-----------|-------|---------------|
| Desktop | >= 1280px | Full layout |
| Tablet | 768-1279px | Collapsed sidebar |
| Mobile | < 768px | Single column, bottom nav |

**Animation / Motion**
| Interaction | Animation | Duration | Easing |
|-------------|-----------|----------|--------|
| [Modal open] | [Fade + scale] | [200ms] | [ease-out] |

### 3. Engineering Alignment

**Analytics Event Plan**
| Event Name | Trigger | Properties | Priority |
|-----------|---------|------------|----------|
| [feature_viewed] | [Page load] | [page_id, user_role] | [MUST] |

**Permission Rules**
| Element | Visibility Condition | Behavior When Hidden |
|---------|---------------------|---------------------|
| [Admin panel] | [role == admin] | [Not rendered] |

**Handoff Acceptance Criteria**
- [ ] All states implemented per state coverage matrix.
- [ ] Responsive behavior verified at all breakpoints.
- [ ] Token usage verified (no hardcoded values).
- [ ] Analytics events firing correctly.
- [ ] Accessibility: keyboard navigation and screen reader verified.

---

## T-03: Architecture ADR Template

### Document Header

```
ADR-[Number]:   [Decision Title]
Date:           [YYYY-MM-DD]
Status:         [Proposed | Accepted | Superseded | Deprecated]
Deciders:       [List of decision makers]
```

### 1. Decision Context

**Problem Statement**
- What problem are we trying to solve?
- What are the current pain points or limitations?

**Constraints**
- Technical: [Technology constraints, compatibility requirements]
- Business: [Budget, timeline, team skill constraints]
- Compliance: [Regulatory or security constraints]

**Assumptions**
- [Assumption 1 and its basis]
- [Assumption 2 and its basis]

### 2. Options Considered

**Option A: [Name]**
| Dimension | Assessment |
|-----------|-----------|
| Description | [How it works] |
| Pros | [Advantages] |
| Cons | [Disadvantages] |
| Cost | [Implementation and operational cost] |
| Risk | [Key risks] |
| Timeline | [Estimated implementation time] |

**Option B: [Name]**
| Dimension | Assessment |
|-----------|-----------|
| Description | [How it works] |
| Pros | [Advantages] |
| Cons | [Disadvantages] |
| Cost | [Implementation and operational cost] |
| Risk | [Key risks] |
| Timeline | [Estimated implementation time] |

**Comparison Matrix**
| Criteria | Weight | Option A | Option B |
|----------|--------|----------|----------|
| [Scalability] | [30%] | [Score] | [Score] |
| [Maintainability] | [25%] | [Score] | [Score] |
| [Performance] | [20%] | [Score] | [Score] |
| [Cost] | [15%] | [Score] | [Score] |
| [Time to implement] | [10%] | [Score] | [Score] |
| **Weighted Total** | **100%** | **[Total]** | **[Total]** |

### 3. Decision

**Chosen Option**: [Option Name]

**Rationale**: [1-3 sentences explaining why this option was selected]

**Trade-offs Accepted**: [What we are giving up and why it is acceptable]

### 4. Consequences

**Positive**
- [Expected benefit 1]
- [Expected benefit 2]

**Negative**
- [Known limitation 1 and how we will manage it]
- [Known limitation 2 and how we will manage it]

### 5. Risk & Rollback

**Risk Assessment**
| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| [Risk 1] | [H/M/L] | [H/M/L] | [Mitigation plan] |

**Rollback Plan**
- Trigger condition: [When would we consider reverting this decision?]
- Rollback steps: [Step-by-step procedure]
- Estimated rollback time: [Duration]
- Data migration (if applicable): [Strategy]

---

## T-04: Implementation PR Template

### PR Header

```
Title:          [Short description of the change]
Ticket:         [Link to requirement/task]
Type:           [Feature | Bugfix | Refactor | Hotfix | Chore]
```

### 1. Change Overview

**What Changed**
- [Summary of the change in 2-3 sentences]
- Affected modules: [List of modules / packages / services]

**Why**
- [Business or technical motivation]
- [Link to requirement or ADR if applicable]

### 2. Quality Evidence

**Test Results**
| Test Type | Status | Coverage | Notes |
|-----------|--------|----------|-------|
| Unit tests | [Pass/Fail] | [Coverage %] | [New tests added] |
| Integration tests | [Pass/Fail] | [N/A or Coverage %] | [Notes] |
| Manual testing | [Pass/Fail] | [N/A] | [What was tested manually] |

**Screenshots / Recordings** (for UI changes)
| State | Before | After |
|-------|--------|-------|
| [Default] | [Screenshot] | [Screenshot] |
| [Error] | [Screenshot] | [Screenshot] |

**Static Analysis**
- Lint: [Pass/Fail]
- Type check: [Pass/Fail]
- Security scan: [Pass/Fail]

### 3. Risk Assessment

**What Could Go Wrong**
| Risk | Likelihood | Mitigation |
|------|-----------|------------|
| [Risk description] | [H/M/L] | [How it was mitigated or tested] |

**Deployment Considerations**
- Database migration required: [Yes/No]
- Feature flag: [Flag name or N/A]
- Backwards compatible: [Yes/No]

### 4. Rollback Plan

**If this change needs to be reverted:**
1. [Step 1: e.g., Revert commit or flip feature flag]
2. [Step 2: e.g., Run migration rollback]
3. [Step 3: e.g., Verify service health]
- Estimated rollback time: [Duration]

### 5. Reviewer Checklist

- [ ] Code follows project style guide and passes lint.
- [ ] Tests cover the changed logic (happy path + failure path).
- [ ] PR description is complete (ticket link, change summary, test evidence, risk, rollback).
- [ ] No hardcoded secrets, credentials, or PII in the code.
- [ ] Database changes are backwards compatible or migration is documented.

---

## T-05: Testing Report Template

### Report Header

```
Project:        [Project Name]
Release:        [Version]
Test Period:    [Start Date] - [End Date]
Author:         [QA Lead Name]
Status:         [In Progress | Complete]
```

### 1. Test Coverage

**Functional Test Cases**
| Module | Total Cases | Passed | Failed | Blocked | Skip | Pass Rate |
|--------|------------|--------|--------|---------|------|-----------|
| [Module A] | [N] | [N] | [N] | [N] | [N] | [%] |
| **Total** | **[N]** | **[N]** | **[N]** | **[N]** | **[N]** | **[%]** |

**Regression Test Cases**
| Suite | Total | Passed | Failed | Pass Rate |
|-------|-------|--------|--------|-----------|
| [Core flows] | [N] | [N] | [N] | [%] |
| **Total** | **[N]** | **[N]** | **[N]** | **[%]** |

**Performance Test Cases**
| Scenario | Metric | Target | Actual | Status |
|----------|--------|--------|--------|--------|
| [API response] | P95 latency | < 200ms | [Xms] | [Pass/Fail] |
| [Page load] | LCP | < 2.5s | [Xs] | [Pass/Fail] |

### 2. Defect Summary

| Severity | Open | Fixed | Verified | Won't Fix | Total |
|----------|------|-------|----------|-----------|-------|
| P0 (Blocker) | [N] | [N] | [N] | [N] | [N] |
| P1 (Critical) | [N] | [N] | [N] | [N] | [N] |
| P2 (Major) | [N] | [N] | [N] | [N] | [N] |
| P3 (Minor) | [N] | [N] | [N] | [N] | [N] |

**Open Blockers** (if any)
| ID | Description | Owner | ETA |
|----|-------------|-------|-----|
| [Bug ID] | [Description] | [Name] | [Date] |

### 3. Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|------------|------------|
| [Identified risk] | [H/M/L] | [H/M/L] | [Mitigation plan] |

### 4. Release Recommendation

- [ ] **GO**: All criteria met, recommend release.
- [ ] **CONDITIONAL GO**: Release with known issues and tracking items listed below.
- [ ] **NO GO**: Blocking issues prevent release.

**Conditions (if conditional)**
| Condition | Owner | Deadline | Tracking |
|-----------|-------|----------|----------|
| [Condition] | [Name] | [Date] | [Ticket link] |

---

## T-06: Security & Compliance Template

### Assessment Header

```
Project:        [Project Name]
Version:        [Version]
Assessor:       [Security Engineer Name]
Date:           [YYYY-MM-DD]
Scope:          [What was assessed]
```

### 1. Risk Surface

**Data Security**
| Data Category | Classification | Protection Measure | Status |
|--------------|---------------|-------------------|--------|
| [User PII] | [Confidential] | [AES-256 at rest, TLS 1.3 in transit] | [Verified] |
| [Payment data] | [Restricted] | [Tokenization + HSM] | [Verified] |

**Permission & Audit**
| Check | Status | Evidence |
|-------|--------|---------|
| Least-privilege enforcement | [Pass/Fail] | [Permission audit report link] |
| Audit log completeness | [Pass/Fail] | [Log sample verification link] |
| Admin action logging | [Pass/Fail] | [Audit trail verification link] |

**Supply Chain Risk**
| Check | Status | Evidence |
|-------|--------|---------|
| Dependency vulnerability scan | [Pass/Fail] | [Trivy/npm audit report link] |
| License compliance | [Pass/Fail] | [License scan report link] |
| Container image scan | [Pass/Fail] | [Image scan report link] |

### 2. Vulnerability Findings

| ID | Severity | Description | CVSS | Status | Remediation | Owner | Deadline |
|----|----------|-------------|------|--------|-------------|-------|----------|
| [V-01] | [Critical] | [Description] | [9.1] | [Fixed] | [Fix details] | [Name] | [Date] |

### 3. Compliance Mapping

| Regulation | Requirement | Technical Control | Evidence | Status |
|-----------|-------------|------------------|---------|--------|
| [GDPR Art.25] | [Data protection by design] | [Encryption, access control] | [Config docs] | [Compliant] |

### 4. Sign-Off

| Role | Name | Decision | Date |
|------|------|----------|------|
| Security Lead | [Name] | [Approve/Reject] | [Date] |
| Compliance Officer | [Name] | [Approve/Reject] | [Date] |

---

## T-07: Release Change Template

### Change Header

```
Change ID:      [CHG-XXXX]
Version:        [Release version]
Release Window: [YYYY-MM-DD HH:MM - HH:MM TZ]
Change Owner:   [Name]
Status:         [Planned | Approved | In Progress | Completed | Rolled Back]
```

### 1. Change Information

**Scope**
- What is being released: [Feature / fix / infrastructure change description]
- Affected systems: [List of systems / services]
- Affected users: [User segments, estimated count]

**Impact Assessment**
| Dimension | Impact | Notes |
|-----------|--------|-------|
| Service availability | [None / Partial / Full downtime] | [Duration if applicable] |
| Data migration | [None / Online / Offline] | [Strategy] |
| API compatibility | [Backward compatible / Breaking] | [Migration guide if breaking] |

### 2. Rollout Strategy

**Rollout Plan**
| Stage | Traffic % | Duration | Criteria to Proceed |
|-------|----------|----------|-------------------|
| Canary | 1% | 1 hour | Error rate < 0.1%, latency stable |
| Early | 10% | 4 hours | Core metrics stable |
| Partial | 50% | 24 hours | No regressions detected |
| Full | 100% | - | All criteria met |

**Rollback Conditions**
- Trigger: [What condition triggers a rollback]
- Procedure: [Step-by-step rollback process]
- Estimated rollback time: [Duration]
- Data reconciliation: [Required / Not required]

### 3. Post-Release Verification

| Check | Method | Expected | Actual | Status |
|-------|--------|----------|--------|--------|
| Core API health | Health endpoint | 200 OK | [Result] | [Pass/Fail] |
| Error rate | Monitoring dashboard | < 0.1% | [Result] | [Pass/Fail] |
| Latency P95 | Monitoring dashboard | < [X]ms | [Result] | [Pass/Fail] |
| Business metric | Analytics | Within 5% of baseline | [Result] | [Pass/Fail] |

**Observation Period**: 48 hours with enhanced monitoring.

---

## T-08: Operations Runbook Template

### Service Header

```
Service:        [Service Name]
Team:           [Owning Team]
On-Call:        [Rotation link / schedule]
Last Updated:   [YYYY-MM-DD]
```

### 1. Service Overview

**Purpose**: [What this service does in 1-2 sentences]

**Key SLOs**
| SLI | SLO Target | Current | Dashboard Link |
|-----|-----------|---------|---------------|
| Availability | [99.95%] | [Current %] | [Link] |
| Latency P99 | [< 500ms] | [Current] | [Link] |
| Error rate | [< 0.1%] | [Current] | [Link] |

**Dependencies**
| Service | Type | Impact if Down | Fallback |
|---------|------|---------------|----------|
| [Database] | [Critical] | [Service unavailable] | [Read replica / cache] |
| [Auth service] | [Critical] | [Cannot authenticate] | [Cached tokens for X min] |
| [Notification] | [Non-critical] | [Notifications delayed] | [Queue and retry] |

### 2. Health Check

**Endpoints**
| Endpoint | Expected Response | Interval |
|----------|------------------|----------|
| `/health` | `200 {"status": "ok"}` | 30s |
| `/ready` | `200 {"ready": true}` | 30s |

**Manual Verification**
```bash
# Quick health check
curl -s https://[service-url]/health | jq .

# Dependency check
curl -s https://[service-url]/health/dependencies | jq .
```

### 3. Alert Response Procedures

**High Error Rate (> 1%)**
1. Check error logs: `[log query or command]`
2. Identify affected endpoints: `[dashboard or query]`
3. Check recent deployments: `[deployment history command]`
4. If caused by recent deploy: rollback using `[rollback command]`
5. If infrastructure issue: check `[infrastructure dashboard]`
6. Escalate to [Team/Person] if unresolved within 15 minutes.

**High Latency (P99 > [threshold])**
1. Check resource utilization: `[dashboard link]`
2. Check database connection pool: `[monitoring link]`
3. Check downstream service latency: `[trace dashboard]`
4. If resource exhaustion: scale up using `[scaling command]`
5. If database bottleneck: check slow query log.
6. Escalate to [Team/Person] if unresolved within 15 minutes.

**Service Unavailable**
1. Check pod / instance status: `[kubectl/command]`
2. Check recent events: `[event log command]`
3. Attempt restart: `[restart command]`
4. If restart fails: check node / host health.
5. Escalate immediately to [Team/Person].

### 4. Scaling Procedures

**Horizontal Scale-Up**
```bash
# Manual scaling
[scaling command, e.g., kubectl scale deployment ...]

# Verify
[verification command]
```

**Auto-Scaling Configuration**
| Metric | Scale-Up Threshold | Scale-Down Threshold | Min / Max Replicas |
|--------|-------------------|---------------------|-------------------|
| CPU | > 70% | < 30% | [2 / 10] |
| Memory | > 80% | < 40% | [2 / 10] |

### 5. Recovery Procedures

**Database Recovery**
1. Identify the failure point: `[diagnostic command]`
2. Check backup status: `[backup check command]`
3. Restore from backup: `[restore command]`
4. Verify data integrity: `[verification command]`
5. Update SLO dashboard with incident impact.

**Restart Procedure**
1. Graceful restart: `[graceful restart command]`
2. Wait for health check to pass: `[health check command]`
3. Verify traffic is flowing: `[traffic verification]`
4. Monitor for 15 minutes post-restart.

---

## T-09: Postmortem Template

### Incident Header

```
Incident ID:    [INC-XXXX]
Severity:       [P0 | P1 | P2]
Date:           [YYYY-MM-DD]
Duration:       [Start time - End time (total duration)]
Author:         [Postmortem author]
Reviewers:      [List of reviewers]
Status:         [Draft | Reviewed | Final]
```

### 1. Executive Summary

[2-3 sentences: what happened, how many users were affected, how long it lasted, and how it was resolved.]

### 2. Impact Assessment

| Dimension | Detail |
|-----------|--------|
| Users affected | [Number and segment] |
| Duration | [Total time from detection to resolution] |
| Revenue impact | [Estimated or N/A] |
| SLA breach | [Yes/No, which SLA] |
| Data impact | [Any data loss or corruption] |

### 3. Timeline

| Time (UTC) | Event |
|-----------|-------|
| [HH:MM] | [First anomaly detected by...] |
| [HH:MM] | [Alert fired / user report received] |
| [HH:MM] | [First responder engaged] |
| [HH:MM] | [Root cause identified] |
| [HH:MM] | [Mitigation applied] |
| [HH:MM] | [Service restored] |
| [HH:MM] | [All-clear confirmed] |

**Detection**
- How was the incident detected? [Alert / User report / Monitoring]
- Time from incident start to detection: [Duration]

**Communication**
- Internal notification sent at: [Time]
- External communication (if applicable) sent at: [Time]
- Communication quality: [Adequate / Needs improvement]

### 4. Root Cause Analysis

**Direct Cause**
[The specific technical failure that caused the incident]

**Contributing Causes**
- [Factor 1: e.g., Missing monitoring for the affected component]
- [Factor 2: e.g., Deployment without sufficient canary period]

**Systemic Cause**
[The organizational or process gap that allowed the contributing causes to exist]

**5 Whys Analysis**
1. Why did [symptom]? Because [direct cause].
2. Why did [direct cause]? Because [contributing cause 1].
3. Why did [contributing cause 1]? Because [contributing cause 2].
4. Why did [contributing cause 2]? Because [systemic cause].
5. Why did [systemic cause]? Because [root organizational gap].

### 5. Action Items

**Immediate Fixes (1-3 days)**
| Action | Owner | Deadline | Status | Ticket |
|--------|-------|----------|--------|--------|
| [Fix description] | [Name] | [Date] | [Open/Done] | [Link] |

**Short-Term Prevention (1-2 weeks)**
| Action | Owner | Deadline | Status | Ticket |
|--------|-------|----------|--------|--------|
| [Prevention action] | [Name] | [Date] | [Open/Done] | [Link] |

**Long-Term Prevention (1-3 months)**
| Action | Owner | Deadline | Status | Ticket |
|--------|-------|----------|--------|--------|
| [Structural improvement] | [Name] | [Date] | [Open/Done] | [Link] |

### 6. Lessons Learned

**What Went Well**
- [e.g., Alert fired within 2 minutes of anomaly]
- [e.g., Rollback procedure executed smoothly]

**What Went Poorly**
- [e.g., Root cause identification took 45 minutes due to missing traces]
- [e.g., External communication was delayed]

**Knowledge Base Updates**
- [ ] New checklist item added to [checklist name].
- [ ] New anti-pattern documented in [knowledge area].
- [ ] Runbook updated for [service name].

---

## T-10: Lifecycle Review Board Template

### Review Header

```
Project:        [Project Name]
Review Date:    [YYYY-MM-DD]
Reviewer:       [Review Board Chair]
Participants:   [List of participants]
```

### 1. Stage Status Dashboard

| Stage | Status | Gate Owner | Gate Pass Date | Blockers |
|-------|--------|-----------|----------------|----------|
| Requirement | [Not Started / In Progress / Passed / Blocked] | [Name] | [Date or -] | [Description or None] |
| Design | [Not Started / In Progress / Passed / Blocked] | [Name] | [Date or -] | [Description or None] |
| Architecture | [Not Started / In Progress / Passed / Blocked] | [Name] | [Date or -] | [Description or None] |
| Implementation | [Not Started / In Progress / Passed / Blocked] | [Name] | [Date or -] | [Description or None] |
| Testing | [Not Started / In Progress / Passed / Blocked] | [Name] | [Date or -] | [Description or None] |
| Security | [Not Started / In Progress / Passed / Blocked] | [Name] | [Date or -] | [Description or None] |
| Release | [Not Started / In Progress / Passed / Blocked] | [Name] | [Date or -] | [Description or None] |
| Operations | [Not Started / In Progress / Passed / Blocked] | [Name] | [Date or -] | [Description or None] |
| Postmortem | [Not Started / In Progress / Passed / Blocked] | [Name] | [Date or -] | [Description or None] |

### 2. Blockers & Decisions

**Active Blockers**
| ID | Blocker | Affected Stage | Owner | Deadline | Escalation |
|----|---------|---------------|-------|----------|-----------|
| B-01 | [Description] | [Stage] | [Name] | [Date] | [Escalation action if deadline missed] |

**Decisions Required**
| ID | Decision | Context | Options | Recommended | Decider | Deadline |
|----|----------|---------|---------|-------------|---------|----------|
| D-01 | [Decision needed] | [Why] | [A, B, C] | [Recommendation] | [Name] | [Date] |

### 3. Risk Register Update

| ID | Risk | Stage | Status | Mitigation Progress |
|----|------|-------|--------|-------------------|
| R-01 | [Risk from requirement stage] | [Stage] | [Active/Mitigated/Closed] | [Progress notes] |

### 4. Next Review

- Next review date: [YYYY-MM-DD]
- Focus areas: [Stages or topics to review]
- Required participants: [Names]

---

## Agent Checklist

- [ ] Select the appropriate template for the current lifecycle stage.
- [ ] Fill in all sections -- do not leave placeholder text in production documents.
- [ ] Ensure every action item has an owner and deadline.
- [ ] Link documents to upstream artifacts (requirement links to PRD, PR links to requirement, etc.).
- [ ] Archive completed templates as project artifacts.
- [ ] Review template catalog when starting a new lifecycle stage to ensure no template is missed.
- [ ] Update the lifecycle review board template at each stage transition.
- [ ] Use the postmortem template within 5 business days of any P0/P1 incident.
