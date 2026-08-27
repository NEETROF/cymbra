## MODIFIED Requirements

### Requirement: Moderator role in the music scope

The system SHALL support a `moderator` role in the `music` scope, in addition to the
existing `user` and `admin` roles, stored in the same scoped `user_roles` model. A
`music`-audience access token SHALL carry the `moderator` role when the account holds it
(via the existing effective-role resolution that unions `global` and the audience scope).
Roles SHALL remain scoped: holding `moderator` (or `admin`) in `music` SHALL confer no
authority in any other module's scope.

Scope isolation SHALL be **enforced at every authorization gate**, not only asserted by
the role model. A moderation or admin gate SHALL name the scope it protects and admit a
caller only when the caller holds the role in that scope or in the `global` break-glass
scope. A gate MUST NOT test the flat, cross-scope union of the caller's roles — that
union spans every product scope for a `back-office` token, so a flat test grants a
moderator of one product authority over another.

Until every gate is scope-matched, the system SHALL refuse to grant `moderator` in any
scope other than `music`, so the hole cannot be created. The refusal SHALL apply to the
grant path only: revoking a role SHALL remain possible in every scope.

#### Scenario: Moderator role appears in the music token

- **WHEN** an account holding `music/moderator` signs in to the `music` audience
- **THEN** its access token's role set includes `moderator`

#### Scenario: Scope isolation

- **WHEN** an account holds `moderator` (or `admin`) only in the `music` scope
- **THEN** it has no moderator/admin authority in the `live` scope or any other module

#### Scenario: A moderator of another product is refused at a music gate

- **WHEN** an account holding `moderator` in another product scope, signed in to the
  `back-office` audience, calls a music moderation operation
- **THEN** the call is refused, even though its flat role set contains `moderator`

#### Scenario: A global admin still passes every gate

- **WHEN** an account holding `global/admin` calls a music moderation operation
- **THEN** the call is admitted

#### Scenario: Granting a moderator outside music is refused while gates are flat

- **WHEN** an administrator grants `moderator` in a scope other than `music`, and the
  gates are not yet scope-matched
- **THEN** the grant is refused with an explanatory error

#### Scenario: Revocation is never blocked

- **WHEN** an administrator revokes a role in any scope
- **THEN** the revocation succeeds, whatever the grant-path restriction is
