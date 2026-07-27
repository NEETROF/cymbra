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
- Let a signed-in user list their active sessions, revoke one by id, and sign out
  everywhere — with the current session identifiable.
- Let an admin revoke all sessions for a target account.
- Reuse the existing `SessionStore` unchanged; keep operations thin and
  authorization-gated.

**Non-Goals:**
- Revoking already-issued **access tokens** mid-life. They are stateless JWTs valid
  until their ~15 min TTL; revoking the refresh session bounds the window. (Introducing
  an access-token denylist/introspection is explicitly out of scope.)
- Changing rotation, reuse detection, TTLs, or session storage (`backend-auth`).
- Per-session device metadata (user-agent/IP/last-seen) beyond `{id, audience}` — a
  possible follow-up; this change ships the revoke levers first.

## Decisions

**1. Identify sessions by the family `id`, revoke by id — never by the raw token.**
`list_for_user` already returns the family `id`. Add `revoke_by_id(user_id, id)` to the
store (authorization-scoped: the id must belong to `user_id`) so a user can end a
session they aren't holding. Logout-by-token stays for the cookie flow. The raw refresh
token is never exposed to a listing.

**2. Operations live on the authenticated `AuthService` (gRPC); the web-auth HTTP
surface proxies the self-service ones for the browser.**
`ListSessions` / `RevokeSession(id)` / `RevokeAllSessions` take the caller's `user_id`
from the internal access token (like `LinkIdentity`). `RevokeAccountSessions(user_id)`
is **admin-gated** (same guard as `GrantRole`). The back office calls the self-service
actions via the cookie-aware web-auth surface (so "sign out everywhere" also clears the
current cookie) and the admin action via gRPC-web. Native clients get the gRPC methods
directly.

**3. "Sign out everywhere" clears the current browser cookie too.**
On the web-auth surface, `revoke-all` runs `revoke_all(user_id)` **and** returns an
expired `Set-Cookie` (like logout), so the calling tab ends locally as well; other tabs
lose refresh on their next attempt (bounded by the access-token TTL).

**4. Mark the current session in the listing.**
The web-auth `list` reads the caller's own refresh cookie, resolves its family id, and
flags that entry as `current: true` so the UI can label "This device" and avoid a
foot-gun. gRPC clients that hold their own refresh token can do the same client-side.

**5. Idempotent + safe.** Revoking an already-revoked/absent session is a successful
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
