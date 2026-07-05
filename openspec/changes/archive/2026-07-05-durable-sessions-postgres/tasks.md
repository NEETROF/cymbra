## 1. Schema

- [x] 1.1 Add `backend/auth/migrations/0002_sessions.sql`: table `sessions` (`id` UUIDv7 PK, `user_id` TEXT, `audience` TEXT, `current_rt_hash` TEXT, `expires_at` TIMESTAMPTZ, `created_at` TIMESTAMPTZ DEFAULT now())
- [x] 1.2 Add a UNIQUE index on `current_rt_hash` (rotate lookup) and a plain index on `user_id` (revoke-all / enumeration); index `expires_at` for the reap
- [x] 1.3 Confirm the migration runs on `cymbra-server` boot (embedded `sqlx::migrate!`) against a live DB

## 2. Session-store seam + pure core

- [x] 2.1 Define a `SessionStore` **trait** in `backend/auth/src/session.rs` (`create`, `rotate`, `revoke`, `revoke_all`, `list_for_user`) returning the existing `Rotated`/session types
- [x] 2.2 Add `session_core` (pure, host-testable): refresh-token encode/parse (`"{fid}.{secret}"`), secret hashing (SHA-256 → `current_rt_hash`), expiry decision, and rotate-outcome classification (win / replay / invalid-or-expired)
- [x] 2.3 Add `FakeSessionStore` (in-memory) implementing the trait for unit tests
- [x] 2.4 Unit-test `session_core`: token round-trips, hashing is stable, expired-is-invalid, and the three rotate outcomes

## 3. Postgres implementation

- [x] 3.1 Add `backend/auth/src/session_pg.rs` — `PgSessionStore { pool, refresh_ttl }` implementing `SessionStore`
- [x] 3.2 `create`: INSERT a family row (`id` UUIDv7, hashed secret, `expires_at = now()+refresh_ttl`); return the encoded refresh token
- [x] 3.3 `rotate`: atomic guarded `UPDATE … WHERE id=$fid AND current_rt_hash=$old AND expires_at > now() RETURNING user_id, audience`, sliding `expires_at`; on 0 rows, in the same tx SELECT the family by `id` and DELETE it if unexpired (replay/theft), else reject
- [x] 3.4 `revoke` (delete family by current token's fid) and `revoke_all` (`DELETE … WHERE user_id=$1`); `list_for_user` (`SELECT … WHERE user_id=$1 AND expires_at > now()`)
- [x] 3.5 Export `PgSessionStore` from `lib.rs`; keep `SessionStore` (trait) exported

## 4. Wire the composition root

- [x] 4.1 Change `AuthModule` to hold `Arc<dyn SessionStore>` (was concrete `SessionStore`); update `AuthModule::new` signature
- [x] 4.2 In `backend/server/src/main.rs`, construct `PgSessionStore` on the `auth_pool` and pass it in; keep the Redis `Cache` wired for rate-limit only
- [x] 4.3 Update `cache.rs`/`main.rs` comments: Redis is now a disposable, non-HA cache (rate-limit + email throttles); remove the Redis-backed session code paths

## 5. Reap job (worker)

- [x] 5.1 Add a `SESSION_REAP` job kind + `JobSpec` to `backend/jobs/src/registry.rs` (channel + default retry, mirroring `ORPHAN_REAP`)
- [x] 5.2 Seed a `session_reap_*` row in `backend/jobs/migrations/0007_seed_schedules.sql` (cron cadence; `enabled = true`)
- [x] 5.3 Add the `session_reap` handler in `backend/worker` that `DELETE`s expired rows, using a `CYMBRA_AUTH_DATABASE_URL` (`auth_svc`) pool — add that pool to the worker wiring (mirror the `orphan_reap` / user-pool pattern)
- [x] 5.4 Document `CYMBRA_AUTH_DATABASE_URL` for the worker in `backend/.env.example` and `backend/deploy/.env.prod.example`

## 6. Tests, coverage, docs

- [x] 6.1 `#[ignore]` live-DB integration tests for `PgSessionStore`: create→rotate→refresh, replay-revokes-family, concurrent-rotate single-winner, revoke / revoke_all, expired rejected, `list_for_user`
- [x] 6.2 Update `AuthModule` unit tests to use `FakeSessionStore`; verify sign-in/refresh/sign-out flows unchanged
- [x] 6.3 Run `cargo llvm-cov --workspace --fail-under-lines 80` (with the existing ignore regex); keep pure logic host-tested so the seam meets the gate
- [x] 6.4 `cargo fmt --all` + `cargo clippy --workspace --all-targets -- -D warnings`; run `scripts/check_boundaries.py` (no new cross-module dependency)
- [x] 6.5 Update `backend/README.md` and the auth module doc comment: sessions are durable in Postgres; Redis is cache-only

## 7. Validate + cutover

- [x] 7.1 `openspec validate durable-sessions-postgres --strict` passes
- [x] 7.2 Manual smoke: sign in, restart Valkey, confirm refresh still works (durability); replay a rotated token, confirm family revoked
- [x] 7.3 Note the one-time cutover effect (active users re-login) in the deploy/PR description
