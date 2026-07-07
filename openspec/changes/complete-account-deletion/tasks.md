## 1. Auth module — erasure primitives

- [ ] 1.1 Add a `delete_credentials(email)` method to the auth `CredentialRepo` trait + `PgCredentialRepo` (`DELETE FROM local_credentials WHERE email = $1`); idempotent (0 rows = success)
- [ ] 1.2 Expose a `delete_sessions_for_user(user_id)` path over the existing `DELETE FROM sessions WHERE user_id = $1` in `session_pg` (wire it through the `SessionStore` seam if not already reachable)
- [ ] 1.3 Unit tests: deleting an absent email / absent user is a no-op success

## 2. Jobs — purge_user job kind

- [ ] 2.1 Register a `purge_user` job kind (payload `{ user_id }`) in `cymbra_jobs` (mirror how `VERIFICATION_EMAIL` is declared)
- [ ] 2.2 Expose its spec via the registry so producers can enqueue it through `jobs.enqueue`

## 3. Worker — purge handler (admin_svc, atomic)

- [ ] 3.1 Give `cymbra-worker` an `admin_svc` connection/pool from `CYMBRA_ADMIN_DATABASE_URL` (worker-only; never in an app service)
- [ ] 3.2 Implement the `purge_user` handler: in ONE `admin_svc` transaction — resolve email from `user_account.user_identities` WHERE `user_id` AND `provider='local'`; `DELETE FROM auth.local_credentials` by that email (skip if none); `DELETE FROM auth.sessions` by `user_id`; `DELETE FROM user_account.users` by `user_id`; commit
- [ ] 3.3 Make the handler idempotent (all deletes no-op when rows absent) and OIDC-only-safe (no local identity → skip credentials)
- [ ] 3.4 Register the handler in the worker's `JobRegistry`

## 4. User service — enqueue on delete

- [ ] 4.1 Change `DeleteAccount` so the user module enqueues a `purge_user{user_id}` job (via `jobs.enqueue`) instead of / in addition to its direct `DELETE FROM users` — the job now owns the user_account delete so it stays in the single atomic transaction
- [ ] 4.2 Keep the gRPC contract unchanged (`DeleteAccountResponse {}`, returns after enqueue)

## 5. Tests (Rust, ≥80% lines)

- [ ] 5.1 Integration: create account (local) → `DeleteAccount` → run the worker → `local_credentials`, `sessions`, `users`, `user_identities` for the user are gone
- [ ] 5.2 Integration: after deletion, re-`SignUpLocal` with the same email SUCCEEDS (no "already registered")
- [ ] 5.3 Integration: a pre-deletion refresh token is rejected after the purge runs
- [ ] 5.4 Integration: OIDC-only account (no local credential) deletes cleanly (no error)
- [ ] 5.5 Idempotency: enqueuing/running `purge_user` twice for the same user is a success no-op

## 6. Validate & ship

- [ ] 6.1 `openspec validate complete-account-deletion --strict` passes
- [ ] 6.2 `cargo fmt --all --check` + `cargo clippy --workspace --all-targets -- -D warnings` clean
- [ ] 6.3 `cargo llvm-cov --workspace --fail-under-lines 80` (per CLAUDE.md excludes) passes
- [ ] 6.4 Manual: on prod, delete a throwaway account and confirm re-signup with the same email works
