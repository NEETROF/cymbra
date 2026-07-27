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

- **Sign out everywhere (self)** — an authenticated operation that revokes **all** of
  the caller's sessions, exposed on the gRPC `AuthService` for native clients. The
  back office does not surface it (a moderator/admin can reset their password, which
  already revokes all sessions); it is the API the **mobile app** will call from a
  Profile screen (follow-up change).
- **Admin session revocation** — an admin can revoke **all** sessions of a target
  account (e.g. a compromised moderator), **scoped to their audience** and recorded in
  a durable audit trail. Surfaced in the back office on the Roles directory.
- **Surfaces**: the authenticated **gRPC `AuthService`** (both operations). No bespoke
  web-auth HTTP surface — that would re-implement access-token auth on the cookie
  surface for no gain. No new session semantics: thin, authorization-gated wrappers
  over the existing `SessionStore` (`revoke_all` + a new transactional, audience-scoped
  admin revoke-and-audit).

## Capabilities

### New Capabilities
- `session-management`: authenticated revocation of refresh-token sessions — a user
  signs out of every session (self), and an admin revokes all sessions of a target
  account (audience-scoped, audited). Thin, authorization-gated wrappers over the
  existing durable `SessionStore`; rotation, reuse detection, and token TTLs are
  unchanged.

### Modified Capabilities
<!-- None. `backend-auth` session/rotation/TTL requirements are unchanged; this change
     ADDS authenticated revoke operations over the same session store. -->

<!-- Scope note: self-service session *listing* + per-device revoke (an "active
     sessions" screen) is intentionally out of this change — it belongs to the mobile
     app and is deferred to a follow-up. -->


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
