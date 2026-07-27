## 1. Session store — revoke by id

- [x] 1.1 Add `revoke_by_id(user_id, session_id)` to `SessionStore` (auth): revoke a family only when it belongs to `user_id`; absent/foreign/already-revoked → successful no-op. Implement in `PgSessionStore` (scoped `DELETE`/mark) and `FakeSessionStore`.
- [x] 1.2 Host-testable tests: revoke-by-id ends only the targeted session; a foreign id is a no-op and reveals nothing; `list_for_user` reflects the change.

## 2. Auth module + port — authenticated session ops

- [x] 2.1 Extend `AuthPort` (auth-port) with `list_sessions(user_id)`, `revoke_session(user_id, session_id)`, `revoke_all_sessions(user_id)` (the admin path reuses `revoke_all_sessions(target)` with authorization enforced at the gRPC layer); implement in `AuthModule` over the session store.
- [x] 2.2 Unit-test the module ops (list scoping, revoke-by-id, sign-out-everywhere), reusing the existing fakes.

## 3. gRPC AuthService

- [ ] 3.1 Add proto messages/RPCs: `ListSessions`, `RevokeSession(id)`, `RevokeAllSessions` (caller from the internal token), and `RevokeAccountSessions(user_id)` (admin-gated). Regenerate stubs (`flutter_rust_bridge`/protoc as applicable).
- [ ] 3.2 Server adapter: self ops read `user_id` from the `AuthIdentity` extension (like `LinkIdentity`); mount the admin RPC behind the strict interceptor + admin-role guard (as `GrantRole`). Record an audit entry for the admin revoke (acting admin + target).
- [ ] 3.3 gRPC handler tests: self ops scope to the caller; admin op requires admin; non-admin denied.

## 4. Web-auth HTTP surface (browser, cookie-aware)

- [ ] 4.1 Add `GET /web/auth/sessions` (list, flag the current one via the request cookie's family id), `POST /web/auth/sessions/revoke` (by id), and `POST /web/auth/logout-all` (revoke_all + expired `Set-Cookie`). Reuse the existing CORS-credentials + CSRF header.
- [ ] 4.2 Host-testable handler tests: list flags current; revoke-by-id; logout-all revokes every session and clears the cookie; all scoped to the cookie's account.

## 5. Back office — UI

- [ ] 5.1 "Active sessions" view + Pinia store (`Async<T>` union): list sessions, revoke one, sign out everywhere — via the web-auth seam; label the current device; localized errors (no raw codes).
- [ ] 5.2 Admin: a "revoke sessions" action on the account directory (`RolesView`) calling `RevokeAccountSessions`; confirm-guarded; success/failure surfaced in the union.
- [ ] 5.3 Unit + e2e: store tests (inject fake); e2e for list → revoke one → sign-out-everywhere, and the admin revoke on a target row; assert no raw error codes leak.

## 6. Docs & checks

- [ ] 6.1 Document the threat model + the access-token TTL residual window (revocation is immediate at the refresh layer; access tokens coast to expiry) in the back-office README / security notes.
- [ ] 6.2 Run the gates: backend `cargo test`/`clippy`/`llvm-cov`; back-office `yarn lint && yarn format:check && yarn typecheck && yarn test:coverage && yarn e2e`.
- [ ] 6.3 `openspec validate add-session-management --strict` passes.
