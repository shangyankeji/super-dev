---
id: saas
title: SaaS Product — Industry-Specific Knowledge
domain: experts
category: product-manager
difficulty: intermediate
tags: [appear, architecture, common, considerations, experts, metrics, must, patterns]
quality_score: 70
last_updated: 2026-06-15
---
# SaaS Product — Industry-Specific Knowledge

## Key Metrics (must appear in PRD success metrics)
- **MRR** (Monthly Recurring Revenue) — the north star metric
- **Churn rate** — monthly: acceptable < 5%, good < 3%
- **CAC** (Customer Acquisition Cost) / **LTV** (Lifetime Value) — LTV/CAC > 3 is healthy
- **Activation rate** — % of signups who complete onboarding and reach "aha moment"
- **NPS** (Net Promoter Score) — measure after 30 days

## SaaS-Specific PRD Considerations
- **Multi-tenancy** — data isolation between customers is non-negotiable
- **Billing integration** — Stripe/Paddle, plan limits, usage metering, proration
- **Onboarding flow** — first-time user experience determines activation rate
- **Team management** — invite members, roles (admin/member/viewer), SSO
- **Self-serve vs sales-led** — affects pricing page, trial flow, upgrade prompts

## Common SaaS Architecture Patterns
- **Database per tenant** (expensive, max isolation) vs **shared DB with RLS** (efficient, careful isolation)
- **Feature flags** — launch to % of users, A/B test features
- **Webhook system** — customers need event notifications for integrations
- **API rate limiting** — per-plan rate limits (free: 100/min, pro: 1000/min)
- **Audit log** — enterprise customers require activity logging

## SaaS Pricing Page Requirements
- 2-4 tiers (free/starter/pro/enterprise)
- Annual vs monthly toggle with savings badge
- Feature comparison table below tier cards
- "Most popular" highlight on recommended tier
- Enterprise: "Contact sales" instead of price
- FAQ section addressing billing questions
