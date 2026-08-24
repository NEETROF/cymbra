## MODIFIED Requirements

### Requirement: Plan- and beta-scoped rollouts are selectable in the flags console

The flags panel SHALL let an admin set a flag's rollout scope to `global`, `staff_only`,
`premium_only`, or `beta:<campaign>` chosen from the **open beta campaigns** (not typed
free-hand), and SHALL display plan- and beta-scoped rollouts distinctly so an operator can tell
at a glance which features are gated by plan or by beta. The new keys introduced by the plan
system (`plans.enabled`, `plans.grace_days`, `plans.premium.products`,
`plans.soundfont_library.max_fonts.*`, `plans.scores.*`, `billing.*.enabled`) SHALL appear in
the panel with self-explanatory descriptions; `plans.enabled` and the `billing.*.enabled` keys
SHALL be treated as sensitive keys.

A stored scope that is not among the offered options SHALL remain selectable, so the panel never
silently rewrites an operator's stored value. Where such a scope names **no existing campaign**,
the panel SHALL present it **distinctly from a valid one**, naming the cause, rather than as an
ordinary option — an unrecognised-but-valid scope (a closed campaign, still legitimately gating)
and a dangling one (a campaign that never existed) are otherwise indistinguishable, and only the
second is a defect.

When a write is refused, the panel SHALL surface a localised explanation of the cause, never a
raw transport status — and the two refusal causes SHALL read differently: a scope naming no
campaign calls for **fixing the scope**, while a check that could not be performed (the campaign
directory unreachable) calls for **retrying later**. Collapsing them would be worse than
unhelpful: an outage message claiming the campaign does not exist invites exactly the
destructive scope edit this validation exists to prevent.

#### Scenario: Admin scopes a flag to a beta campaign

- **WHEN** an admin selects `beta:midi-drums` for a flag from the campaign list and saves
- **THEN** the flag's rollout is `beta:midi-drums`, the change is audited, and the panel marks it as beta-scoped

#### Scenario: Only existing campaigns are offered

- **WHEN** an admin opens the rollout selector
- **THEN** the beta options are exactly the campaigns that are not closed

#### Scenario: Billing switches are sensitive

- **WHEN** an admin toggles `billing.apple.enabled`
- **THEN** the sensitive-key protection applies before the change is made

#### Scenario: A closed campaign's stored scope stays selectable and unremarkable

- **WHEN** a flag's stored scope names a campaign that exists but is closed
- **THEN** it remains selectable and is not presented as a defect, because it is still
  legitimately gating

#### Scenario: A dangling scope is presented as such

- **WHEN** a flag's stored scope names no existing campaign
- **THEN** the panel presents it distinctly from a valid scope and names the cause

#### Scenario: A refused write explains itself

- **WHEN** a save is rejected because its scope names no campaign
- **THEN** the panel shows a localised explanation rather than a raw transport status

#### Scenario: An outage refusal says retry, not fix

- **WHEN** a save is refused because campaign existence could not be verified
- **THEN** the panel says the check is temporarily unavailable and to retry — it never claims
  the campaign does not exist
