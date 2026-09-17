# Tasks — fix-interrupted-refresh-signouts

## 1. Auth module

- [ ] 1.1 Migration: `sessions.prev_rt_hash` + `sessions.prev_replaced_at` (nullable, no backfill)
- [ ] 1.2 `CYMBRA_REFRESH_REUSE_GRACE` (default 60s) in the platform config, threaded to the session store
- [ ] 1.3 `rotate`: current token → rotate and record the replaced one; immediately-previous token within the grace → rotate again and return a pair; anything else → reuse detection unchanged. One conditional `UPDATE … RETURNING` per branch, so concurrency stays atomic
- [ ] 1.4 The in-memory fake follows the same three-way resolution (the module's tests run on it)
- [ ] 1.5 Unit tests: rotation records the replaced token; a replay inside the grace returns a pair and keeps the family; a replay outside it revokes; a token two generations back revokes; concurrent refreshes both end usable
- [ ] 1.6 `backend/auth/tests/auth_flow.rs` (live Postgres, `--ignored`): the interrupted-client sequence end to end

## 2. Lingua extension

- [ ] 2.1 Keep the access token with its expiry where a suspended background page finds it again, and refresh only when it is absent or about to expire (the single-flight refresh stays)
- [ ] 2.2 vitest: a wake with a live access token sends no refresh; an expired one refreshes once; the stored expiry survives a simulated restart

## 3. Gates

- [ ] 3.1 `cargo test -p cymbra-auth`, `cargo fmt --check`, `clippy -D warnings`; extension `typecheck`/`lint`/`test`/`build`
- [ ] 3.2 `openspec validate fix-interrupted-refresh-signouts --strict`
- [ ] 3.3 Deploy the backend before shipping the extension build (the grace is inert for clients that never replay)
