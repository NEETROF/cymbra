## 1. Store & service (backend)

- [ ] 1.1 Migration: `feature_flags` / `app_config` (key TEXT PK, value_type, value, scope `global`/`staff_only`, sensitive BOOL, updated_by, updated_at) + `feature_flag_changes` audit (key, old_value, new_value, actor, at).
- [ ] 1.2 A **code registry** of declared keys: name, type, default, scope, fail-direction (safe state), sensitive flag, short doc. Absent key ⇒ code default; only declared keys are editable.
- [ ] 1.3 A flags/config **service** with typed accessors and a two-tier cache: **L1** = an in-process snapshot of the whole flag/config set (hot-path reads, near-zero latency), refreshed atomically on a short TTL (~15 s safety net) or on invalidation; **L2** = **Redis pub/sub** as the invalidation bus so all server+worker instances refresh L1 within ms on an admin edit. Read path L1→Postgres; write path Postgres→publish→refresh L1. Fail-safe: Redis down → L1 last snapshot + TTL-poll Postgres; Postgres down → code defaults (kill-switches to safe state). Worker instances share the same service + subscription.
- [ ] 1.4 Scope resolution per identity (global vs staff-only → admin/moderator only).

## 2. Enforcement & client read (backend)

- [ ] 2.1 Enforcement hooks: where a feature is gated, read the flag and reject the op / withhold data when off (backend-authoritative; data preserved, not deleted). Wire the actual gates as each feature is implemented.
- [ ] 2.2 A `GetEffectiveFlags` authenticated RPC returning the caller's effective flags/config (scope + identity applied).

## 3. Admin editing (BO, needs #3)

- [ ] 3.1 `SetFlag` / `SetConfig` admin RPCs guarded by `require_admin`; validate type/range; append a `feature_flag_changes` audit entry; trigger Redis invalidation.
- [ ] 3.2 Sensitive keys (legal/infra, e.g. min public-sharing age, retention) require an explicit confirmation and are distinguished; not flippable like an ordinary toggle.
- [ ] 3.3 Vue BO panel: list declared keys (value, default, scope, sensitive), toggle flags, edit typed config, show recent audit; admin-only.

## 4. App consumption

- [ ] 4.1 Fetch effective flags on **launch and resume** via `GetEffectiveFlags`; expose through a small Riverpod provider (service seam, fake in tests).
- [ ] 4.2 Feature entry points read the provider to show/hide; backend still enforces regardless.

## 5. Initial registry

- [ ] 5.1 Register the tunables as config with the straw-man defaults: `rating.review.min_votes`=5, `rating.review.threshold`=2.0; reward point bands/daily-cap/levels/shop-costs (#4); `leaderboard.global.best_n`=20, difficulty weights, `leaderboard.season.length` (#6/#7). Register on/off flags per major feature: rating, rewards/shop, profiles, per-piece boards, global board, onboarding. Mark min-age/retention as sensitive.

## 6. Tests & verification

- [ ] 6.1 Rust: default fallback (absent key + store outage), L1/L2 cache + Redis invalidation (edit effective on invalidation, TTL as backstop; Redis-down → L1 snapshot + Postgres poll; Postgres-down → code defaults), scope resolution (staff-only vs global), enforcement (gated op rejected when off; data preserved), audit recorded, sensitive-key confirmation. `cargo llvm-cov ... --fail-under-lines 80`.
- [ ] 6.2 Flutter: flag provider reflects fetched flags; launch/resume refresh; entry point shows/hides (via fakes). `flutter test --coverage` ≥ 80%.
- [ ] 6.3 Vue BO panel: toggle/edit + admin-gating + sensitive-key confirmation (own test setup).
- [ ] 6.4 `cargo fmt`/`clippy` + `melos run analyze`/`dart format` clean; regenerate codegen as needed.
- [ ] 6.5 `openspec validate add-runtime-feature-flags --strict` passes.
