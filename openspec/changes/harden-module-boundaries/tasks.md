## 1. Grant lock — buy the window (~15 min)

- [ ] 1.1 In `backend/user/src/module.rs` `grant_role` (~:220), after `validate_scope_role`, refuse `role == "moderator"` when `scope != "music"` with an `InvalidArgument` naming the reason (guards not yet scope-matched). Do **not** touch `validate_scope_role` — it is shared with `revoke_role` (~:243).
- [ ] 1.2 Test: granting `moderator` in a non-`music` scope is refused; granting `admin` in any scope still succeeds.
- [ ] 1.3 Test: `revoke_role("moderator")` in a non-`music` scope still **succeeds** while the lock is active (the trap D1 names).

## 2. Account erasure — close the GDPR gap

- [ ] 2.1 Read `backend/music/migrations/0013_soundfont_moderation.sql` (~:47) and `backend/music/src/user_soundfont.rs` to confirm the exact object-key column on `music.user_soundfonts`.
- [ ] 2.2 Add a purge job kind for the **private soundfont bucket** in `backend/jobs/src/registry.rs` (const + spec + channel, modelled on `PURGE_SCORE_OBJECT` ~:27). Do **not** reuse `PURGE_SCORE_OBJECT` — it targets the score store (D4).
- [ ] 2.3 Wire `soundfont_store` into `WorkerCtx` (`backend/worker/src/main.rs` ~:100-113, today built only for `ScorePreviewRenderer`) and add the handler in `backend/worker/src/handlers.rs`.
- [ ] 2.4 In `purge_user_with` (`backend/worker/src/lib.rs`), `DELETE FROM music.user_soundfonts … RETURNING <object_key>` and enqueue one cleanup job per object **in the same transaction**, reusing the transactional-enqueue idiom (~:133-150).
- [ ] 2.5 Test: erasing an account removes the rows and issues the object deletion against the **private** bucket.
- [ ] 2.6 Test: a transient object-store failure retries the cleanup without leaving rows behind.
- [ ] 2.7 Add the coverage test that fails when an account-keyed personal-data table is not reached by the erasure path, naming the uncovered table.
- [ ] 2.8 Audit the remaining `music.*` and `plans.*` tables against the erasure path for other omissions of the same kind; fix or record what is found.

## 3. Scope-matched moderation guards

- [ ] 3.1 Add `require_moderator_or_admin_in_scope(id, scope)` in `backend/platform/src/guard.rs`, built on `has_role_in_scope` (`backend/platform/src/identity.rs` ~:41), modelled on `require_admin_in_scope` (~:28).
- [ ] 3.2 Correct the `require_moderator_or_admin` doc-comment (`guard.rs` ~:38-44): its justification holds only for a single-scope app audience and is false for `back-office`.
- [ ] 3.3 Migrate the 9 sites in `backend/music/src/grpc.rs` (~:795, :900, :921, :950, :1189, :1435, :1476, :1492, :1783) to the scope-matched guard. Re-derive with `grep -n require_moderator_or_admin` before starting — this file drifts.
- [ ] 3.4 Migrate the 5 sites in `backend/server/src/soundfont.rs` (~:337, :547, :860, :933, :978).
- [ ] 3.5 Migrate the 2 sites in `backend/server/src/score_preview.rs` (~:100, :169).
- [ ] 3.6 Replace the 2 inline flat checks in `backend/music/src/grpc.rs` (~:380, :1450) with the scope-matched guard.
- [ ] 3.7 Review the 5 coarse `require_admin` sites (`backend/music/src/soundfont_pricing.rs` ~:43, `backend/auth/src/grpc.rs` ~:193, `backend/feature-flags/src/grpc.rs` ~:55, `backend/server/src/soundfont.rs` ~:395, `backend/user/src/grpc.rs` ~:195): scope-match each, or record why it is legitimately cross-product (design Open Question 1).
- [ ] 3.8 Remove the flat `require_moderator_or_admin` so any missed site fails to compile.
- [ ] 3.9 Test: a `moderator` in another product scope on a `back-office` token is refused at a music gate; a `global/admin` still passes.
- [ ] 3.10 Remove the grant lock from task 1.1 and its test 1.2; keep 1.3 adapted.

## 4. Product-scoped plan unlocks

- [ ] 4.1 Give each `Unlock` variant an owning product in `backend/plans/src/model.rs` (~:39-52) and replace the flat `PREMIUM_UNLOCKS` block (~:68-75) with a per-product resolution.
- [ ] 4.2 Read and carry `product` through `backend/plans/src/pg.rs` (today 0 occurrences) for `plan_entitlements` and `beta_campaigns`; no backfill is needed — `DEFAULT 'music'` already makes every existing row correct.
- [ ] 4.3 Thread the product through the plan snapshot / `PlanSource` so "does the plan grant unlock X" is answered for X's product.
- [ ] 4.4 Migrate the 5 consumers: `backend/music/src/catalog_daily_access.rs` ~:93, `backend/music/src/module.rs` ~:357, `backend/music/src/curation_rewards_module.rs` ~:112, `backend/server/src/soundfont.rs` ~:549 and ~:886.
- [ ] 4.5 Test: an account with an active non-music `premium` and no music entitlement is denied every music unlock.
- [ ] 4.6 Test: an account holding entitlements for two products gets each product's unlocks independently.
- [ ] 4.7 Test: an entitlement written before products were distinguished still grants the full music unlock set.
- [ ] 4.8 **Last**: unblock back-office plan visibility. The `/roles` screen was split into `/users` + `/users/{id}` (2026-09-06), so the `adminScopes.includes("music")` gate now sits at **4 sites**: `apps/back-office/src/stores/roles.ts` ~:54, `views/UsersView.vue` ~:32, `views/UserDetailView.vue` ~:61, `App.vue` ~:62 (nav).

## 5. Empty the composition root

- [ ] 5.1 Add `axum`, `tower_http` and `jsonwebtoken` to `backend/music/Cargo.toml` (none is declared today) — the assumed consequence of D5.
- [ ] 5.2 Move `backend/server/src/soundfont.rs` (~2 679 l.) and `score_preview.rs` (~385 l.) into `backend/music`, exposing an `axum::Router` instead of free handlers.
- [ ] 5.3 Move the four music backfill binaries from `backend/server/src/bin/` into `backend/music`.
- [ ] 5.4 Reduce `backend/server/src/main.rs` to mounting the music router; remove the ~137 lines of music wiring.
- [ ] 5.5 Check the coverage ignore regexes (`.github/workflows/rust.yml`, `sonar.yml`) for paths naming the moved `server/` files; update the moved paths only, without anchoring or splitting the regex.
- [ ] 5.6 Verify no behaviour change: routes, auth and responses identical before/after.

## 6. External contract net

- [ ] 6.1 Add a `buf` configuration and a CI job running `buf breaking` over the ten `backend/*/proto/*.proto`.
- [ ] 6.2 Decide and document the baseline: previous commit, or the last shipped `music-v*` tag (design Open Question 2).
- [ ] 6.3 Document how an intentional break is recorded in a change, and note the measured expectation (~1 override per 8–9 proto commits).
- [ ] 6.4 Verify the gate fails on a removed RPC and on a renumbered field, and passes on an added field.

## 7. Design record and documented decisions

- [ ] 7.1 Delete `GrpcUserClient` (`backend/user-port/src/lib.rs` ~:352-377) and fix the doc-comment at ~:146-147 that presents it as an implementor.
- [ ] 7.2 Fix `backend/auth-port/src/lib.rs` ~:1-5, which claims to carry a gRPC client adapter that does not exist.
- [ ] 7.3 **After 7.1**, add `.build_client(false)` to the seven `build.rs`; `auth-port/build.rs` and `user-port/build.rs` use the `compile_protos` shorthand and need `configure()` first.
- [ ] 7.4 Amend `openspec/changes/archive/2026-06-27-add-cymbra-id/design.md` (~:170-180): remove the 1:1 port↔gRPC-service rule, the "client adapter implementing the port trait", and the shared contract test.
- [ ] 7.5 Name the worker among the permitted ops actors in `backend/db/init/roles.sql.tpl` (~:118-119), and state that module boundaries inside the worker are not database-enforced.
- [ ] 7.6 Record the three-object boundary rule (internal boundary / internal transport / external contract) in `CLAUDE.md`, with the reference patterns `backend/plans/src/ports.rs` and `backend/feature-flags/src/context.rs`.

## 8. One implementation of role resolution

> Not a privilege defect: `PgAdminScopeResolver` runs on the **user pool** as `user_svc`
> (`backend/server/src/main.rs` ~:56 `flags_resolver_pool = user_pool.clone()`), whose
> `search_path` is `user_account` (`backend/db/init/roles.sql.tpl` ~:32), so its unqualified
> `FROM user_roles` reads a table that role owns. `flags_svc` never touches `user_account`.
> The defect is **duplication**: a second implementation of a role-resolution rule the user
> module already owns, which will diverge silently the day that rule gains a nuance.

- [ ] 8.1 Replace the SQL in `PgAdminScopeResolver` (`backend/server/src/flags.rs` ~:34-48) with `UserPort::scoped_effective_roles(user_id, &["global"])`, checking for `admin`. The composition root already holds `Arc<dyn UserPort>` (`backend/server/src/main.rs` ~:80).
- [ ] 8.2 Change `build_flag_service` (`backend/server/src/flags.rs` ~:53) to take `Arc<dyn UserPort>` instead of a `PgPool`, and drop `flags_resolver_pool` (`backend/server/src/main.rs` ~:56, `:162`).
- [ ] 8.3 Test: an account holding `global/admin` resolves as platform admin; one holding `music/admin` only does not. Double the port with `MockUserPort` rather than a database — the point of the change is that no database is needed here.
- [ ] 8.4 Record the two direct cross-schema reads as **named exceptions**, not silent ones, where the ops role is defined (alongside task 7.5):
  - `backend/music/src/pg_streak.rs:194` (`LEFT JOIN user_account.users`, two columns) — **assumed exception**: worker path only, on the ops connection, and the worker is an ops actor by decision. State that it is not reachable from the request path.
  - `backend/notifications/src/pg.rs` (7 statements on `user_account.*`, including `UPDATE user_account.users SET timezone`) — **named debt, deliberately deferred**: a schema-ownership problem, not a request-path leak — `notifications` has no schema or migrations of its own, its tables being created by `backend/user/migrations/0008_push_notifications.sql`. Give it its own change rather than folding it in here.
- [ ] 8.5 Verify no other request-path read of another module's schema remains: grep the module crates for another module's schema qualifier and confirm every hit is either fixed above or listed as an exception.

## 9. Verification

- [ ] 9.1 `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` clean.
- [ ] 9.2 `cargo llvm-cov --workspace --fail-under-lines 80` passes.
- [ ] 9.3 `melos run analyze` and `dart format` clean; Flutter tests and `dart run custom_lint` pass (back-office and app touched only in 4.8).
- [ ] 9.4 `openspec validate harden-module-boundaries --strict` passes.
- [ ] 9.5 Manual: delete a test account holding a private SoundFont and confirm both the row and the `.sf2` object are gone from the private bucket.
- [ ] 9.6 Manual: from a non-staff account holding a role in one product scope only, confirm the music moderation surfaces are refused.
