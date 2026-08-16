## ADDED Requirements

### Requirement: Two plans with a fixed premium unlock set

The system SHALL define the plans `free` and `premium`. `premium` SHALL map to a set of named
**unlocks** fixed in code (`catalog.unlimited`, `soundfonts.library`,
`soundfont_library.extended`, `scores.extended_quotas`, `offline.cache`); `free` SHALL grant
no unlock. Consumers
SHALL ask "does the effective plan grant unlock X" and MUST NOT compare plan names. Security
guardrails (catalog download/enumeration limits, auth throttles) MUST NOT depend on the plan or
on any beta membership.

#### Scenario: Premium unlock set is not editable at runtime

- **WHEN** an operator edits any runtime configuration
- **THEN** the set of unlocks granted by `premium` is unchanged

#### Scenario: Guardrails ignore plan and betas

- **WHEN** a premium user or a beta member exceeds the catalog download burst or enumeration cap
- **THEN** the request is refused exactly as for a free user

### Requirement: Multi-source expiring entitlement ledger

The system SHALL keep an entitlement ledger where each row records the user, the product, the
**source** (`apple`, `google`, `web`, `code`, `admin`), an opaque provider or issuer reference,
`starts_at`, `ends_at` (nullable only for `admin`), a status, and — for trial rows — the campaign
that produced them. Rows SHALL be upserted by `(source, provider_ref)` so a repeated provider event
moves an existing row forward instead of creating a duplicate. Rows SHALL never be deleted except
by account erasure; lapse, cancellation and revocation are status/`ends_at` transitions. The
ledger MUST NOT store names, addresses, emails, card or invoice data.

#### Scenario: Repeated provider event updates one row

- **WHEN** the same subscription is reported twice with a later expiry
- **THEN** exactly one row exists for that `(source, provider_ref)` and its `ends_at` is the later value

#### Scenario: Cancellation keeps the row until its end

- **WHEN** a provider reports a subscription cancelled with a future `ends_at`
- **THEN** the row is marked cancelled, stays active until `ends_at`, and is not deleted

#### Scenario: No personal billing data is stored

- **WHEN** the ledger schema and every write path are inspected
- **THEN** only identifiers, dates, status and provider references are present

### Requirement: Effective plan is premium while any row is active; betas never touch paid rows

The effective plan SHALL be `premium` when at least one row is active now (`starts_at ≤ now <
effective_end`, not revoked, grace included) and `free` otherwise. When several rows are active
the latest `effective_end` governs. No campaign operation (enrolment closing, feature-beta closing,
membership revocation, code revocation) SHALL create, shorten or end a row whose source is
`apple`, `google` or `web`. A user in any beta MUST be able to purchase at any time.

#### Scenario: Two rows, latest end governs

- **WHEN** a user has a trial row ending March 1 and an Apple row ending June 1
- **THEN** the effective plan is `premium` until June 1

#### Scenario: Trial end does not touch a purchase

- **WHEN** a premium-trial tester's trial row ends while their Apple row is active
- **THEN** the effective plan is still `premium` and the Apple row is unchanged

#### Scenario: Beta member buys premium

- **WHEN** a member of a feature beta on the free plan completes a purchase
- **THEN** a `premium` row is created, the effective plan is `premium`, and the membership is unchanged

#### Scenario: No row means free

- **WHEN** a user has no active row
- **THEN** the effective plan is `free` and no unlock is granted

### Requirement: Beta memberships are a separate axis with two campaign kinds

The system SHALL record beta **memberships** — the account belongs to zero or more campaigns —
independently of the plan. A campaign SHALL have a kind: **`premium_trial`**, whose enrolment
also creates a `premium` row ending `duration_days` after **that tester's** enrolment (campaign
field, fixed at creation, default 90), or **`feature`**, whose enrolment creates a membership
only, with no end date, ended for every member when the operator closes the campaign. A
membership SHALL be active iff its campaign is not closed, its `ends_at` is null or in the future,
and it is not revoked. At most one premium trial SHALL be active per account at a time. Active
memberships SHALL be exposed to feature-flag evaluation and to the app.

#### Scenario: Trial duration is per tester

- **WHEN** two testers enrol in a 90-day trial campaign on January 1 and February 1
- **THEN** their premium rows end on April 1 and May 2 respectively

#### Scenario: Closing enrolment shortens nobody

- **WHEN** an admin closes enrolment of a trial campaign
- **THEN** no new member can enrol and every existing trial row keeps its end date

#### Scenario: Feature beta ends when closed

- **WHEN** an admin closes a feature campaign
- **THEN** every membership of that campaign becomes inactive at that moment and no plan row changes

#### Scenario: Second trial refused while one runs

- **WHEN** an account with an active premium trial enrols in another trial campaign
- **THEN** enrolment is refused and nothing changes

#### Scenario: Feature-beta member on the free plan

- **WHEN** a free user is enrolled in the `midi-drums` feature beta
- **THEN** their effective plan is `free` and their memberships contain `midi-drums`

### Requirement: Grace on lapse, then degradation to free without loss

A row whose provider reports a grace or billing-retry state SHALL remain active until
`ends_at + plans.grace_days` (runtime config). After that, or on refund/revocation, the
effective plan degrades. Degradation MUST NOT delete or alter what the user owns (favorites
list, imported fonts, uploads, points-redeemed pianos, points, badges, progression); only future
gate decisions change and plan-only content is withdrawn (see below).

#### Scenario: Billing retry keeps access during grace

- **WHEN** a provider reports the subscription in billing retry and `ends_at` has passed by less than `plans.grace_days`
- **THEN** the row is still active and the effective plan unchanged

#### Scenario: Grace exhausted degrades to free

- **WHEN** `ends_at + plans.grace_days` has passed with no renewal
- **THEN** the effective plan is computed without that row

#### Scenario: Degradation loses no data

- **WHEN** a premium user degrades to free
- **THEN** their favorites list, imported fonts, uploads, points-redeemed pianos, points and badges are intact

### Requirement: Plan and memberships are readable by the app in one call

The system SHALL expose the caller's effective plan through one RPC returning the plan, its
source, its end date, the active premium trial (campaign and end) if any, the list of active beta
memberships (campaign, kind, joined date), where the subscription is managed, and whether a
purchase can be started from the calling platform. The answer SHALL be computed server-side; the
app MUST NOT derive plan or memberships from roles or local purchase state.

#### Scenario: Snapshot for a trial tester

- **WHEN** a user in a premium trial calls the plan RPC
- **THEN** the answer is plan `premium`, source `code`, the trial campaign and its end, and purchase allowed for the platform

#### Scenario: Snapshot for a store subscriber on another platform

- **WHEN** a user with an active `apple` row calls the plan RPC from an Android or desktop build
- **THEN** the answer is plan `premium`, source `apple`, "managed on the App Store", and purchase not offered here

#### Scenario: Snapshot lists feature betas

- **WHEN** a member of two feature betas calls the plan RPC
- **THEN** both campaigns appear in the memberships with their kind and join date

### Requirement: Kill-switch restores pre-plan behaviour

A runtime flag `plans.enabled` (default off) SHALL gate every plan-aware decision. When off, every
consumer SHALL behave as if the effective plan were `free` and no membership existed, and the plan
RPC SHALL report `free` with purchase not offered, so the product behaves exactly as before this
change.

#### Scenario: Flag off means today's behaviour

- **WHEN** `plans.enabled` is off
- **THEN** no unlock is granted to anyone, no beta rollout matches, the paywall is hidden, and the ledger is untouched

### Requirement: Entitlement data follows the account and is erased with it

Entitlement rows, memberships, redemptions and billing events of a user SHALL be erased by the
account erasure job. Erasure MUST NOT call a provider to cancel a store subscription (the user
cancels on the store), but SHALL cancel a web-MoR subscription through the provider API when one
is active, so a deleted account is not billed.

#### Scenario: Erasure purges the ledger

- **WHEN** an account is erased
- **THEN** no entitlement row, membership, redemption or billing event references that user afterwards

#### Scenario: Erasure cancels an active web subscription

- **WHEN** an erased account had an active `web` row
- **THEN** a cancellation is requested from the web provider before the row is purged

### Requirement: Plan-only content is withdrawn after the entitlement ends, never during grace

The system SHALL **withdraw the content a plan granted** when a user's effective plan drops to
`free` — trial ended, subscription lapsed past `plans.grace_days`, comp expired, admin
revocation: it SHALL rotate the user's offline cache secret once (so cached catalog
scores become unreadable on every device) and SHALL keep refusing plan-only SoundFont bytes
through the existing entitlement gate. Withdrawal SHALL happen **once per lapse**, be idempotent,
and MUST NOT run while a row is in grace or billing retry. Withdrawal MUST NOT touch anything the
user owns: imported `.sf2`, uploads, points-redeemed fonts, the favorites list, points, badges,
progression. Re-acquiring premium later SHALL require no repair: favorites are intact and content
is simply fetched again. A feature-beta closing SHALL trigger no withdrawal (it granted no
content).

#### Scenario: Trial ends

- **WHEN** a premium-trial tester's row passes `ends_at` and no other row is active
- **THEN** the offline cache secret is rotated once and plan-only SoundFont downloads are refused from then on

#### Scenario: Grace is respected

- **WHEN** a subscriber's row is in billing retry within `plans.grace_days`
- **THEN** no withdrawal happens and the effective plan is still `premium`

#### Scenario: Owned content survives

- **WHEN** withdrawal runs for a user with imported fonts, uploads and points-redeemed pianos
- **THEN** none of those, nor the favorites list, points or badges, is affected

#### Scenario: Idempotent across devices and retries

- **WHEN** the lapse is evaluated several times (sweep job, plan RPC, several devices reconnecting)
- **THEN** the secret is rotated exactly once for that lapse

#### Scenario: Re-subscription needs no repair

- **WHEN** a lapsed user purchases premium again
- **THEN** their favorites are still listed and cached/plan-only content becomes fetchable again

#### Scenario: Feature beta closing withdraws nothing

- **WHEN** a feature campaign is closed
- **THEN** no secret rotation and no download refusal results from it
