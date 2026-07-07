## MODIFIED Requirements

### Requirement: Account deletion

The app SHALL let a signed-in user permanently delete their account via
`DeleteAccount`, gated behind **fresh re-authentication** and an explicit
confirmation that states the action is irreversible. Re-authentication SHALL match
the user's method: an email user re-enters their password (verified via
`SignInLocal`), and a Google/Apple user re-runs the native sign-in to produce a
fresh `id_token`. `DeleteAccount` SHALL only be called after re-authentication
succeeds and the user confirms. On success the app SHALL clear the local session
and return to the entry screen. Account deletion SHALL NOT be offered in guest mode.

`DeleteAccount` SHALL perform a **complete erasure** of the user's personal data
across all backend modules — the `user_account` record (with its identities and
roles) **and** the auth module's local credentials and sessions — so that no email,
password hash, OIDC identity, verification/reset token, or refresh token for that
user survives. The erasure SHALL be **atomic** (all-or-nothing) and **idempotent**
(safe to retry after a partial failure). After a successful deletion the freed email
address SHALL be registrable again, and any outstanding refresh token for the
deleted user SHALL be rejected.

#### Scenario: Confirmed deletion after re-authentication
- **WHEN** a signed-in user re-authenticates successfully and then confirms account deletion
- **THEN** the app calls `DeleteAccount`, clears secure storage, and returns to the entry screen

#### Scenario: Re-authentication fails
- **WHEN** the user enters a wrong password (or the re-run OIDC sign-in fails) at the deletion gate
- **THEN** no `DeleteAccount` call is made, the account remains intact, and the app surfaces an authentication error

#### Scenario: Deletion requires explicit confirmation
- **WHEN** the user re-authenticates but does not confirm the irreversible step
- **THEN** no `DeleteAccount` call is made and the account remains intact

#### Scenario: Deleted email can register again
- **WHEN** a user deletes their account and later signs up with the same email address
- **THEN** sign-up succeeds with no "already registered" error and a fresh account is created

#### Scenario: Refresh tokens invalidated on deletion
- **WHEN** an account is deleted while a refresh token for it is still unexpired
- **THEN** using that refresh token is rejected and no new access token is issued
