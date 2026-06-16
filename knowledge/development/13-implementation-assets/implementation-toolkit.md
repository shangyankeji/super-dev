---
id: implementation-toolkit
title: Implementation Toolkit - Comprehensive Asset Reference
domain: development
category: 13-implementation-assets
difficulty: intermediate
tags: [coverage, decision, development, execution, gates, implementation, knowledge, matrix]
quality_score: 70
last_updated: 2026-06-15
---
# Implementation Toolkit - Comprehensive Asset Reference

> Consolidated reference covering knowledge gates execution, scenario coverage matrix, scene decision tree, UI KPIs & quality gates, and the 180-day knowledge base roadmap.

---

## 1. Knowledge Gates Execution

### 1.1 Purpose

Knowledge gates connect development domain rules to the release pipeline, ensuring that knowledge standards are enforced at build time rather than discovered post-release.

### 1.2 Local Execution Commands

| Task | Command |
|------|---------|
| Run development domain audit | `python3 scripts/audit_development_kb.py` |
| Run knowledge gates | `python3 scripts/check_knowledge_gates.py --project-dir .` |
| Generate report (JSON) | `python3 scripts/check_knowledge_gates.py --project-dir . --format json --out artifacts/knowledge-gates.json` |
| Generate report (Markdown) | `python3 scripts/check_knowledge_gates.py --project-dir . --format md --out artifacts/knowledge-gates.md` |
| Generate report (HTML) | `python3 scripts/check_knowledge_gates.py --project-dir . --format html --out artifacts/knowledge-gates.html` |
| Generate report (JUnit) | `python3 scripts/check_knowledge_gates.py --project-dir . --format junit --out artifacts/knowledge-gates.xml` |
| Generate lifecycle packet | `python3 scripts/generate_lifecycle_packet.py --project-dir . --name <packet-name>` |

### 1.3 CI Pipeline Integration

Knowledge gates must be integrated into the CI pipeline as a mandatory stage:

```
Build -> Lint -> Test -> Knowledge Gates -> Security Scan -> Deploy
```

- In Jenkins: add a "Knowledge Gates" stage after the test stage.
- Any gate failure must block subsequent build and deployment stages.
- Gate results should be archived as build artifacts for traceability.

### 1.4 Failure Diagnosis & Resolution

| Failure Type | Diagnosis | Resolution |
|-------------|-----------|------------|
| Missing entry | Required knowledge file not found in expected directory | Create the file in the correct directory and update the catalog index |
| Rule gap | UI gate or scenario matrix missing required items | Update the corresponding gate YAML or matrix file |
| Lifecycle gap | Stage-exit-criteria missing a required phase | Add the missing phase definition to `stage-exit-criteria.yaml` |
| Template gap | Lifecycle template directory missing required templates | Create the template file and register it in `template-catalog.yaml` |
| Missing header | Document lacks the required first-line metadata | Add the standard header line to the document |
| Stale content | Document has not been updated within the freshness threshold | Prioritize refreshing high-risk entries; update content and timestamp |
| Topic duplication | Multiple files covering the same topic | Merge duplicates into a single source of truth; redirect or remove the duplicate |

### 1.5 Pass Criteria

All of the following must be satisfied for the knowledge gate to pass:

- [ ] Development domain audit reaches "knowledge base" maturity level.
- [ ] Scenario coverage matrix includes all 8 scenario types (B2B, B2C, Multi-tenant SaaS, Internationalization, Mobile, AI Application, Fintech, E-commerce Peak).
- [ ] UI quality gates include all 8 core metric categories.
- [ ] Full lifecycle stages cover the complete chain from requirements through post-incident review.
- [ ] Full lifecycle templates cover deliverables for every stage.
- [ ] No critical gate failures; composite score meets the configured threshold.

---

## 2. Scenario Coverage Matrix

### 2.1 Required Assets Per Scenario

Every scenario must have all five asset types present and non-empty:

| Scenario | Standard | Playbook | Checklist | Anti-Pattern | Case Study |
|----------|----------|----------|-----------|-------------|------------|
| B2B Enterprise | Required | Required | Required | Required | Required |
| B2C Growth | Required | Required | Required | Required | Required |
| Multi-Tenant SaaS | Required | Required | Required | Required | Required |
| Internationalization | Required | Required | Required | Required | Required |
| Mobile Superapp | Required | Required | Required | Required | Required |
| AI Application | Required | Required | Required | Required | Required |
| Fintech Regulated | Required | Required | Required | Required | Required |
| E-commerce Peak | Required | Required | Required | Required | Required |

### 2.2 Coverage Audit Procedure

1. For each scenario in the matrix, verify that all 5 asset files exist.
2. For each asset file, verify that it contains substantive content (not just headers).
3. For each asset file, verify that the content was reviewed within the freshness period (default: 90 days).
4. Report coverage percentage: `(populated_assets / total_required_assets) * 100`.
5. Target: 100% coverage. Any gap must have a tracked remediation ticket.

### 2.3 Matrix Configuration (YAML)

```yaml
coverage_matrix:
  b2b:
    required_assets: [standard, playbook, checklist, antipattern, case]
  b2c:
    required_assets: [standard, playbook, checklist, antipattern, case]
  multitenant_saas:
    required_assets: [standard, playbook, checklist, antipattern, case]
  internationalization:
    required_assets: [standard, playbook, checklist, antipattern, case]
  mobile:
    required_assets: [standard, playbook, checklist, antipattern, case]
  ai_application:
    required_assets: [standard, playbook, checklist, antipattern, case]
  fintech:
    required_assets: [standard, playbook, checklist, antipattern, case]
  ecommerce_peak:
    required_assets: [standard, playbook, checklist, antipattern, case]
```

---

## 3. Scene Decision Tree

### 3.1 Entry Questions

The decision tree routes a project to the correct scenario packs based on five key questions:

```
Q1: Is the product enterprise-facing or consumer-facing?
    -> Enterprise: B2B scenario pack
    -> Consumer: B2C growth scenario pack

Q2: Does the product require multi-tenant isolation?
    -> Yes: Stack Multi-Tenant SaaS scenario pack

Q3: Does the product serve multiple regions or languages?
    -> Yes: Stack Internationalization scenario pack

Q4: Is the primary user experience a mobile application?
    -> Yes: Stack Mobile Superapp scenario pack

Q5: Does the product include AI generation or intelligent decision-making?
    -> Yes: Stack AI Application scenario pack

Q6: Is the product subject to financial regulations?
    -> Yes: Stack Fintech Regulated scenario pack

Q7: Does the product face peak-traffic promotional events?
    -> Yes: Stack E-commerce Peak scenario pack
```

### 3.2 Decision Output Rules

- Enterprise service -> prioritize B2B scenario pack.
- Consumer growth -> prioritize B2C growth scenario pack.
- Tenant isolation required -> stack Multi-Tenant SaaS scenario pack.
- Cross-border requirements -> stack Internationalization scenario pack.
- Mobile core journey -> stack Mobile Superapp scenario pack.
- Intelligent capabilities -> stack AI Application scenario pack.
- Financial regulation -> stack Fintech Regulated scenario pack.
- Peak traffic events -> stack E-commerce Peak scenario pack.

Multiple packs are additive. Apply all that match.

### 3.3 Decision Documentation

When selecting scenarios for a project, document:
1. Each question answered and the reasoning.
2. The selected scenario packs.
3. Any packs considered but excluded, with justification.
4. Review date for re-evaluation (recommended: quarterly).

---

## 4. UI KPIs & Quality Gates

### 4.1 Quality Gate Thresholds

These gates must pass before any UI change can ship:

| Gate | Threshold | Measurement |
|------|-----------|-------------|
| Token Coverage | >= 95% | Percentage of component styles using tokens vs. hardcoded values |
| Component State Coverage | 100% | All required states implemented per component state matrix |
| WCAG Compliance | 2.2 AA | axe-core or equivalent automated scan |
| Lighthouse Accessibility | >= 95 | Lighthouse audit score |
| Lighthouse Performance | >= 85 | Lighthouse audit score |
| Visual Regression Threshold | <= 0.3% | Pixel diff percentage in snapshot comparison |
| Critical Flow Pass Rate | 100% | All critical user flows pass E2E test |
| Blocking Bugs | 0 | No P0/P1 bugs open at release time |

### 4.2 KPI Targets

These KPIs measure the ongoing effectiveness of the UI:

| KPI | Target | Measurement Frequency |
|-----|--------|----------------------|
| Task Success Rate | >= 90% | Weekly analytics review |
| Interaction Error Rate | <= 2% | Weekly analytics review |
| UI Consistency Score | >= 90 | Per-release design review |
| Design Review Reopen Rate | <= 10% | Per-sprint retrospective |

### 4.3 KPI Configuration (YAML)

```yaml
ui_quality_gates:
  token_coverage: ">=95%"
  component_state_coverage: "100%"
  accessibility_wcag: "2.2-AA"
  lighthouse_accessibility: ">=95"
  lighthouse_performance: ">=85"
  visual_regression_threshold: "<=0.3%"
  critical_flow_pass_rate: "100%"
  blocking_bugs: 0

ui_kpi:
  task_success_rate: ">=90%"
  interaction_error_rate: "<=2%"
  ui_consistency_score: ">=90"
  design_review_reopen_rate: "<=10%"
```

---

## 5. Knowledge Base Roadmap (180 Days)

### Phase 1: Foundation (Days 0-30)

**Objective**: Establish minimum viable knowledge base with governance, standards, and scenario pack closure.

| Week | Deliverable | Success Criteria |
|------|-------------|-----------------|
| 1-2 | Complete governance rules, directory structure, catalog index | Audit script runs without structural errors |
| 2-3 | Complete core development standards (top 5 by usage frequency) | Standards referenced in at least 1 project |
| 3-4 | Complete scenario pack minimum closure (5 assets x 8 scenarios) | Scenario coverage matrix reports 100% |

Quality baseline established:
- Knowledge audit passes at "foundation" maturity level.
- Search hit rate baselined for future comparison.

### Phase 2: Depth (Days 31-90)

**Objective**: Complete UI excellence system and business scenario case studies.

| Week | Deliverable | Success Criteria |
|------|-------------|-----------------|
| 5-6 | UI excellence system fully deployed (all 10 knowledge areas) | UI gate passes for new projects |
| 7-8 | 8 business scenario case studies completed with real data | Each case study has measurable outcome |
| 9-10 | First quarterly audit completed | Audit report generated; improvement items tracked |
| 11-12 | Quarterly remediation cycle established | Remediation backlog < 10 items |

Quality targets:
- Knowledge audit reaches "comprehensive" maturity level.
- Quarterly audit cadence established with remediation tracking.

### Phase 3: Maturity (Days 91-180)

**Objective**: Automate knowledge gates, enable cross-team reuse, and reach maturity level 4.

| Week | Deliverable | Success Criteria |
|------|-------------|-----------------|
| 13-16 | Knowledge gates integrated into CI pipeline for all projects | Gate failures block deployment |
| 17-20 | Cross-team knowledge reuse mechanism established | At least 3 teams consuming shared knowledge base |
| 21-24 | Contribution incentive program launched | At least 10 external contributions received |
| 25-26 | Maturity L4 achieved; L5 (AI-assisted) roadmap drafted | L4 assessment passed; L5 plan documented |

Maturity targets:
- Knowledge gates automated and enforced in CI.
- Cross-team reuse mechanism operational.
- Maturity Level 4 achieved.
- Level 5 (intelligent knowledge management) initiative planned.

---

## Agent Checklist

- [ ] Run knowledge gates locally before submitting PR (`python3 scripts/check_knowledge_gates.py --project-dir .`).
- [ ] Verify scenario coverage matrix shows 100% for all applicable scenarios.
- [ ] Walk through scene decision tree when starting a new project or feature.
- [ ] Confirm UI quality gates meet thresholds before merging UI changes.
- [ ] Check knowledge base roadmap phase alignment for current project timeline.
- [ ] Archive gate results as build artifacts for audit trail.
- [ ] Address any gate failures before proceeding with deployment.
