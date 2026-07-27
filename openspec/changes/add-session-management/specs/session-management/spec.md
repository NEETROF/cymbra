## ADDED Requirements

### Requirement: A user can list their active sessions

The system SHALL provide an authenticated operation that returns the caller's
non-expired session families, each identified by its session id and audience/app, with
the caller's current session flagged. The operation SHALL derive the account from the
validated access token (or, on the browser surface, the session cookie) and MUST NOT
return sessions belonging to any other account.

#### Scenario: List returns only my sessions with the current one flagged

- **WHEN** a signed-in user requests their active sessions
- **THEN** the response lists that account's non-expired sessions (id + audience), flags the session the request was made from as current, and includes no other account's sessions

### Requirement: A user can revoke one of their sessions by id

The system SHALL provide an authenticated operation that revokes a single session
identified by its id, only when that session belongs to the caller. A revoked session's
refresh token SHALL no longer rotate. Revoking an id that is absent or already revoked
SHALL be a successful no-op.

#### Scenario: Revoke a specific session ends it

- **WHEN** a signed-in user revokes one of their session ids
- **THEN** that session can no longer refresh, and the user's other sessions are unaffected

#### Scenario: Cannot revoke another account's session

- **WHEN** a user attempts to revoke a session id that does not belong to them
- **THEN** the operation does not revoke it and reveals nothing about that id (no-op / not-found, never another account's data)

### Requirement: A user can sign out everywhere

The system SHALL provide an authenticated operation that revokes every session for the
caller's account. On the browser web-auth surface it SHALL also clear the caller's
refresh cookie (expired `Set-Cookie`). After it succeeds, no existing refresh token for
the account SHALL be able to refresh.

#### Scenario: Sign out everywhere revokes all sessions and clears the cookie

- **WHEN** a signed-in user triggers sign-out-everywhere from the browser
- **THEN** every session for the account is revoked, the response clears the refresh cookie, and any subsequent refresh (this browser or another device) fails

### Requirement: An admin can revoke all sessions for an account

The system SHALL provide an admin-gated operation that revokes every session for an
explicitly targeted account. The operation SHALL be permitted only for a caller with the
admin role and SHALL be auditable (the acting admin and target account are recorded). It
MUST NOT require the target's refresh token.

#### Scenario: Admin revokes a compromised account's sessions

- **WHEN** an admin revokes all sessions for a target account
- **THEN** every session for that account is revoked so it can no longer refresh, and the action is recorded with the acting admin and target

#### Scenario: A non-admin cannot revoke another account's sessions

- **WHEN** a caller without the admin role invokes the admin revoke operation
- **THEN** the request is denied and no sessions are revoked

### Requirement: Session revocation does not invalidate already-issued access tokens

Revoking a session SHALL take effect at the refresh layer immediately, but an access
token already issued for that session SHALL remain valid until its normal expiry. This
residual window is bounded by the access-token TTL and MUST be documented.

#### Scenario: A revoked session's access token still works until it expires

- **WHEN** a session is revoked while its short-lived access token is still within its TTL
- **THEN** in-flight calls with that access token continue to succeed until it expires, while any attempt to refresh the session fails
