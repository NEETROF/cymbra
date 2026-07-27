## 1. Session store — revoke by id

- [x] 1.1 Add `revoke_by_id(user_id, session_id)` to `SessionStore` (auth): revoke a family only when it belongs to `user_id`; absent/foreign/already-revoked → successful no-op. Implement in `PgSessionStore` (scoped `DELETE`/mark) and `FakeSessionStore`.
- [x] 1.2 Host-testable tests: revoke-by-id ends only the targeted session; a foreign id is a no-op and reveals nothing; `list_for_user` reflects the change.

## 2. Auth module + port — authenticated session ops

- [x] 2.1 Extend `AuthPort` (auth-port) with `list_sessions(user_id)`, `revoke_session(user_id, session_id)`, `revoke_all_sessions(user_id)` (the admin path reuses `revoke_all_sessions(target)` with authorization enforced at the gRPC layer); implement in `AuthModule` over the session store.
- [x] 2.2 Unit-test the module ops (list scoping, revoke-by-id, sign-out-everywhere), reusing the existing fakes.

## 3. gRPC AuthService

- [x] 3.1 Add proto messages/RPCs: `ListSessions`, `RevokeSession(id)`, `RevokeAllSessions` (caller from the internal token), and `RevokeAccountSessions(user_id)` (admin-gated). Regenerate stubs (tonic build.rs; back-office `yarn gen` for group 5).
- [x] 3.2 Server adapter: self ops read `user_id` from the `AuthIdentity` extension (like `LinkIdentity`); the admin RPC is guarded by `require_admin` on the identity roles. Durable audit: a new `auth.session_revocation_audit` table (migration `0003`) records acting admin + target + count on the admin revoke — a queryable trail, not a log line.
- [x] 3.3 gRPC handler tests: self ops scope to the caller; missing identity → unauthenticated; admin op requires admin; non-admin denied.

## 4. Cookie handling on sign-out-everywhere (design change — no new HTTP surface)

Session ops go through the **authenticated gRPC `AuthService`** (called from the BO
over the existing gRPC-web transport), not a bespoke web-auth HTTP surface — that would
have re-implemented access-token auth on the cookie surface for no gain.

- [x] 4.1 "Sign out everywhere" in the BO = `RevokeAllSessions` (gRPC) **then** the
  existing `POST /web/auth/logout` (clears the HttpOnly cookie) + local sign-out. No new
  backend endpoint needed. (Implemented as part of the store in group 5.)

## 5. Back office — UI

- [x] 5.1 "Active sessions" view + Pinia store (`Async<T>` union): list sessions, revoke one, sign out everywhere — via the web-auth seam; label the current device; localized errors (no raw codes).
- [x] 5.2 Admin: a "revoke sessions" action on the account directory (`RolesView`) calling `RevokeAccountSessions`; confirm-guarded; success/failure surfaced in the union.
- [x] 5.3 Unit + e2e: store tests (inject fake); e2e for list → revoke one → sign-out-everywhere, and the admin revoke on a target row; assert no raw error codes leak.

## 6. Docs & checks

- [x] 6.1 Document the threat model + the access-token TTL residual window (revocation is immediate at the refresh layer; access tokens coast to expiry) in the back-office README / security notes.
- [x] 6.2 Run the gates: backend `cargo test`/`clippy`/`llvm-cov`; back-office `yarn lint && yarn format:check && yarn typecheck && yarn test:coverage && yarn e2e`.
- [x] 6.3 `openspec validate add-session-management --strict` passes.
