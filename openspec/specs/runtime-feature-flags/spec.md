# runtime-feature-flags Specification

## Purpose
TBD - created by archiving change add-runtime-feature-flags. Update Purpose after archive.
## Requirements
### Requirement: A shared, app-scoped flag system reusable across Cymbra apps

The feature-flag system SHALL be a **shared platform component**, reusable by every Cymbra app
(`music`, `live`, and future apps) **without re-implementation**, not coupled to any one app's
module. Each key SHALL have an **app scope**: `all` (applies to every app) or a specific app; the
system SHALL resolve a caller's effective keys from the caller's app (its token audience), so an
app sees only the `all` keys plus its own. This app scope is independent of the rollout scope.

#### Scenario: A per-app key applies only to its app

- **WHEN** a key is scoped to `music` and a `live` caller evaluates flags
- **THEN** that key does not apply to the `live` caller

#### Scenario: A shared key applies to every app

- **WHEN** a key is scoped to `all`
- **THEN** every app's callers resolve it

#### Scenario: A new app reuses the system without re-coding

- **WHEN** a new Cymbra app is added
- **THEN** it consumes the same shared flag system and its own app-scoped keys, without a new implementation

### Requirement: Runtime flag and config store with code defaults

The system SHALL provide a runtime store holding **boolean feature flags** and **typed config
values**, whose source of truth is the database, and SHALL declare every key **in code with a
typed default**. When a key has no stored override, its **code default** SHALL be used. The store
therefore only overrides declared defaults, and an unknown key is not editable.

#### Scenario: Absent key resolves to its code default

- **WHEN** a flag or config key has no stored value
- **THEN** its declared code default is used

#### Scenario: Stored value overrides the default

- **WHEN** an admin sets a value for a declared key
- **THEN** that value is used instead of the default

#### Scenario: Only declared keys are editable

- **WHEN** an edit targets a key not in the code registry
- **THEN** it is rejected

### Requirement: Hot evaluation without redeploy, fail-safe to defaults

The system SHALL evaluate flags/config at runtime so a change takes effect **within seconds
without a redeploy or restart**, across horizontally-scaled instances — via a short-lived cache
with invalidation on change. If the backing store is unreachable, the system SHALL **fall back to
the code defaults** rather than failing, and kill-switch flags SHALL default to their **safe**
state so an outage never silently enables a risky capability.

#### Scenario: Change takes effect within seconds

- **WHEN** an admin changes a flag or config value
- **THEN** the new value is in effect within seconds on all instances, without a redeploy

#### Scenario: Store outage falls back to defaults

- **WHEN** the backing store is unreachable
- **THEN** evaluation uses the code defaults and does not error

#### Scenario: Kill-switch fails safe

- **WHEN** the store is unreachable for a kill-switch flag
- **THEN** it resolves to its safe (disabled) state

### Requirement: A disabled feature is enforced by the backend

When a feature's flag is **off**, the backend SHALL **enforce** the disablement — rejecting the
feature's operations or withholding its data — so the feature is actually unavailable regardless
of the client. Hiding UI in the app is defense-in-depth, not the gate. Disabling a feature SHALL
gate it **without deleting** its data.

#### Scenario: Gated operation rejected when off

- **WHEN** a feature is flagged off and a client invokes its operation
- **THEN** the backend rejects it (or withholds the data), independent of the client UI

#### Scenario: Data is preserved while disabled

- **WHEN** a feature is turned off
- **THEN** its stored data is retained (gated, not deleted) and returns when re-enabled

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

### Requirement: The app receives its effective flags

The system SHALL expose a read that returns the caller's **effective** flags/config together with a
**version/ETag** — **authenticated**, returning the identity-scoped set (roles + app); or
**unauthenticated**, returning the anonymous global (non-staff) set so a pre-account app respects
kill-switches too. The app SHALL fetch them at **launch and on resume** so feature entry points
reflect the current flags. Subsequent fetches
SHALL be able to send the known version so the server can answer **"unchanged"** cheaply. The app
SHALL cache the last set locally for a flicker-free start and fall back to defaults when it has
never fetched. The backend remains **authoritative** — the app view is presentation-only, so a
momentarily stale UI results only in a gated action **failing gracefully** server-side (a
localized unavailable message), never incorrect access.

#### Scenario: App reflects current flags on launch

- **WHEN** the app starts and fetches effective flags
- **THEN** it shows or hides feature entry points according to those flags

#### Scenario: App refreshes on resume, cheaply when unchanged

- **WHEN** the app resumes and re-fetches with its known version
- **THEN** the server answers "unchanged" when nothing changed, or returns the new set otherwise

#### Scenario: Stale client is presentation-only

- **WHEN** the app's cached flags are momentarily behind a just-made change and a user triggers a now-disabled action
- **THEN** the backend rejects it and the app shows a localized "unavailable" message, with no incorrect access

#### Scenario: Flicker-free start from local cache

- **WHEN** the app starts before its first fetch returns
- **THEN** it uses the locally cached last set (or defaults if never fetched) rather than flickering

### Requirement: Client cache is a per-identity snapshot, reset on auth change

The app SHALL cache the effective flags as a **single snapshot fetched in one read** — reads of
individual keys are local and synchronous, never per-key network calls. The snapshot is
**identity-scoped** (it reflects the caller's roles and app). A refresh SHALL be an **atomic
swap** that keeps serving the **last-good snapshot** until a new one arrives, and a **failed**
refresh SHALL keep the previous snapshot rather than clearing to an empty state. On **sign-out**
the app MUST discard the signed-in snapshot and revert to the anonymous/default set; on **user
switch** it MUST NOT reuse the previous user's snapshot and MUST refetch for the new identity. Any
**persisted** cache MUST be keyed by identity (or cleared on sign-out) so one user's flags never
apply to another.

#### Scenario: One fetch, local per-key reads

- **WHEN** the app needs several flag values
- **THEN** it reads them locally from the one fetched snapshot, without a network call per key

#### Scenario: Failed refresh keeps the last-good snapshot

- **WHEN** a refresh fails (offline or server error)
- **THEN** the app keeps serving the previous snapshot rather than dropping to an empty/gap state

#### Scenario: Sign-out clears the signed-in snapshot

- **WHEN** the user signs out
- **THEN** the signed-in snapshot is discarded and the app reverts to the anonymous/default set

#### Scenario: User switch does not inherit the previous user's flags

- **WHEN** one user signs out and another signs in on the same device
- **THEN** the new user's flags are refetched and the previous user's snapshot is not reused (no staff-only leakage)

### Requirement: Flag and config changes are audited

Every change to a flag or config value SHALL be recorded in a durable, append-only audit
capturing the key, the old and new values, the administrator who made it, and when. The audit
SHALL be queryable, mirroring the role-grant audit.

#### Scenario: Editing records an audit entry

- **WHEN** an admin changes a flag or config value
- **THEN** an audit entry records the key, old→new value, the actor, and the time

#### Scenario: Audit is queryable

- **WHEN** an administrator reviews recent changes
- **THEN** the audit can be queried to show who changed which key, and when

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

