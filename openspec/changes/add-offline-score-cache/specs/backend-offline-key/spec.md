## ADDED Requirements

### Requirement: Issue a stable per-user offline cache secret

The backend SHALL expose an authenticated operation that returns the caller's
per-user offline cache secret — a high-entropy value (at least 32 bytes) used by
the client as one input to its offline-cache key derivation. The secret MUST be
created on first request and returned unchanged on every subsequent request for
the same user, so the user's favorites decrypt consistently across all of their
devices. The operation MUST be owner-scoped: a caller only ever receives their
own secret. Unauthenticated requests MUST be rejected.

#### Scenario: First request creates and returns the secret

- **WHEN** an authenticated user requests their offline cache secret for the
  first time
- **THEN** a new high-entropy secret is generated, persisted for that user, and
  returned

#### Scenario: Subsequent requests return the same secret

- **WHEN** the same user requests their offline cache secret again
- **THEN** the previously stored secret is returned unchanged

#### Scenario: Secret is owner-scoped

- **WHEN** a user requests the offline cache secret
- **THEN** only that user's own secret is returned, never another user's

#### Scenario: Unauthenticated request rejected

- **WHEN** the request arrives without a valid authenticated identity
- **THEN** it is rejected and no secret is created or returned

### Requirement: Rotate a user's offline cache secret

The backend SHALL support rotating a user's offline cache secret so that all
previously written offline caches for that user become undecryptable once their
devices re-derive keys. After rotation, the next request for the secret MUST
return the new value. Account deletion MUST invalidate (rotate or remove) the
user's secret so residual offline cache files can no longer be decrypted.

#### Scenario: Rotation changes the returned secret

- **WHEN** a user's offline cache secret is rotated
- **THEN** the next request for that user returns a different secret value than
  before rotation

#### Scenario: Account deletion invalidates the secret

- **WHEN** a user's account is deleted
- **THEN** the user's offline cache secret is invalidated so prior offline caches
  can no longer be decrypted

### Requirement: Offline secret is never exposed to other users or unauthenticated callers

The offline cache secret SHALL be treated as sensitive material: it MUST only be
returned over the authenticated, owner-scoped operation, MUST NOT appear in logs
or in any listing/administrative surface that exposes it to other users, and MUST
be stored at rest under the backend's existing protections for sensitive data.

#### Scenario: Secret is not returned to a different user

- **WHEN** any user calls the operation
- **THEN** the response contains only the caller's own secret and no other user's

#### Scenario: Secret is not logged

- **WHEN** the operation is served
- **THEN** the secret value does not appear in application logs
