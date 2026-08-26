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

The verification SHALL apply to the **effective** scope about to be stored — the
request-supplied one, else the scope preserved from the existing override, else the registry
default — and SHALL run only when the write would **set or change** the stored scope. A write
that leaves the stored scope untouched — disabling a flag, editing an unrelated field, a save
that re-submits the same scope — is never subject to the check: it cannot introduce a dangling
scope, and it keeps every flag disablable while the campaign directory is down. An incident
must never be un-mitigatable because an unrelated service is.

The verification SHALL test **existence, not openness**. A closed campaign's scope stays valid,
because closing a campaign is how a feature is withdrawn from its members *without* a flag edit;
invalidating the stored scope would break that and force an edit precisely when none is wanted.
Existence is also **kind-agnostic**: a campaign of any kind satisfies it, trials included — the
scope's evaluation semantics do not depend on kind, and filtering by kind would refuse
legitimately evaluable scopes.

The verification SHALL fail closed: when campaign existence cannot be determined, a write that
would set or change the scope SHALL be refused rather than stored unverified. An override write
is a rare, retryable operator action, so refusing it costs little, while storing an unverifiable
scope reintroduces exactly the silent failure this requirement exists to prevent. The two
refusal causes — the scope **names no campaign**, and existence **could not be determined** —
SHALL be distinct typed errors, so a client can tell "fix the scope" from "retry later"; the
check that produces them is therefore fallible by design, answering exists, does not exist, or
cannot determine — never collapsing an error into either definite answer.

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

- **WHEN** any caller sets or changes a rollout scope to `beta:<key>` where no campaign has that key
- **THEN** the write is rejected with a typed error and no override is stored

#### Scenario: A closed campaign's scope is still accepted

- **WHEN** an override is written naming a campaign that exists but is closed
- **THEN** the write succeeds, because closing withdraws access without requiring a flag edit

#### Scenario: An unverifiable check refuses the write

- **WHEN** campaign existence cannot be determined and the write would set or change the stored scope
- **THEN** the write is refused rather than stored unverified, with a typed error distinct from
  the one for a scope naming no campaign

#### Scenario: Disabling a beta-scoped flag survives a directory outage

- **WHEN** the campaign directory is unreachable and a write disables a flag whose stored scope
  is `beta:<key>` without changing that scope
- **THEN** the write succeeds, because it leaves the stored scope untouched

#### Scenario: An unrelated edit to a dangling scope saves

- **WHEN** a flag's stored scope names no campaign and a write changes only the flag's value
- **THEN** the write succeeds and the stored scope stays exactly as it was

#### Scenario: Evaluation is unchanged

- **WHEN** a stored beta-scoped override is evaluated
- **THEN** the audience it grants is exactly what it was before this verification existed
