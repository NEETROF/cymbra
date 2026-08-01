## MODIFIED Requirements

### Requirement: Admin-only role granting within a scope

The backend SHALL expose authenticated operations for an administrator to grant and
revoke a role for an account within a scope (e.g. grant `music/moderator`). These
operations SHALL be **scope-matched**: to grant or revoke a role in scope `S`, the
caller MUST hold `admin` in `S`, or hold `global/admin` (the cross-scope break-glass).
A caller who is `admin` in one scope but not in the target scope MUST be rejected with
`PERMISSION_DENIED`, and no role changes — the token's audience alone SHALL NOT confer
authority over another scope. A non-admin caller MUST likewise be rejected with
`PERMISSION_DENIED`. Granting the `admin` role SHALL itself require the caller to satisfy
this scope-matched admin check for the target scope. Grants SHALL be idempotent
(granting an already-held role is a no-op success).

#### Scenario: Admin grants moderator in their own scope

- **WHEN** a `music/admin` grants `music/moderator` to an account
- **THEN** the account holds `music/moderator` and gains moderator access on its next token

#### Scenario: Admin cannot act outside their scope

- **WHEN** a `music/admin` (without `global/admin`) attempts to grant or revoke any role in the `live` scope
- **THEN** the request is rejected with `PERMISSION_DENIED` and no role changes

#### Scenario: Global admin acts across scopes

- **WHEN** a `global/admin` grants `live/moderator` to an account
- **THEN** the account holds `live/moderator`, because `global/admin` is the cross-scope break-glass

#### Scenario: Global admin grants a global role

- **WHEN** a `global/admin` grants `global/admin` to another account
- **THEN** the account holds `global/admin`, because granting in the `global` scope requires `admin` in `global`

#### Scenario: Only a global admin can grant a global role

- **WHEN** a `music/admin` (without `global/admin`) attempts to grant or revoke any role in the `global` scope
- **THEN** the request is rejected with `PERMISSION_DENIED` and no role changes

#### Scenario: Non-admin cannot grant roles

- **WHEN** a caller without admin in the target scope invokes grant or revoke
- **THEN** the request is rejected with `PERMISSION_DENIED` and no role changes

#### Scenario: Revoking removes access

- **WHEN** an admin authorized for the target scope revokes `music/moderator` from an account
- **THEN** the account no longer holds that role and loses moderator access after token refresh

#### Scenario: Granting admin requires admin in the target scope

- **WHEN** a caller who is not `admin` in the target scope (nor `global/admin`) attempts to grant the `admin` role in that scope
- **THEN** the request is rejected and no role is granted
