---
profile: feature
id: docs/guide/07-examples/ecommerce/website/cart-checkout
status: stable
description: Checkout checkout flow supporting one-time orders and recurring subscription signups.
depends:
  - subscription-service.md
related:
  - ../support/refund-guide.md
resources:
  - path: ../resources/user-flow-checkout.pdf
load:
  - subscription-service.md
ignore:
  - ../marketing
---

# Website Cart and Checkout Flow

## Goal

Provide a frictionless, responsive checkout process for one-time purchases and recurring subscription signups.

## Scope

- Responsive shopping cart page.
- Credit card and Apple Pay processing.
- Seamless email confirmation.

## Requirements

- Render all cart items with current pricing.
- Auto-calculate tax and shipping on zip entry.
- Integrate Stripe subscription billing if a recurring product is present.

## Acceptance Criteria

- Successful order redirects to confirmation page with HTTP 200.
- Failed payment triggers user-facing card error details.
- See the full wireframe in [user-flow-checkout.pdf](../resources/user-flow-checkout.pdf).

## Risks

- Checkout latency causing cart abandonment — mitigated by server-side caching of tax rates.
