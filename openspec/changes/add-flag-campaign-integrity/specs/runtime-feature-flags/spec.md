## MODIFIED Requirements

### Requirement: Rollout scope — global or staff-only

Each flag SHALL have a rollout scope of **global**, **staff-only**, **premium-only** or
**beta:`<campaign>`**. A **staff-only** flag SHALL be in effect only for admin/moderator
identities, so a feature can be exercised by staff before a global rollout; a **premium-only**
flag SHALL be in effect for identities whose effective plan is `premium`, and for staff; a
**beta:`<campaign>`** flag SHALL be in effect for identities holding an **active membership** of
that beta campaign, and for staff — and NOT for premium payers outside the campaign; a **global**
flag applies to everyone. Scope SHALL be resolved server-side per the caller's identity, effective
plan and active memberships. While `plans.enabled` is off, every identity evaluates as `free` with
no memberships, so plan- and beta-scoped flags reach staff only. Closing a beta campaign SHALL make
its flags stop matching members without any flag edit.

A **beta:`<campaign>`** scope SHALL name a campaign that **exists**, and the system SHALL verify
this when the override is written, rejecting the write with a typed error otherwise. Shape
validation alone is insufficient: a well-formed key naming no campaign stores cleanly, is
audited as a legitimate change, and then matches nobody indefinitely — a failure invisible to
the operator who made it, because staff match every beta scope regardless of whether the key is
correct.

The verification SHALL test **existence, not openness**. A closed campaign's scope stays valid,
because closing a campaign is how a feature is withdrawn from its members *without* a flag edit;
invalidating the stored scope would break that and force an edit precisely when none is wanted.

The verification SHALL fail closed: when campaign existence cannot be determined, the write
SHALL be refused rather than stored unverified. An override write is a rare, retryable
operator action, so refusing it costs little, while storing an unverifiable scope reintroduces
exactly the silent failure this requirement exists to prevent.

#### Scenario: Staff-only feature is limited to staff

- **WHEN** a feature flag is staff-only and enabled
- **THEN** it is in effect for admin/moderator identities and not for normal users

#### Scenario: Premium-only feature reaches premium and staff

- **WHEN** a feature flag is premium-only and enabled
- **THEN** it is in effect for premium identities and for staff, and not for free users

#### Scenario: Beta-scoped feature reaches that campaign's members

- **WHEN** a feature flag is `beta:midi-drums` and enabled
- **THEN** it is in effect for active members of the `midi-drums` campaign and for staff, whatever their plan, and not for anyone else — including premium subscribers outside the campaign

#### Scenario: Closing the campaign withdraws the feature

- **WHEN** the `midi-drums` campaign is closed
- **THEN** the `beta:midi-drums` flag no longer matches its former members at the next evaluation

#### Scenario: Global feature applies to everyone

- **WHEN** a feature flag is global and enabled
- **THEN** it is in effect for all users

#### Scenario: A scope naming no campaign is refused

- **WHEN** any caller writes a rollout scope of `beta:<key>` where no campaign has that key
- **THEN** the write is rejected with a typed error and no override is stored

#### Scenario: A closed campaign's scope is still accepted

- **WHEN** an override is written naming a campaign that exists but is closed
- **THEN** the write succeeds, because closing withdraws access without requiring a flag edit

#### Scenario: An unverifiable check refuses the write

- **WHEN** campaign existence cannot be determined at write time
- **THEN** the write is refused rather than stored unverified

#### Scenario: Evaluation is unchanged

- **WHEN** a stored beta-scoped override is evaluated
- **THEN** the audience it grants is exactly what it was before this verification existed
