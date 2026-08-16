## ADDED Requirements

### Requirement: Plan-scoped rollouts are selectable in the flags console

The flags panel SHALL let an admin set a flag's rollout scope to `global`, `staff_only`,
`beta_only` or `premium_only`, and SHALL display the plan-scoped rollouts distinctly so an
operator can tell at a glance which features are gated by plan. The new keys introduced by the
plan system (`plans.enabled`, `plans.beta.features`, `plans.grace_days`,
`plans.soundfont_library.max_fonts.*`, `billing.*.enabled`) SHALL appear in the panel with
self-explanatory descriptions; `plans.enabled` and the `billing.*.enabled` keys SHALL be treated
as sensitive keys.

#### Scenario: Admin sets a beta-only rollout

- **WHEN** an admin selects `beta_only` for a flag and saves
- **THEN** the flag's rollout is `beta_only`, the change is audited, and the panel marks it as plan-scoped

#### Scenario: Billing switches are sensitive

- **WHEN** an admin toggles `billing.apple.enabled`
- **THEN** the sensitive-key protection applies before the change is made
