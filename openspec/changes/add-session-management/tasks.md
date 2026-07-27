## 1. Backend — session revocation store

- [x] 1.1 Add a transactional, audience-scoped `revoke_account_sessions_audited(target, admin, audience) -> count` to `SessionStore` (Pg + fake): one `DELETE ... WHERE user_id AND audience` + audit INSERT in the same transaction, returning `rows_affected`. Reuse the existing `revoke_all` for self sign-out-everywhere.
- [x] 1.2 Durable audit table `auth.session_revocation_audit` (migration `0003`): id, target_user_id, acting_admin, audience, revoked_count, at.
- [x] 1.3 Host tests: audience-scoped admin revoke records the audit (admin + target + audience + count); self `revoke_all` cuts every session.

## 2. Auth module + port

- [x] 2.1 `AuthPort`/`AuthModule`: `revoke_all_sessions(user_id)` (self) and `revoke_account_sessions(admin, target, audience)` (admin, delegates to the audited store method).
- [x] 2.2 Unit tests (self sign-out-everywhere; admin audience-scoped + audited).

## 3. gRPC AuthService

- [x] 3.1 Add `RevokeAllSessions` (self, caller from the access token) and `RevokeAccountSessions` (admin) to the proto; regenerate stubs (tonic build.rs; back-office `yarn gen`).
- [x] 3.2 Adapter: self op scopes to the `AuthIdentity` user_id; the admin op is gated by `require_admin` and scoped to the identity's audience.
- [x] 3.3 Adapter tests: self op uses the caller id; missing identity → unauthenticated; admin op requires admin; non-admin denied.

## 4. Back office — admin action only

- [x] 4.1 An admin "Revoke sessions" action on the Roles directory (`RevokeAccountSessions`), behind the sessions store (`Async` op union). Confirm the destructive action; surface success/failure in the view (not a silent store). Route-gated (admin) + server-gated.
- [x] 4.2 Store unit test + e2e (a failed admin revoke surfaces the localized error). (Self-service session management is NOT surfaced in the BO — it belongs to the mobile app; see the follow-up change.)

## 5. Docs & checks

- [x] 5.1 Document the threat model + the access-token TTL residual window (revocation is immediate at the refresh layer; access tokens coast to expiry).
- [x] 5.2 Run the gates: backend `cargo test`/`clippy`/`llvm-cov`; back-office `yarn lint && yarn format:check && yarn typecheck && yarn test:coverage && yarn e2e`.
- [x] 5.3 `openspec validate add-session-management --strict` passes.
