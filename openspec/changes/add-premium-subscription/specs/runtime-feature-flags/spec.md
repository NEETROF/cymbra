## MODIFIED Requirements

### Requirement: Rollout scope — global or staff-only

Each flag SHALL have a scope of **global**, **staff-only**, **beta-only** or **premium-only**. A
**staff-only** flag SHALL be in effect only for admin/moderator identities, so a feature can be
exercised by staff before a global rollout; a **beta-only** flag SHALL be in effect for identities
whose effective plan is `beta` or `premium`, and for staff; a **premium-only** flag SHALL be in
effect for identities whose effective plan is `premium`, and for staff; a **global** flag applies
to everyone. Scope SHALL be resolved server-side per the caller's identity and effective plan.
While `plans.enabled` is off, every identity's plan evaluates as `free`, so plan-scoped flags
reach staff only.

#### Scenario: Staff-only feature is limited to staff

- **WHEN** a feature flag is staff-only and enabled
- **THEN** it is in effect for admin/moderator identities and not for normal users

#### Scenario: Beta-only feature reaches beta, premium and staff

- **WHEN** a feature flag is beta-only and enabled
- **THEN** it is in effect for beta and premium identities and for staff, and not for free users

#### Scenario: Premium-only feature reaches premium and staff

- **WHEN** a feature flag is premium-only and enabled
- **THEN** it is in effect for premium identities and for staff, and not for free or beta users

#### Scenario: Global feature applies to everyone

- **WHEN** a feature flag is global and enabled
- **THEN** it is in effect for all users

## ADDED Requirements

### Requirement: The evaluation context and client snapshot carry the effective plan

The server-side evaluation context SHALL include the caller's effective plan next to app scope
and staff status, resolved once per request from the plan entitlements. The client's cached
snapshot SHALL be keyed by identity **and** plan, so a purchase, a redemption or a lapse
invalidates the snapshot and the next fetch reflects the new plan.

#### Scenario: Purchase refreshes plan-scoped flags

- **WHEN** a free user becomes premium and the app refreshes its flags
- **THEN** premium-only flags are now in effect on the device

#### Scenario: Lapse withdraws plan-scoped flags

- **WHEN** a premium user lapses to free
- **THEN** the next snapshot no longer contains premium-only flags in effect
