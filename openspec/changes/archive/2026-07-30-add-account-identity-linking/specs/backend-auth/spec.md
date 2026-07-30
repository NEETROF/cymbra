## ADDED Requirements

### Requirement: Add a local credential to an existing account

Cymbra ID SHALL expose an authenticated `AuthService.SetLocalCredential(email,
password)` RPC that attaches an email/password credential to the **caller's**
account (change: add-account-identity-linking) — the missing primitive so an
OIDC-only account can also sign in with email. The caller's `user_id` comes from
the validated access token (never a request field). The credential SHALL be
created **unverified** and a verification email SHALL be enqueued, so the password
becomes usable only after the email is verified (mirroring `SignUpLocal`). The RPC
SHALL reject with `ALREADY_EXISTS` when the account already has a local credential
or the email is registered to another account, and with `INVALID_ARGUMENT` when
the password fails the policy. On a failure to bind the identity after the
credential row is written, the server SHALL compensate by erasing that row so a
retry is not blocked.

#### Scenario: Password added to an OIDC account, verified, then usable

- **WHEN** a signed-in Google-only user calls `SetLocalCredential` with a fresh email and a policy-compliant password
- **THEN** the account gains an unverified `local` identity and a verification email is enqueued; a local sign-in is refused with `FAILED_PRECONDITION` until the email is verified, after which it signs in to the **same** account (no second account is created)

#### Scenario: Second password refused

- **WHEN** the account already has a `local` credential
- **THEN** `SetLocalCredential` returns `ALREADY_EXISTS` and no change is made

#### Scenario: Email owned by another account is refused without side effects

- **WHEN** the chosen email is already registered to a different account
- **THEN** `SetLocalCredential` returns `ALREADY_EXISTS` and the other account's credential is left intact
