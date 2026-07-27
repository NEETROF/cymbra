## Why

Cymbra already issues durable, revocable refresh-token **sessions** (rotated, with
reuse detection) and the `SessionStore` implements `revoke(refresh_token)`,
`revoke_all(user_id)`, and `list_for_user(user_id) -> [{id, audience}]`. But those
capabilities are only wired to three inward flows: single-session logout, password
reset, and account deletion. A user has **no way to see their active sessions, sign
out a specific device, or sign out everywhere**, and an admin has **no way to revoke a
compromised moderator/admin's sessions**. When a laptop is lost or a session is
suspected stolen, the only lever today is a full password reset — heavy, and
unavailable to the OIDC-only accounts that have no password.

## What Changes

- **Self-service session management** exposed to the signed-in user:
  - **List active sessions** — the account's non-expired session families (id +
    audience/app), so the current session is identifiable.
  - **Revoke one session by id** — end a specific device/session.
  - **Sign out everywhere** — revoke all of the account's sessions (`revoke_all`).
- **Admin session revocation** — an admin can revoke **all** sessions for a target
  account (e.g. a compromised moderator), from the back office.
- **Surfaces**: extend the browser **web-auth HTTP surface** (cookie-aware, added in
  `add-web-auth-cookies`) for the self-service actions the back office needs, and the
  **gRPC `AuthService`** for the authenticated list/revoke operations (native clients +
  the admin action). No new session semantics — these are thin wrappers over the
  existing `SessionStore` methods.
- **Back-office UI** — an "active sessions" view for the signed-in user (revoke one /
  sign out everywhere) and an admin "revoke sessions" control on the account directory.

## Capabilities

### New Capabilities
- `session-management`: authenticated listing and revocation of refresh-token sessions
  — a user lists/revokes their own sessions (including sign-out-everywhere), and an
  admin revokes all sessions for a target account. Thin, authorization-gated wrappers
  over the existing durable `SessionStore` (list/revoke/revoke_all); rotation, reuse
  detection, and token TTLs are unchanged.

### Modified Capabilities
<!-- None. `backend-auth` session/rotation/TTL requirements are unchanged; this change
     ADDS authenticated read/revoke operations over the same session store. The
     `web-auth-session` cookie surface (add-web-auth-cookies) is extended additively. -->

## Impact

- **Backend**: new authenticated `AuthService` RPCs (list sessions, revoke session by
  id, revoke all / sign-out-everywhere for self; admin revoke-all for a target),
  plus the matching self-service endpoints on the web-auth HTTP surface. Authorization:
  self operations use the caller's `user_id` from the validated access token; the admin
  operation is admin-role-gated like the existing `GrantRole`. Reuses `SessionStore`
  (`list_for_user`/`revoke`/`revoke_all`) — no storage or rotation changes.
- **Back office**: an "active sessions" view + store (list/revoke/sign-out-everywhere)
  and an admin revoke-sessions action on the account directory, behind the existing
  API seam; localized errors, no raw codes.
- **Security/UX**: bounds a lost/stolen session immediately at the refresh layer;
  already-issued **access tokens remain valid until their short (~15 min) TTL** — the
  residual window is documented, not eliminated (a stateless-JWT trade-off).
- **Docs**: threat model + the access-token-TTL residual-window note.
