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

## ADDED Requirements

### Requirement: The evaluation context and client snapshot carry plan and memberships

The server-side evaluation context SHALL include the caller's effective plan and the set of
active beta campaign keys next to app scope and staff status, resolved once per request from the
plan entitlements. The client's cached snapshot SHALL be keyed by identity, plan **and**
memberships, so a purchase, an enrolment, a lapse or a beta closing invalidates the snapshot and
the next fetch reflects the new state.

#### Scenario: Purchase refreshes plan-scoped flags

- **WHEN** a free user becomes premium and the app refreshes its flags
- **THEN** premium-only flags are now in effect on the device

#### Scenario: Enrolment refreshes beta-scoped flags

- **WHEN** a user enrols in the `midi-drums` beta and the app refreshes its flags
- **THEN** `beta:midi-drums` flags are now in effect on the device

#### Scenario: Lapse withdraws plan-scoped flags

- **WHEN** a premium user lapses to free
- **THEN** the next snapshot no longer contains premium-only flags in effect
