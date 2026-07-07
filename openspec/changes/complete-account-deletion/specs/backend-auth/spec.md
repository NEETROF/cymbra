## ADDED Requirements

### Requirement: Account-deletion erasure

When a user account is deleted, the auth module SHALL erase every piece of data it
holds for that user: the `local_credentials` row (matched by the user's email) and
all `sessions` rows for the user's id. After erasure the auth module MUST NOT retain
that user's password hash, verification token, reset token, or any refresh token.
The erasure MUST be **idempotent** — running it for a user whose auth data is
already gone is a successful no-op — so a retried deletion converges. Erasing the
auth data of a user who signed up via OIDC only (no local credential) MUST also
succeed as a no-op on the credentials side.

#### Scenario: Local credentials removed
- **WHEN** an account whose user has a local credential is deleted
- **THEN** the `local_credentials` row for that email is removed and the email becomes free to register again

#### Scenario: Sessions removed
- **WHEN** an account is deleted
- **THEN** every `sessions` row for that user id is removed and any subsequent refresh with one of those tokens is rejected

#### Scenario: Idempotent re-run
- **WHEN** the erasure runs again for a user whose auth data has already been removed
- **THEN** it completes successfully without error

#### Scenario: OIDC-only user has no credential to remove
- **WHEN** an account with no local credential (OIDC sign-up only) is deleted
- **THEN** the sessions are removed and the missing local credential is not treated as an error
