## ADDED Requirements

### Requirement: Moderator role in the music scope

The system SHALL support a `moderator` role in the `music` scope, in addition to the
existing `user` and `admin` roles, stored in the same scoped `user_roles` model. A
`music`-audience access token SHALL carry the `moderator` role when the account holds it
(via the existing effective-role resolution that unions `global` and the audience scope).
Roles SHALL remain scoped: holding `moderator` (or `admin`) in `music` SHALL confer no
authority in any other module's scope.

#### Scenario: Moderator role appears in the music token

- **WHEN** an account holding `music/moderator` signs in to the `music` audience
- **THEN** its access token's role set includes `moderator`

#### Scenario: Scope isolation

- **WHEN** an account holds `moderator` (or `admin`) only in the `music` scope
- **THEN** it has no moderator/admin authority in the `live` scope or any other module

### Requirement: Moderators and admins may access unvalidated scores

An identity authorized as `music` `moderator` or `admin` (or `global/admin`) SHALL be
permitted to list/search and open (fetch the bytes of) catalog scores in **any**
moderation status, and to use the privileged moderation-status filter — the operations
that #1 restricted to admins only are hereby available to moderators as well. A normal
caller remains limited to `accepted` scores.

#### Scenario: Moderator uses the privileged status filter

- **WHEN** a `music/moderator` searches with the moderation-status filter set to `pending`
- **THEN** the request is authorized and returns `pending` scores

#### Scenario: Moderator opens a pending score

- **WHEN** a `music/moderator` requests the bytes of a `pending` or `rejected` score
- **THEN** the bytes are served so the moderator can review it

#### Scenario: Normal caller still blocked

- **WHEN** a caller without `moderator`/`admin` supplies the status filter or requests a
  non-`accepted` score's bytes
- **THEN** the request is refused (permission denied / not found) as before

### Requirement: Admin-only role granting within a scope

The backend SHALL expose authenticated operations for an administrator to grant and
revoke a role for an account within a scope (e.g. grant `music/moderator`). These
operations SHALL be guarded so that only an `admin` (in that scope, or `global/admin`)
may invoke them; a non-admin caller MUST be rejected with `PERMISSION_DENIED`. Granting
the `admin` role SHALL itself require the caller to be an `admin`. Grants SHALL be
idempotent (granting an already-held role is a no-op success).

#### Scenario: Admin grants moderator

- **WHEN** a `music/admin` grants `music/moderator` to an account
- **THEN** the account holds `music/moderator` and gains moderator access on its next token

#### Scenario: Non-admin cannot grant roles

- **WHEN** a caller without admin invokes grant or revoke
- **THEN** the request is rejected with `PERMISSION_DENIED` and no role changes

#### Scenario: Revoking removes access

- **WHEN** an admin revokes `music/moderator` from an account
- **THEN** the account no longer holds that role and loses moderator access after token refresh

#### Scenario: Granting admin requires admin

- **WHEN** a non-admin attempts to grant the `admin` role
- **THEN** the request is rejected and no role is granted

### Requirement: First administrator is bootstrapped out-of-band

Because there is no in-app path to mint the very first administrator, the system SHALL
provide an out-of-band way for an operator with database access to seed the initial
`music/admin` (e.g. an ops command or SQL seed). Thereafter, administrators SHALL be able
to grant further admin/moderator roles through the guarded role-granting operations.

#### Scenario: Operator seeds the first admin

- **WHEN** an operator with database access runs the bootstrap for a chosen account
- **THEN** that account holds `music/admin` and can sign in to the back office

#### Scenario: Bootstrapped admin promotes others

- **WHEN** the seeded `music/admin` grants `music/moderator` to another account
- **THEN** that account becomes a moderator without any further operator/database action
