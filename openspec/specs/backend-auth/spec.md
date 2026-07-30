# backend-auth Specification

## Purpose
TBD - created by archiving change add-cymbra-id. Update Purpose after archive.
## Requirements
### Requirement: Local email/password sign-up

The auth module SHALL let a user create an account with an email and password. The
password MUST be stored only as an **argon2id** hash (never in clear text). On
sign-up the module MUST create the account with the email marked **unverified**
and send a verification message to the email. A sign-up for an email that already
has a local credential MUST be rejected.

#### Scenario: New email signs up

- **WHEN** a user signs up with an email that has no existing local credential and
  an acceptable password
- **THEN** an account and a `local` identity for that email are created
- **AND** only an argon2id hash of the password is stored, with the email marked
  unverified
- **AND** a verification message is sent to the email

#### Scenario: Duplicate local email is rejected

- **WHEN** a user signs up with an email that already has a local credential
- **THEN** the module rejects it with gRPC status `ALREADY_EXISTS`

### Requirement: Email verification required before local sign-in

The auth module SHALL require a local account's email to be verified before it can
sign in. Verification MUST be performed with a single-use, expiring token/code. A
local sign-in attempt for an unverified email MUST be rejected.

#### Scenario: Valid verification confirms the email

- **WHEN** a user submits a valid, unexpired verification token for their email
- **THEN** the email is marked verified

#### Scenario: Sign-in blocked until verified

- **WHEN** a user attempts local sign-in with correct credentials but an unverified
  email
- **THEN** the module rejects it with gRPC status `FAILED_PRECONDITION`

#### Scenario: Invalid or expired verification token

- **WHEN** a verification token is unknown, already used, or expired
- **THEN** the module rejects it and the email remains unverified

### Requirement: OIDC sign-in via Google and Apple

The auth module SHALL authenticate a user from an external OIDC provider ID token
through a pluggable `IdentityVerifier`, supporting **Google** and **Apple**. It
MUST validate the token against the matching issuer's JWKS and verify `iss`,
`aud`, and `exp`, then resolve or provision the account by `(provider, subject)`.
Adding another provider MUST be possible by adding a verifier without changing
callers.

#### Scenario: Valid provider token signs in

- **WHEN** a user presents a Google or Apple ID token with a valid signature,
  issuer, audience, and unexpired lifetime
- **THEN** the account is resolved (or provisioned on first sign-in) by
  `(provider, subject)` and a session is issued

#### Scenario: Token from an untrusted issuer is rejected

- **WHEN** a presented ID token fails signature/issuer/audience/expiry validation,
  or comes from an issuer the backend does not trust
- **THEN** the module rejects it with gRPC status `UNAUTHENTICATED`

### Requirement: Internal-token signing and JWKS publication

The backend SHALL sign internal access tokens with an **asymmetric** signing key
(e.g. EdDSA or RS256) whose header carries a key id (`kid`), and SHALL publish the
corresponding public key(s) at a **JWKS endpoint** so downstream apps (Cymbra
Music, Cymbra Live) can validate Cymbra ID tokens **offline** without calling the
backend per request. Multiple active keys MUST be supported to allow rotation
without invalidating live tokens.

#### Scenario: Token verifiable via the published JWKS

- **WHEN** a downstream service fetches the JWKS and validates an access token
- **THEN** the signature verifies against the published key matching the token `kid`

#### Scenario: Rotation keeps live tokens valid

- **WHEN** the signing key is rotated and both old and new keys remain published
- **THEN** access tokens signed by the previous key still validate until they expire

### Requirement: Internal session tokens

Sign-in SHALL target an **app audience** (e.g. `music` or `live`), validated
against a configured allow-list; an unknown audience MUST be rejected. On success
(local or OIDC) the auth module SHALL issue the backend's **own** session tokens —
a short-lived access token (signed with the asymmetric key above) and a refresh
token. The access token SHALL set `aud` to the target app and carry the account's
`user_id` and the **effective role set for that audience** (roles whose scope is
`global` or that app's scope, read from the user module — never from the provider
token). Protected gRPC methods MUST be authorized by validating the internal
**access** token (not the provider token), and an interceptor MUST reject requests
whose access token is missing, invalid, or expired. Access-token validation SHALL
be **offline** (signature + claims against the published JWKS), so protected calls
MUST NOT depend on the session store. The access token SHALL be **short-lived**
(target ~15 minutes) and the refresh token **long-lived and sliding** (target ~30
days) — the refresh token is the effective session length. The refresh token MUST
be exchangeable for a new access token and MUST be **rotated on use** with **reuse
detection**: presenting an expired, revoked, or already-rotated refresh token MUST
be rejected, and replay of a rotated token SHALL revoke the whole session family.
Rotation and reuse detection MUST be **atomic** — the check-and-rotate SHALL be a
single durable, conditional operation so that concurrent refreshes cannot both
succeed. An expired **access** token alone MUST NOT require re-authentication while
the refresh token is still valid. Each session/refresh token is **bound to the
audience chosen at sign-in**; `Refresh` preserves that audience (it takes no
audience parameter), and tokens are never shared across apps — a user signs in to
each app **independently (one login per app)**.

#### Scenario: Expired access token is refreshed without re-login

- **WHEN** a client's access token has expired but its refresh token is still valid
- **THEN** the client obtains a new access token via refresh, with no credential
  re-entry by the user

#### Scenario: Successful sign-in issues an audience-scoped token

- **WHEN** a sign-in succeeds for an allowed audience (valid local credentials with
  a verified email, or a valid OIDC token)
- **THEN** the module returns a signed access token whose `aud` is that app and
  whose roles are the effective set for that audience, plus a refresh token

#### Scenario: Unknown audience is rejected

- **WHEN** a sign-in targets an audience that is not in the configured allow-list
- **THEN** the module rejects it with gRPC status `INVALID_ARGUMENT` and issues no
  tokens

#### Scenario: Wrong local password is rejected

- **WHEN** a local sign-in supplies an incorrect password
- **THEN** the module rejects it with gRPC status `UNAUTHENTICATED` and issues no
  tokens

#### Scenario: Protected call requires a valid internal token

- **WHEN** a request to a protected method has a missing, invalid, or expired
  internal access token
- **THEN** the interceptor rejects it with gRPC status `UNAUTHENTICATED`

#### Scenario: Refresh rotates the token

- **WHEN** a valid, unexpired refresh token is presented
- **THEN** a new access token is issued and the refresh token is rotated

#### Scenario: Revoked or expired refresh is rejected

- **WHEN** a refresh token that is expired, already rotated, or revoked is presented
- **THEN** the module rejects it with gRPC status `UNAUTHENTICATED`

#### Scenario: Reused refresh token revokes the session

- **WHEN** an already-rotated refresh token is replayed
- **THEN** the module rejects it and revokes the whole session family so the stolen
  token chain is dead

#### Scenario: Concurrent refresh of the same token yields one winner

- **WHEN** the same refresh token is presented by two concurrent requests
- **THEN** at most one succeeds and rotates the token; the other is rejected with
  `UNAUTHENTICATED` (and, being a replay of a now-rotated token, revokes the family)

### Requirement: Sign out and session revocation

The auth module SHALL let an authenticated user sign out, revoking the current
session's refresh token so it can no longer be used, and SHALL support revoking all
of an account's sessions (e.g. after a password reset). Revocation SHALL be served
from the durable session store. A revoked session's refresh token MUST be rejected
on use. Because sessions are stored durably per account, the module SHALL be able to
**enumerate an account's active sessions** (for revoke-all and for a future
"active devices" surface) via an indexed lookup by `user_id`.

#### Scenario: Sign-out revokes the session

- **WHEN** an authenticated user signs out
- **THEN** that session's refresh token is revoked and subsequent refresh with it is
  rejected with `UNAUTHENTICATED`

#### Scenario: Revoke-all ends every session

- **WHEN** all sessions for an account are revoked
- **THEN** every previously issued refresh token for that account is rejected on use

#### Scenario: Account's active sessions can be listed

- **WHEN** the module looks up an account's sessions by `user_id`
- **THEN** it returns that account's non-expired session families

### Requirement: Local credential hardening

The auth module SHALL enforce a configurable **password policy** at sign-up and
password reset, **rate-limit** local sign-in attempts with a temporary lockout
after repeated failures, and **throttle** verification/reset email sends. Limits
MUST be tracked centrally (shared cache) so they hold across instances.

#### Scenario: Weak password rejected

- **WHEN** a sign-up or reset supplies a password failing the policy
- **THEN** the module rejects it with `INVALID_ARGUMENT` and stores nothing

#### Scenario: Repeated failures trigger lockout

- **WHEN** local sign-in fails more than the configured threshold for an account/IP
- **THEN** further attempts are temporarily refused with `RESOURCE_EXHAUSTED` until
  the window elapses

#### Scenario: Email sends are throttled

- **WHEN** verification or reset emails are requested faster than the configured rate
- **THEN** the excess requests are throttled and no additional email is sent

### Requirement: Resend email verification

The auth module SHALL let a user request a new verification email for an unverified
local account, issuing a fresh single-use token and invalidating the previous one,
subject to throttling.

#### Scenario: Resend issues a fresh token

- **WHEN** a user requests verification resend for an unverified email within the
  allowed rate
- **THEN** a new single-use verification token is sent and the previous one no
  longer verifies

### Requirement: Password reset

The auth module SHALL let a user reset a local password via email: a request issues
a single-use, expiring reset token, and submitting that token with a new policy-
compliant password updates the argon2id hash and revokes existing sessions. The
request flow MUST NOT reveal whether an email exists (no account enumeration).

#### Scenario: Reset updates the password and revokes sessions

- **WHEN** a user submits a valid reset token with a policy-compliant new password
- **THEN** the stored argon2id hash is replaced and all existing sessions are revoked

#### Scenario: Request does not reveal account existence

- **WHEN** a password reset is requested for an email
- **THEN** the response is the same whether or not a local account exists for it

### Requirement: Explicit provider linking

The auth module SHALL let an authenticated user link an additional identity — an
OIDC provider or a local email/password credential — to their **current** account.
The module MUST NOT auto-merge accounts by email. Linking an identity that is
already bound to a different account MUST be rejected.

#### Scenario: Authenticated user links another provider

- **WHEN** a signed-in user links a new identity (e.g. adds Google, or adds a local
  email/password) and that identity is not bound elsewhere
- **THEN** the identity is attached to the user's current account

#### Scenario: Linking an already-bound identity is rejected

- **WHEN** a signed-in user tries to link an identity already bound to a different
  account
- **THEN** the module rejects it with gRPC status `ALREADY_EXISTS` and links nothing

### Requirement: Explicit provider unlinking

The auth module SHALL let an authenticated user unlink one of their linked
identities (OIDC provider or local credential), **except the last remaining one** —
removing the last identity would lock the user out and MUST be rejected. Unlinking a
local credential MUST remove its stored secret.

#### Scenario: Unlink a non-last identity

- **WHEN** a signed-in user unlinks an identity and at least one other identity
  remains on the account
- **THEN** the identity is detached (and a local credential's secret removed)

#### Scenario: Unlinking the last identity is rejected

- **WHEN** a signed-in user tries to unlink their only remaining identity
- **THEN** the module rejects it with gRPC status `FAILED_PRECONDITION` and changes
  nothing

### Requirement: Role-based authorization guard

The backend SHALL provide a role-based authorization guard (`require_role(r)`, with
`is_admin` = `require_role("admin")`) that reads the **role set** from the validated
internal access token. The guard provides the enforcement mechanism only; concrete
admin endpoints are out of scope.

#### Scenario: Guard blocks users without the required role

- **WHEN** a user whose role set lacks the required role invokes a guarded method
- **THEN** the backend rejects it with gRPC status `PERMISSION_DENIED`

#### Scenario: Guard allows users holding the required role

- **WHEN** a user whose role set contains the required role invokes the guarded
  method
- **THEN** the request is permitted to proceed

### Requirement: Durable session persistence

The auth module SHALL store refresh-token session state — the session **family**
(`user_id`, `audience`, `current_rt`, `expires_at`) — in the backend's durable
system of record (Postgres, the `auth` schema), NOT solely in an in-memory cache.
Session state MUST survive a restart or outage of the cache tier: a cache (Redis)
restart MUST NOT invalidate active sessions or force re-authentication. The cache
tier SHALL hold only disposable data (rate-limit and email-throttle counters) and
therefore MUST NOT be required to be highly available or durable for sessions to
work. Session records SHALL carry an expiry (`expires_at`) that bounds them to the
refresh-token lifetime; expired sessions MUST be treated as invalid on use
regardless of cleanup timing, and a periodic reap SHALL remove expired rows for
table hygiene.

#### Scenario: Sessions survive a cache-tier restart

- **WHEN** the cache (Redis) service is restarted or lost while a user holds a
  valid refresh token
- **THEN** the user's session remains valid and a subsequent refresh succeeds
  without credential re-entry

#### Scenario: Cache loss does not affect session validity

- **WHEN** rate-limit/throttle counters in the cache are lost
- **THEN** only those counters reset; no session is revoked and no user is signed
  out

#### Scenario: Expired session is rejected even before reap runs

- **WHEN** a refresh token whose session `expires_at` is in the past is presented,
  before the reap job has deleted the row
- **THEN** the module rejects it with gRPC status `UNAUTHENTICATED`

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

