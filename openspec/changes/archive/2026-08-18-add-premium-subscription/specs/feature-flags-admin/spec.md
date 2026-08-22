## ADDED Requirements

### Requirement: Plan- and beta-scoped rollouts are selectable in the flags console

The flags panel SHALL let an admin set a flag's rollout scope to `global`, `staff_only`,
`premium_only`, or `beta:<campaign>` chosen from the **open beta campaigns** (not typed
free-hand), and SHALL display plan- and beta-scoped rollouts distinctly so an operator can tell
at a glance which features are gated by plan or by beta. The new keys introduced by the plan
system (`plans.enabled`, `plans.grace_days`, `plans.premium.products`,
`plans.soundfont_library.max_fonts.*`, `plans.scores.*`, `billing.*.enabled`) SHALL appear in
the panel with self-explanatory descriptions; `plans.enabled` and the `billing.*.enabled` keys
SHALL be treated as sensitive keys.

#### Scenario: Admin scopes a flag to a beta campaign

- **WHEN** an admin selects `beta:midi-drums` for a flag from the campaign list and saves
- **THEN** the flag's rollout is `beta:midi-drums`, the change is audited, and the panel marks it as beta-scoped

#### Scenario: Only existing campaigns are offered

- **WHEN** an admin opens the rollout selector
- **THEN** the beta options are exactly the campaigns that are not closed

#### Scenario: Billing switches are sensitive

- **WHEN** an admin toggles `billing.apple.enabled`
- **THEN** the sensitive-key protection applies before the change is made
