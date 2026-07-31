## ADDED Requirements

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

Each flag SHALL have a scope of **global** or **staff-only**. A **staff-only** flag SHALL be in
effect only for admin/moderator identities, so a feature can be exercised by staff before a global
rollout; a **global** flag applies to everyone. Scope SHALL be resolved server-side per the
caller's identity.

#### Scenario: Staff-only feature is limited to staff

- **WHEN** a feature flag is staff-only and enabled
- **THEN** it is in effect for admin/moderator identities and not for normal users

#### Scenario: Global feature applies to everyone

- **WHEN** a feature flag is global and enabled
- **THEN** it is in effect for all users

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
