## MODIFIED Requirements

### Requirement: The evaluation context and client snapshot carry plan and memberships

The server-side evaluation context SHALL include the caller's effective plan and the set of
active beta campaign keys next to app scope and staff status, resolved once per request from the
plan entitlements **of the app being evaluated**: a caller evaluated for Lingua carries the
memberships of Lingua campaigns, a caller evaluated for Music those of Music campaigns, and a
caller whose app has no product (the web site, the back office) carries none. The client's cached
snapshot SHALL be keyed by identity, plan **and** memberships, so a purchase, an enrolment, a
lapse or a beta closing invalidates the snapshot and the next fetch reflects the new state.

#### Scenario: Purchase refreshes plan-scoped flags

- **WHEN** a free user becomes premium and the app refreshes its flags
- **THEN** premium-only flags are now in effect on the device

#### Scenario: Enrolment refreshes beta-scoped flags

- **WHEN** a user enrols in the `midi-drums` beta and the app refreshes its flags
- **THEN** `beta:midi-drums` flags are now in effect on the device

#### Scenario: Lapse withdraws plan-scoped flags

- **WHEN** a premium user lapses to free
- **THEN** the next snapshot no longer contains premium-only flags in effect

#### Scenario: A Lingua member reading Lingua flags

- **WHEN** an account enrolled in the Lingua campaign `online-translation` fetches its flags with a Lingua token
- **THEN** flags scoped `beta:online-translation` are in effect for it

#### Scenario: A Music membership does not reach Lingua

- **WHEN** an account enrolled only in the Music campaign `midi-drums` fetches its flags with a Lingua token
- **THEN** its Lingua evaluation carries no membership

## ADDED Requirements

### Requirement: A beta scope names a campaign of the flag's app
A rollout scope `beta:<key>` SHALL be accepted on a flag only when `<key>` names a campaign of the flag's app; a write naming a campaign of another product SHALL be refused like a write naming no campaign.
A flag declared for every app SHALL accept a campaign of any product. A stored scope written before
this rule SHALL keep evaluating as before.

#### Scenario: A Lingua flag scoped to a Lingua campaign
- **WHEN** an admin sets `lingua.online_translation.enabled` to `beta:online-translation`, a Lingua campaign
- **THEN** the write is accepted and audited

#### Scenario: A Lingua flag scoped to a Music campaign
- **WHEN** an admin sets a Lingua flag to `beta:midi-drums`, a Music campaign
- **THEN** the write is refused with the cause that the campaign does not belong to the flag's app
