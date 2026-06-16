---
id: ecommerce
title: E-Commerce Product — Industry-Specific Knowledge
domain: experts
category: product-manager
difficulty: intermediate
tags: [architecture, considerations, e-commerce, e-commerce-specific, ecommerce, experts, metrics, patterns]
quality_score: 70
last_updated: 2026-06-15
---
# E-Commerce Product — Industry-Specific Knowledge

## Key Metrics
- **Conversion rate** — visitors → purchases, industry avg 2-3%
- **AOV** (Average Order Value) — optimize with upsells, bundles, free shipping threshold
- **Cart abandonment rate** — industry avg 70%, target < 55%
- **Repeat purchase rate** — % of customers who buy again within 90 days
- **ROAS** (Return on Ad Spend) — for paid acquisition channels

## E-Commerce-Specific PRD Considerations
- **Product catalog** — SKU management, variants (size/color), inventory tracking
- **Cart** — persistent across sessions, merge anonymous → logged-in
- **Checkout** — 1-page preferred, guest checkout required (don't force registration)
- **Payment** — Stripe/PayPal minimum, support Apple Pay/Google Pay for mobile
- **Shipping** — real-time rate calculation, multiple carriers, free shipping threshold
- **Tax** — per-jurisdiction calculation (use Stripe Tax / TaxJar)
- **Returns/refunds** — self-serve return requests, automated refund processing
- **Order tracking** — real-time status updates, email/SMS notifications

## E-Commerce Architecture Patterns
- **Cart service** — separate from order service, handles anonymous + authenticated
- **Inventory management** — optimistic locking to prevent overselling
- **Search** — faceted search with filters (price range, color, size, rating)
- **Image CDN** — responsive images, WebP/AVIF, lazy loading, zoom capability
- **Recommendation engine** — "customers also bought", "frequently bought together"

## E-Commerce UX Requirements
- **Product page** — hero image gallery, variant selector, price + savings, trust signals (reviews, return policy)
- **Add to cart** — no page navigation, drawer/modal confirmation, "continue shopping" option
- **Cart** — edit quantity, remove items, promo code input, shipping estimate
- **Checkout** — shipping → payment → review → confirm, progress indicator, save address for next time
- **Order confirmation** — order number, expected delivery, items summary, email confirmation
- **Mobile** — bottom sticky "Add to Cart" button, swipeable image gallery, one-tap payment
