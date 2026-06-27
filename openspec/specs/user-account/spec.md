# user-account Specification

## Purpose
TBD - created by archiving change add-cymbra-id. Update Purpose after archive.
## Requirements
### Requirement: Account with linked identities

The user module SHALL model an internal account (`users`) that can have one or more
linked provider identities (`user_identities`), where each identity is a
`(provider, subject)` pair — `provider` is `local`, `google`, or `apple`, and
`subject` is the email (for `local`) or the OIDC subject. A `(provider, subject)`
pair MUST be unique across all accounts (an identity belongs to exactly one
account). The module owns this data; other modules reach it only through the
`user` port.

#### Scenario: Account owns its identities

- **WHEN** an account is created with an initial identity
- **THEN** a `users` row and a linked `user_identities` row are created, and the
  account can carry additional identities later

### Requirement: Resolve or provision account by identity

The user module SHALL resolve an existing account from a `(provider, subject)`
pair, or provision a new account (with that first identity) when none exists.
Repeated resolution for the same pair MUST return the same account.

#### Scenario: First identity provisions an account

- **WHEN** the module is asked to resolve-or-provision a `(provider, subject)` that
  is not linked anywhere
- **THEN** a new account is created with that identity and returned

#### Scenario: Known identity resolves to its account

- **WHEN** the module resolves a `(provider, subject)` already linked to an account
- **THEN** the existing account is returned and no duplicate is created

### Requirement: Link an identity to an existing account

The user module SHALL attach a new `(provider, subject)` identity to an existing
account. If the pair is already linked to a different account, the link MUST be
rejected (uniqueness preserved).

#### Scenario: Identity linked to the account

- **WHEN** a new `(provider, subject)` not bound elsewhere is linked to an account
- **THEN** a `user_identities` row is added for that account

#### Scenario: Already-linked identity is rejected

- **WHEN** linking a `(provider, subject)` already bound to a different account
- **THEN** the module rejects the link and changes nothing

### Requirement: Unlink an identity from an account

The user module SHALL remove a `(provider, subject)` identity from an account,
**unless it is the account's last remaining identity** (which MUST be rejected to
avoid locking the user out).

#### Scenario: Identity unlinked

- **WHEN** an identity is unlinked and the account still has at least one other
  identity
- **THEN** the `user_identities` row is removed

#### Scenario: Removing the last identity is rejected

- **WHEN** unlinking would leave the account with no identity
- **THEN** the module rejects it and changes nothing

### Requirement: List linked identities

The user module SHALL return the providers linked to the authenticated caller's
account (provider + subject + linked-at), without exposing other accounts' data.

#### Scenario: Caller lists their identities

- **WHEN** an authenticated user lists their linked identities
- **THEN** the module returns that account's `(provider, subject, linked_at)`
  entries only

### Requirement: Retrieve account

The user module SHALL return the authenticated caller's account: profile fields,
preferences, roles, version, and `updated_at`. It MUST return only the caller's own
account.

#### Scenario: Account retrieved

- **WHEN** an authenticated user requests their account
- **THEN** the module returns their profile, preferences, roles, version, and
  `updated_at`

#### Scenario: Retrieval is isolated per user

- **WHEN** an authenticated user retrieves their account
- **THEN** the response contains only that user's data

### Requirement: Update account

The user module SHALL let the authenticated caller update their editable profile
fields and preferences. Each account MUST carry a monotonically increasing
`version` and an `updated_at` timestamp, and updates MUST use optimistic
concurrency: a write succeeds only when the supplied version matches the stored
version.

#### Scenario: Successful update increments version

- **WHEN** an authenticated user updates their account with the current known
  version
- **THEN** the changes are persisted, the version is incremented, and the new
  version and `updated_at` are returned

#### Scenario: Stale update is rejected

- **WHEN** an update supplies a version older than the stored version
- **THEN** the module rejects it with gRPC status `ABORTED` (conflict) and returns
  the current server version
- **AND** the stored account is unchanged

### Requirement: Delete account

The user module SHALL delete the authenticated caller's account, erasing the
`users` row together with its linked identities and roles. After deletion the
account MUST NOT resolve for any of its former identities. (The auth module purges
the related credentials and sessions; detailed erasure semantics are finalized in
implementation.)

#### Scenario: Account erased

- **WHEN** an authenticated user deletes their account
- **THEN** the account, its `user_identities`, and its `user_roles` are removed
- **AND** none of its former identities resolves to an account afterwards

### Requirement: Per-user scoped roles (RBAC scaffold)

Each account SHALL carry a **set of scoped roles** stored as `(scope, role)` pairs
(`scope` is `global`, `music`, or `live`), independently of any provider identity,
with `UNIQUE(user_id, scope, role)`. Roles MUST NOT be derived from OIDC token
claims — they are assigned server-side. A newly provisioned account SHALL receive a
default `(global, user)` role. The module SHALL expose, through the `user` port, the
**effective roles for a given scope** (roles whose scope is `global` or the
requested scope) and a `has_role(scope, role)` check. This change provides the
scaffold (storage + read + check); concrete role-assignment admin endpoints are out
of scope.

#### Scenario: Default role on provisioning

- **WHEN** an account is provisioned
- **THEN** its role set contains `(global, user)`

#### Scenario: An account can hold multiple scoped roles

- **WHEN** an account is granted additional scoped roles (e.g. `(live, broadcaster)`,
  `(music, teacher)`)
- **THEN** all granted pairs are stored and returned for that account, each at most
  once

#### Scenario: Effective roles are scoped per app

- **WHEN** the effective roles for scope `live` are requested
- **THEN** the result contains the account's `global` roles plus its `live` roles,
  and excludes roles scoped to other apps (e.g. `music`)

#### Scenario: Roles are independent of identity

- **WHEN** a user signs in via any linked provider (local, Google, or Apple)
- **THEN** the same scoped role set is resolved for the account, regardless of
  provider

