## ADDED Requirements

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

## MODIFIED Requirements

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
