## Context

Refresh-token sessions are durable in Postgres (`auth.sessions`), rotated with reuse
detection, and audience-bound (change: `durable-sessions-postgres`). The
`SessionStore` trait already exposes everything this change needs:

- `revoke(refresh_token)` — end one session (used by logout).
- `revoke_all(user_id)` — end every session for an account (used by password reset +
  account deletion).
- `list_for_user(user_id) -> Vec<SessionInfo{ id, audience }>` — the account's live
  session families.

What's missing is **authenticated read/revoke operations** callers can invoke. Logout
today needs the refresh token itself (from the cookie or gRPC body); there is no way to
revoke a session you're *not* currently holding, list your sessions, or (for an admin)
revoke someone else's. The `add-web-auth-cookies` change added the browser web-auth
HTTP surface and confirmed the access token carries the caller's `user_id` + roles.

## Goals / Non-Goals

**Goals:**
- Let a signed-in user sign out of every session (self "sign out everywhere").
- Let an admin revoke all sessions of a target account, scoped and audited.
- Keep operations thin and authorization-gated over the existing `SessionStore`.

**Non-Goals:**
- A self-service "active sessions" screen (list + per-device revoke + "this device"
  flag). For an admin-only back office it's overkill (and password reset already
  revokes all your sessions); it belongs to the **mobile app** and is deferred to a
  follow-up change. This change therefore adds **no** `ListSessions`/`RevokeSession`
  RPCs, no `sid` claim, and no per-session device metadata.
- Revoking already-issued **access tokens** mid-life. They are stateless JWTs valid
  until their ~15 min TTL; revoking the refresh session bounds the window. (An
  access-token denylist/introspection is explicitly out of scope.)
- Changing rotation, reuse detection, TTLs, or session storage (`backend-auth`).

## Decisions

**1. Two operations on the authenticated gRPC `AuthService` — no bespoke web-auth
surface.**
`RevokeAllSessions` (self) takes the caller's `user_id` from the internal access token
(like `LinkIdentity`, via the `AuthIdentity` extension). `RevokeAccountSessions(user_id)`
is **admin-gated** (`require_admin` on the identity roles). A cookie-aware HTTP surface
was considered and rejected — it would re-implement access-token validation on the
web-auth surface for no benefit.

**2. The admin revoke is audience-scoped.**
`RevokeAccountSessions` cuts the target's sessions **only in the caller's token audience**
(`id.audience`), so a `music` admin can't nuke a target's `live` sessions. Erasure paths
(password reset, account deletion) keep using the all-audiences `revoke_all`.

**3. The admin revoke + its audit are one transaction.**
A single store method `revoke_account_sessions_audited(target, admin, audience)` does the
audience-scoped `DELETE` and the audit `INSERT` in one transaction and returns
`rows_affected` as the exact count — no pre-count race, and a failure rolls the delete
back rather than losing the trail. The audit lives in a dedicated append-only table
`auth.session_revocation_audit` (migration `0003`): `{target_user_id, acting_admin,
audience, revoked_count, at}` — queryable, not a `tracing` line lost in the stream.

**4. Idempotent + safe.** Revoking an already-revoked/absent session is a successful
no-op (matches `SessionStore` semantics), so retries converge and a revoked id can't be
used to probe existence beyond the caller's own account.

## Risks / Trade-offs

- **Access-token residual window** → a revoked session's *access* token still works
  until it expires (~15 min). Documented; the short TTL is the mitigation. Callers that
  need instant cut-off must shorten the access TTL or adopt introspection (out of scope).
- **Admin abuse / audit** → `RevokeAccountSessions` is admin-only; reuse the existing
  role-grant audit pattern (record who revoked whose sessions) so the action is
  traceable.
- **Enumeration** → revoke/list are strictly scoped to the caller's `user_id` (or, for
  the admin RPC, a role-gated explicit target), so a user can neither list nor revoke
  another account's sessions.
- **Multi-tab logout UX** → consistent with `add-web-auth-cookies`: sign-out-everywhere
  is immediate at the refresh layer, open tabs coast on their in-memory access token
  until TTL. Acceptable and documented.
