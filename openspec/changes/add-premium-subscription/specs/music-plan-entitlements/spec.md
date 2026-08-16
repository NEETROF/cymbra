## ADDED Requirements

### Requirement: Ranked plans with an explicit unlock set each

The system SHALL define the plans `free`, `beta` and `premium`, ranked in that order. Each plan
SHALL map to a set of named **unlocks** (for example `catalog.unlimited`, `soundfonts.library`,
`soundfont_library.extended`). The `premium` unlock set SHALL be fixed in code. The `beta` unlock
set SHALL be a runtime configuration (`plans.beta.features`) evaluated as the **intersection** of
the configured keys with the `premium` set, so `beta` can never grant an unlock that `premium`
does not. `free` SHALL grant no unlock. Consumers SHALL ask "does the effective plan grant unlock
X" and MUST NOT compare plan names directly.

#### Scenario: Premium unlock set is not editable at runtime

- **WHEN** an operator edits any runtime configuration
- **THEN** the set of unlocks granted by `premium` is unchanged

#### Scenario: Beta set is capped by premium

- **WHEN** `plans.beta.features` contains a key that is not in the `premium` set
- **THEN** that key is ignored and `beta` grants only the keys present in both

#### Scenario: Beta subset is tunable without release

- **WHEN** an operator adds `soundfonts.library` to `plans.beta.features`
- **THEN** beta users are granted that unlock at the next evaluation, without an app or backend release

### Requirement: Multi-source expiring entitlement ledger

The system SHALL keep an entitlement ledger where each row records the user, the product, the
plan, the **source** (`apple`, `google`, `web`, `code`, `admin`), an opaque provider or issuer
reference, `starts_at`, `ends_at` (nullable only for `admin`), a status, and — for `code` rows —
the campaign that produced them. Rows SHALL be upserted by `(source, provider_ref)` so a repeated
provider event moves an existing row forward instead of creating a duplicate. Rows SHALL never be
deleted except by account erasure; lapse, cancellation and revocation are status/`ends_at`
transitions. The ledger MUST NOT store names, addresses, emails, card or invoice data.

#### Scenario: Repeated provider event updates one row

- **WHEN** the same subscription is reported twice with a later expiry
- **THEN** exactly one row exists for that `(source, provider_ref)` and its `ends_at` is the later value

#### Scenario: Cancellation keeps the row until its end

- **WHEN** a provider reports a subscription cancelled with a future `ends_at`
- **THEN** the row is marked cancelled, stays active until `ends_at`, and is not deleted

#### Scenario: No personal billing data is stored

- **WHEN** the ledger schema and every write path are inspected
- **THEN** only identifiers, plan, dates, status and provider references are present

### Requirement: Effective plan is the highest-ranked active row, premium beats beta

The effective plan of a user SHALL be the highest-ranked plan among rows that are active now
(`starts_at ≤ now < effective_end`, not revoked), where `effective_end` for `code` rows is the
campaign's end date unless the row carries its own override. With no active row the effective
plan is `free`. An active `premium` row SHALL make the effective plan `premium` regardless of any
`beta` row. The end or revocation of a beta campaign MUST NOT change any row of another source.
A `beta` user MUST be able to purchase or be granted `premium` at any time; the beta row is then
outranked and kept for the record.

#### Scenario: Premium outranks beta

- **WHEN** a user has an active `beta` row and an active `premium` row
- **THEN** the effective plan is `premium`

#### Scenario: Beta campaign end does not touch a paid row

- **WHEN** a beta campaign closes or its end date passes for a user who also has an active `apple`/`google`/`web` `premium` row
- **THEN** the user's effective plan is still `premium` and the paid row is unchanged

#### Scenario: Beta tester subscribes

- **WHEN** a user with an active `beta` row completes a purchase on any channel
- **THEN** a `premium` row is created, the effective plan becomes `premium`, and the `beta` row remains recorded

#### Scenario: No row means free

- **WHEN** a user has no active row
- **THEN** the effective plan is `free` and no unlock is granted

### Requirement: Grace on lapse, then degradation to free without loss

A row whose provider reports a grace or billing-retry state SHALL remain active until
`ends_at + plans.grace_days` (runtime config). After that, or on refund/revocation, the
effective plan degrades. Degradation MUST NOT delete or alter user data (favorites, offline
cache, imported fonts, points, badges); only future gate decisions change.

#### Scenario: Billing retry keeps access during grace

- **WHEN** a provider reports the subscription in billing retry and `ends_at` has passed by less than `plans.grace_days`
- **THEN** the row is still active and the effective plan unchanged

#### Scenario: Grace exhausted degrades to free

- **WHEN** `ends_at + plans.grace_days` has passed with no renewal
- **THEN** the effective plan is computed without that row

#### Scenario: Degradation loses no data

- **WHEN** a premium user degrades to free
- **THEN** their favorites, cached scores, imported fonts, points and badges are intact

### Requirement: The effective plan is readable by the app in one call

The system SHALL expose the caller's effective plan through one RPC returning the plan, its
source, its end date, the beta campaign end date when a beta row is active, where the
subscription is managed, and whether a purchase can be started from the calling platform. The
answer SHALL be computed server-side; the app MUST NOT derive the plan from roles or from local
purchase state.

#### Scenario: Plan snapshot for a beta tester

- **WHEN** a user with an active `code` row of a beta campaign calls the plan RPC
- **THEN** the answer is plan `beta`, source `code`, the campaign end date, and purchase allowed for the platform

#### Scenario: Plan snapshot for a store subscriber on another platform

- **WHEN** a user with an active `apple` row calls the plan RPC from an Android or desktop build
- **THEN** the answer is plan `premium`, source `apple`, "managed on the App Store", and purchase not offered here

### Requirement: Kill-switch restores pre-plan behaviour

A runtime flag `plans.enabled` (default off) SHALL gate every plan-aware decision. When off, every
consumer SHALL behave as if the effective plan were `free` and the plan RPC SHALL report `free`
with purchase not offered, so the product behaves exactly as before this change.

#### Scenario: Flag off means today's behaviour

- **WHEN** `plans.enabled` is off
- **THEN** no unlock is granted to anyone, the paywall is hidden, and the ledger is untouched

### Requirement: Entitlement data follows the account and is erased with it

Entitlement rows, redemptions and billing events of a user SHALL be erased by the account
erasure job. Erasure MUST NOT call a provider to cancel a store subscription (the user cancels
on the store), but SHALL cancel a web-MoR subscription through the provider API when one is
active, so a deleted account is not billed.

#### Scenario: Erasure purges the ledger

- **WHEN** an account is erased
- **THEN** no entitlement row, redemption or billing event references that user afterwards

#### Scenario: Erasure cancels an active web subscription

- **WHEN** an erased account had an active `web` row
- **THEN** a cancellation is requested from the web provider before the row is purged
