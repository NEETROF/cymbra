## 1. Store & service (shared crate)

- [ ] 1.0 Create a **dedicated crate** `backend/feature-flags` (`cymbra-feature-flags`) owning its schema/migrations + service (model of `backend/jobs`/`observability`); **not** in `backend/music`. Wire it into the server + worker; it is consumable by any app/module.
- [ ] 1.1 Migration (owned by the crate): `feature_flags` / `app_config` (key TEXT, **app** TEXT `all`/`music`/`live`/…, value_type, value, rollout_scope `global`/`staff_only`, sensitive BOOL, updated_by, updated_at, PRIMARY KEY (app, key)) + `feature_flag_changes` audit (key, app, old_value, new_value, actor, at).
- [ ] 1.2 A **code registry** of declared keys: name, **app scope** (all|specific), type, default, rollout scope, fail-direction (safe state), sensitive flag, short doc. Absent key ⇒ code default; only declared keys are editable. Keys resolve per the caller's app (token `aud`).
- [ ] 1.3 A flags/config **service** with an **OpenFeature-shaped** typed evaluation API (`bool/number/string/json(key, default, ctx)`, built in-house — not the OpenFeature SDK) and a two-tier cache: **L1** = an in-process snapshot of the whole flag/config set (hot-path reads, near-zero latency), refreshed atomically on a short TTL (~15 s safety net) or on invalidation; **L2** = **Redis pub/sub** as the invalidation bus (publish a coarse "flags changed" on edit → all server+worker instances refresh L1 within ms). Fail-safe: Redis down → L1 last snapshot + TTL-poll Postgres; Postgres down → code defaults (kill-switches safe).
- [ ] 1.4 Scope resolution per identity (global vs staff-only → admin/moderator only).

## 2. Enforcement & client read (backend)

- [ ] 2.1 Enforcement hooks: where a feature is gated, read the flag and reject the op / withhold data when off (backend-authoritative; data preserved, not deleted). Wire the actual gates as each feature is implemented.
- [ ] 2.2 A `GetEffectiveFlags` authenticated RPC returning the caller's effective flags/config (app + rollout scope + identity applied — an app sees `all` + its own keys).

## 3. Admin editing (BO, needs #3)

- [ ] 3.1 `SetFlag` / `SetConfig` admin RPCs: **platform admin** (global) may change any app's + `all` keys; **per-app admin** only that app's keys (reject `all`/other-app); validate type/range; append a `feature_flag_changes` audit entry; trigger Redis invalidation.
- [ ] 3.2 Sensitive keys (legal/infra, e.g. min public-sharing age, retention) require an explicit confirmation and are distinguished; not flippable like an ordinary toggle.
- [ ] 3.3 Vue BO panel: **app filter**; list declared keys (app, value, default, rollout scope, sensitive), toggle flags, edit typed config, show recent audit; app-aware admin gating. Built app-agnostic so a future `live` BO reuses the component.

## 4. App consumption

- [ ] 4.1 A **shared Flutter package** holding the flag client: fetch effective flags on **launch and resume** via `GetEffectiveFlags`; expose through a small Riverpod provider (service seam, fake in tests). Reusable by music, live, and future apps.
- [ ] 4.2 Feature entry points read the provider to show/hide; backend still enforces regardless.

## 5. Initial registry

- [ ] 5.1 Register the tunables as config with the straw-man defaults: `rating.review.min_votes`=5, `rating.review.threshold`=2.0; reward point bands/daily-cap/levels/shop-costs (#4); `leaderboard.global.best_n`=20, difficulty weights, `leaderboard.season.length` (#6/#7). Register on/off flags per major feature: rating, rewards/shop, profiles, per-piece boards, global board, onboarding. Mark min-age/retention as sensitive.

## 6. Tests & verification

- [ ] 6.1 Rust: default fallback (absent key + store outage), L1/L2 cache + Redis invalidation (edit effective on invalidation, TTL as backstop; Redis-down → L1 snapshot + Postgres poll; Postgres-down → code defaults), **app-scope resolution** (a `music` caller sees `all`+`music`, not `live`), rollout-scope (staff-only vs global), **admin scoping** (per-app admin can't change `all`/other-app; platform admin can), enforcement (gated op rejected when off; data preserved), audit recorded, sensitive-key confirmation. `cargo llvm-cov ... --fail-under-lines 80`.
- [ ] 6.2 Flutter: flag provider reflects fetched flags; launch/resume refresh; entry point shows/hides (via fakes). `flutter test --coverage` ≥ 80%.
- [ ] 6.3 Vue BO panel: toggle/edit + admin-gating + sensitive-key confirmation (own test setup).
- [ ] 6.4 `cargo fmt`/`clippy` + `melos run analyze`/`dart format` clean; regenerate codegen as needed.
- [ ] 6.5 `openspec validate add-runtime-feature-flags --strict` passes.
