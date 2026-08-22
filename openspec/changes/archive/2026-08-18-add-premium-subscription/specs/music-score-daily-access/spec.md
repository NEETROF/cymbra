## MODIFIED Requirements

### Requirement: Subscription bypass seam and upsell hook

The open gate SHALL consult a subscription seam (`has_active_subscription(caller)`).
A subscriber SHALL have **unlimited** opens (the daily quota does not apply). The seam
SHALL be implemented over the plan entitlements: `has_active_subscription(caller)` is
true exactly when the caller's **effective plan grants the `catalog.unlimited` unlock**
(i.e. the effective plan is `premium`, whatever its source — store, web, trial or admin).
The quota logic MUST NOT know plan names or beta campaigns — it consumes the seam only.
When the quota is reached / an unlock is offered, the access state SHALL carry an
**upsell** signal that the client renders as a real, platform-appropriate call to action
to the paywall. While `plans.enabled` is off the seam SHALL answer false, exactly as
before.

#### Scenario: A subscriber bypasses the quota
- **WHEN** the caller's effective plan grants `catalog.unlimited`
- **THEN** any piece is served regardless of the daily quota or points balance

#### Scenario: A premium-trial tester bypasses the quota like a subscriber
- **WHEN** the caller holds an active premium trial row
- **THEN** any piece is served regardless of the daily quota

#### Scenario: A feature-beta member on the free plan stays under quota
- **WHEN** the caller is a member of a feature beta but has no premium row
- **THEN** the quota applies normally

#### Scenario: Seam is inert while plans are disabled
- **WHEN** `plans.enabled` is off
- **THEN** `has_active_subscription` returns false and the quota applies normally

#### Scenario: Quota-reached response carries the upsell signal
- **WHEN** a non-subscriber reaches the quota and is offered the points unlock
- **THEN** the response includes an upsell signal the client renders as a call to action to the paywall for that platform
