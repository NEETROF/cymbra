# session-management Specification

## Purpose
TBD - created by archiving change add-session-management. Update Purpose after archive.
## Requirements
### Requirement: A user can sign out of every session

The system SHALL provide an authenticated operation that revokes **every** session for
the caller's account (sign out everywhere). It SHALL derive the account from the
validated access token — never a request-supplied id — and MUST NOT affect any other
account. After it succeeds, no existing refresh token for the account SHALL be able to
refresh. This operation is exposed on the authenticated gRPC `AuthService` for native
clients; the browser back office does not surface it (a moderator/admin who needs it can
reset their password, which already revokes all sessions).

#### Scenario: Sign out everywhere revokes all of the caller's sessions

- **WHEN** a signed-in user invokes sign-out-everywhere
- **THEN** every session for their account is revoked, and any subsequent refresh (this device or another) fails, while other accounts are unaffected

### Requirement: An admin can revoke all sessions for an account

The system SHALL provide an operation that revokes every session for an explicitly
targeted account, **scoped to the caller's audience** (an admin cannot cut sessions in an
app they do not administer). It SHALL be permitted only for a caller holding the `admin`
role and SHALL be recorded in a durable, queryable audit trail (acting admin, target,
audience, count). It MUST NOT require the target's refresh token.

#### Scenario: Admin revokes a compromised account's sessions

- **WHEN** an admin revokes the sessions of a target account
- **THEN** every session of that account **in the admin's audience** is revoked so it can no longer refresh, the account's sessions in other audiences are untouched, and a durable audit entry records the acting admin, target, audience, and count

#### Scenario: A non-admin cannot revoke another account's sessions

- **WHEN** a caller without the `admin` role invokes the admin revoke operation
- **THEN** the request is denied and no sessions are revoked

### Requirement: Session revocation does not invalidate already-issued access tokens

Revoking a session SHALL take effect at the refresh layer immediately, but an access
token already issued for that session SHALL remain valid until its normal expiry. This
residual window is bounded by the access-token TTL and MUST be documented.

#### Scenario: A revoked session's access token still works until it expires

- **WHEN** a session is revoked while its short-lived access token is still within its TTL
- **THEN** in-flight calls with that access token continue to succeed until it expires, while any attempt to refresh the session fails

