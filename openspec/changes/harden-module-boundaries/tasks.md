## 1. Grant lock — buy the window (~15 min)

- [x] 1.1 In `backend/user/src/module.rs` `grant_role` (~:220), after `validate_scope_role`, refuse `role == "moderator"` when `scope != "music"` with an `InvalidArgument` naming the reason (guards not yet scope-matched). Do **not** touch `validate_scope_role` — it is shared with `revoke_role` (~:243).
- [x] 1.2 Test: granting `moderator` in a non-`music` scope is refused; granting `admin` in any scope still succeeds.
- [x] 1.3 Test: `revoke_role("moderator")` in a non-`music` scope still **succeeds** while the lock is active (the trap D1 names).
- [x] 1.4 **Found by the separation-of-powers audit, after the lock shipped.** `backend/scripts/seed_admin.sh` (~:19-20, `SCOPE="${2:-music}" ROLE="${3:-admin}"`, then ~:48-50 `INSERT INTO user_account.user_roles … ON CONFLICT DO NOTHING`) writes roles **directly**: it bypasses `validate_scope_role`, bypasses the lock, and writes **no `role_grants` audit row**. It needs privileged psql credentials, so it is not an application escalation — but it is the only entry point for any privileged role in production, and a review of `role_grants` would wrongly conclude nobody holds one. Make it validate the scope/role vocabulary and record the audit row, so the table is the whole truth.
- [x] 1.5 Consider a DB-level `CHECK` on `user_account.user_roles(scope, role)`, the way `role_grants` already constrains `action` (`backend/user/migrations/0004_role_grants.sql` ~:15). The lock is a Rust `if`; the table has no vocabulary constraint at all (`0001_init.sql` ~:23-28).

## 2. Account erasure — close the GDPR gap

- [x] 2.1 Read `backend/music/migrations/0013_soundfont_moderation.sql` (~:47) and `backend/music/src/user_soundfont.rs` to confirm the exact object-key column on `music.user_soundfonts`.
- [x] 2.2 Add a purge job kind for the **private soundfont bucket** in `backend/jobs/src/registry.rs` (const + spec + channel, modelled on `PURGE_SCORE_OBJECT` ~:27). Do **not** reuse `PURGE_SCORE_OBJECT` — it targets the score store (D4).
- [x] 2.3 Wire `soundfont_store` into `WorkerCtx` (`backend/worker/src/main.rs` ~:100-113, today built only for `ScorePreviewRenderer`) and add the handler in `backend/worker/src/handlers.rs`.
- [x] 2.4 In `purge_user_with` (`backend/worker/src/lib.rs`), `DELETE FROM music.user_soundfonts … RETURNING <object_key>` and enqueue one cleanup job per object **in the same transaction**, reusing the transactional-enqueue idiom (~:133-150).
- [x] 2.5 Test: erasing an account removes the rows and issues the object deletion against the **private** bucket.
- [x] 2.6 Test: a transient object-store failure retries the cleanup without leaving rows behind.
- [x] 2.7 Add the coverage test that fails when an account-keyed personal-data table is not reached by the erasure path, naming the uncovered table.
- [x] 2.8 Audit the remaining `music.*` and `plans.*` tables against the erasure path. **It found four gaps, not one.** Beyond `music.user_soundfonts`: `music.user_score_collections` and `plans.sandbox_accounts` were unreached and are now purged; `user_account.push_tokens` and `notification_prefs` turned out to be safe (`REFERENCES user_account.users ON DELETE CASCADE`); and `music.user_score_takedowns` is a **retention decision, not an oversight** — see the design's open question. The audit is now a test (`backend/worker/tests/erasure_coverage.rs`) rather than a one-off.

## 3. Scope-matched moderation guards

> Findings from the separation-of-powers audit that belong to this group, beyond the
> site list: `require_admin` is documented "in any scope — the coarse gate", so it is
> **not** scope-matched either — task 3.7 must decide each of its 5 sites, and
> `backend/feature-flags/src/grpc.rs` ~:55 is the one that gates every kill-switch, so
> scope-matching it is not enough on its own (the actor also has to reach
> `recent_changes`, and sensitive values need redacting). Removing the flat helper (3.8)
> is what makes a missed site fail to compile — do not skip it.


- [x] 3.1 Add `require_moderator_or_admin_in_scope(id, scope)` in `backend/platform/src/guard.rs`, built on `has_role_in_scope` (`backend/platform/src/identity.rs` ~:41), modelled on `require_admin_in_scope` (~:28).
- [x] 3.2 ~~Correct the `require_moderator_or_admin` doc-comment~~ — moot: 3.8 deleted the function, and its false justification with it. The reasoning is preserved in the new guard's doc, which states why the flat set answered "moderator somewhere" on a console token. Original: correct the doc-comment (`guard.rs` ~:38-44): its justification holds only for a single-scope app audience and is false for `back-office`.
- [x] 3.3 Migrate the **8 production sites** in `backend/music/src/grpc.rs` (~:795, :900, :921, :950, :1189, :1435, :1476, :1492) to the scope-matched guard. A ninth hit at ~:1783 is inside `#[cfg(test)]` (the block starts ~:1603) — an earlier anchor refresh counted it, so the total for this file is 8, not 9. Re-derive before starting, and filter out the test block: this file drifts.
- [x] 3.4 Migrate the 5 sites in `backend/server/src/soundfont.rs` (~:337, :547, :860, :933, :978).
- [x] 3.5 Migrate the 2 sites in `backend/server/src/score_preview.rs` (~:100, :169).
- [x] 3.6 Replace the 2 inline flat checks in `backend/music/src/grpc.rs` (~:380, :1450) with the scope-matched guard.
- [~] 3.7 **3 of 5 settled; 2 split out as 3.11/3.12.** Scope-matched: `music/src/soundfont_pricing.rs` (pricing a music font is music authority) and `server/src/soundfont.rs` ~:395 (auto-accepting bypasses music moderation). Left coarse **by decision**: `auth/src/grpc.rs` — the scope match happens on the *effect*, since `RevocationScope` already limits the cut to the scopes the admin governs, and narrowing the entry would break the `global` break-glass. Original: review the 5 coarse `require_admin` sites (`backend/music/src/soundfont_pricing.rs` ~:43, `backend/auth/src/grpc.rs` ~:193, `backend/feature-flags/src/grpc.rs` ~:55, `backend/server/src/soundfont.rs` ~:395, `backend/user/src/grpc.rs` ~:195): scope-match each, or record why it is legitimately cross-product (design Open Question 1).
- [x] 3.8 Remove the flat `require_moderator_or_admin` so any missed site fails to compile.
- [x] 3.9 Test: a `moderator` in another product scope on a `back-office` token is refused at a music gate; a `global/admin` still passes.
- [ ] 3.10 Remove the grant lock from task 1.1 and its test 1.2; keep 1.3 adapted. **Blocked on 3.11 AND 3.12, not on 3.1-3.9.** The lock refuses `admin`/`moderator` in any app scope but `music`, so today no `live/admin` exists to exploit those two cross-product reads — removing it while they are open is what would arm them. Ship it in its own PR: the lock is the stopgap that stops a privileged non-music role from being created at all, so deleting it in the same change that builds its replacement would leave no moment where both hold. A guard missed in the migration would silently rearm the hole.
- [ ] 3.11 **`feature-flags/src/grpc.rs` ~:203 (`list_flag_changes`) discards the actor** (`let _ = self.admin_actor(&req)?;`) and passes the caller's `app_filter` straight through, so any admin reads any product's flag change history — who changed which kill-switch, and to what. Listing *definitions* is deliberately cross-app (the console shows every app, editable per scope), so the fix is not a guard swap: either `recent_changes` takes the actor's `admin_apps` (store signature change) or a non-platform admin must name an app they administer. The back office calls it with `appFilter: ""` (`apps/back-office/src/stores/flags.ts` ~:185), so either shape changes what the console shows — decide the product behaviour first. Sensitive values may also need redacting.
- [ ] 3.12 **`user/src/grpc.rs` ~:195 (`list_role_grants`) returns every scope's grants** behind a coarse `require_admin`, so a `music/admin` reads who was made a `live/admin` and by whom. Filter the rows to the scopes the caller administers (`admin_scopes`), plus `global` for a platform admin. The back office renders this unfiltered in the users console (`apps/back-office/src/stores/roles.ts` ~:105), so confirm what an admin of one product should see in a mixed history before shrinking it.

## 4. Product-scoped plan unlocks

- [ ] 4.1 Give each `Unlock` variant an owning product in `backend/plans/src/model.rs` (~:39-52) and replace the flat `PREMIUM_UNLOCKS` block (~:68-75) with a per-product resolution.
- [ ] 4.2 Read and carry `product` through `backend/plans/src/pg.rs` (today 0 occurrences) for `plan_entitlements` and `beta_campaigns`; no backfill is needed — `DEFAULT 'music'` already makes every existing row correct.
- [ ] 4.3 Thread the product through the plan snapshot / `PlanSource` so "does the plan grant unlock X" is answered for X's product.
- [ ] 4.4 Migrate the 5 consumers, re-derived: `backend/music/src/catalog_daily_access.rs` ~:93, `backend/music/src/module.rs` **~:404** (not :357), `backend/music/src/curation_rewards_module.rs` ~:112, `backend/server/src/soundfont.rs` ~:549 and ~:886.
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

## 9. Transport failure is expressible

- [ ] 11.1 Add `Unavailable` and `DeadlineExceeded` to `AppError` (`backend/platform/src/error.rs`), with their gRPC status mappings both ways.
- [ ] 11.2 Add `From<tonic::Status> for AppError`, preserving the distinction between a domain outcome and a transport failure. `Internal` currently flattens to `"internal error"` (`error.rs` ~:58) — a remote `Internal` must not become indistinguishable from a transport fault.
- [ ] 11.3 Test: a timeout maps to `DeadlineExceeded`, an unreachable callee to `Unavailable`, and a remote not-found stays a not-found.

## 10. Stop the silent seams

> Six outbound calls discard their error by construction. That is correct for the
> domain outcome they were written for — a private profile legitimately yields no
> credit — but it also swallows a dependency failure, and nothing records it. This
> already bites today whenever the database is unhealthy; it is not split preparation.
> Visible behaviour must not change: the field stays omitted, a diagnostic appears.

- [ ] 10.1 `backend/music/src/module.rs` ~:1241 (`attach_review_attribution`, `let Ok(acct) = user.get_account`): match instead, and `warn!` on a non-domain failure.
- [ ] 10.2 `backend/music/src/module.rs` ~:1261 (`attach_public_credit`, `let Ok(p) = user.get_player_profile`): same treatment.
- [ ] 10.3 `backend/music/src/grpc.rs` ~:646 (`let Ok(profiles) = user.listable_profiles`): same treatment.
- [ ] 10.4 `backend/music/src/grpc.rs` ~:836 (`if let Ok(acct) = user.get_account`, admin SoundFont listing): same treatment.
- [ ] 10.5 `backend/music/src/leaderboard_module.rs` ~:299 (`.ok()` on `get_player_profile`): same treatment.
- [ ] 10.6 `backend/music/src/global_leaderboard_module.rs` ~:244 (`.ok()` on `get_player_profile`): same treatment.
- [ ] 10.7 Test: a dependency failure still yields a successful response with the field omitted **and** a diagnostic record; a private profile yields the omission with no record.
- [ ] 10.8 Re-derive these six anchors with grep before starting — `music/src/grpc.rs` and `module.rs` drift with every feature.

## 11. Verification

- [ ] 11.1 `cargo fmt --all --check` and `cargo clippy --workspace --all-targets -- -D warnings` clean.
- [ ] 11.2 `cargo llvm-cov --workspace --fail-under-lines 80` passes.
- [ ] 11.3 `melos run analyze` and `dart format` clean; Flutter tests and `dart run custom_lint` pass (back-office and app touched only in 4.8).
- [ ] 11.4 `openspec validate harden-module-boundaries --strict` passes.
- [ ] 11.5 Manual: delete a test account holding a private SoundFont and confirm both the row and the `.sf2` object are gone from the private bucket.
- [ ] 11.6 Manual: from a non-staff account holding a role in one product scope only, confirm the music moderation surfaces are refused.
