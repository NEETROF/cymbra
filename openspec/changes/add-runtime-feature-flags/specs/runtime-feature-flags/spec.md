## ADDED Requirements

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

The system SHALL expose an authenticated read that returns the caller's **effective** flags/config
(respecting scope and identity), and the app SHALL fetch them at **launch and on resume** so
feature entry points reflect the current flags. The backend remains authoritative — the app view
is for presentation, not enforcement.

#### Scenario: App reflects current flags on launch

- **WHEN** the app starts and fetches effective flags
- **THEN** it shows or hides feature entry points according to those flags

#### Scenario: App refreshes on resume

- **WHEN** the app resumes
- **THEN** it refreshes the effective flags so a recent change is reflected

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
