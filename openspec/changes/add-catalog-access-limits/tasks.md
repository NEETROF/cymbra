## 1. Config & thresholds

- [ ] 1.1 Add rate-limit config knobs to `backend/platform/src/config.rs` beside the existing throttle config: download burst (max + window); play-aware volume allowance (`base_floor`, play multiplier `k`, `hard_ceiling`, allowance window); enumeration (max + window); and an enable/kill-switch boolean.
- [ ] 1.2 Give every knob a documented non-zero default (permissive; generous for real human use) so egress is never unlimited when overrides are absent.
- [ ] 1.3 Expose the thresholds and kill-switch through the runtime feature-flags / config platform (`cymbra-feature-flags`) so they can be changed without a redeploy.

## 2. Wire the cache into ScoreModule

- [ ] 2.1 Add `Arc<dyn Cache>` to `ScoreModule::new` (`backend/music/src/module.rs`) and store it on the struct.
- [ ] 2.2 Pass the always-on `cache` handle at the construction site in `backend/server/src/main.rs` (the current wiring gap).

## 3. Play-aware allowance input

- [ ] 3.1 Add a way to read `plays_in_window` per user from `PlayService` session data (decide the signal: distinct scores played vs session count vs play time — see design Open Questions).
- [ ] 3.2 Cache `plays_in_window` per user in Redis with a short TTL to avoid a `PlayService` round-trip on every download; on cache/PlayService unavailability, fall back to the `base_floor` allowance (fail-safe).

## 4. Rate-limit enforcement in ScoreService

- [ ] 4.1 Add a small guard helper (in `backend/music/src/grpc.rs` or a `module.rs` method) that calls `cymbra_platform::ratelimit::check` for a given scope/subject and honours the kill-switch; it SHALL short-circuit (no limit) when `id.has_role_in_scope("music", "admin")` (music/admin or global break-glass), and apply limits normally to every other caller — including moderators and admins scoped only to another domain (e.g. `live`).
- [ ] 4.2 Enforce the **burst cap** (pure rate, scope `cat_dl_burst`) in `get_catalog_score_bytes`, before any storage read; reject with `RESOURCE_EXHAUSTED` when exceeded.
- [ ] 4.3 Enforce the **play-aware volume allowance** in `get_catalog_score_bytes`: compare the user's rolling download count against `effective = min(hard_ceiling, base_floor + k * plays_in_window)`; reject with `RESOURCE_EXHAUSTED` when the count would exceed `effective`.
- [ ] 4.4 Apply the same burst + play-aware guardrail to `get_rating_preview_bytes` (shares the download counters).
- [ ] 4.5 Enforce the enumeration request-rate cap in `search_catalog`, `get_catalog_score`, and `list_rating_deck`; keep the existing page-size clamp on `search_catalog`.
- [ ] 4.6 Key every counter/allowance on `AuthIdentity.user_id` (via the existing `owner()` / `identity()` helpers) so limits are per-user and isolated.
- [ ] 4.7 Redis-error posture (fail-open: serve + log on cache error, per design); ensure a rejected request never touches the object store.

## 5. Observability

- [ ] 5.1 Confirm `RESOURCE_EXHAUSTED` rejections surface per-method in the existing `ObserveLayer` RED metrics; add a targeted counter/log for catalog rate-limit rejections if method+status labels are insufficient for alerting.

## 6. Client (Flutter, apps/music)

- [ ] 6.1 Map `RESOURCE_EXHAUSTED` from the score gRPC client to a localized, non-technical "slow down / limit reached" message (per the no-raw-technical-errors-in-UI rule); add the strings to the l10n arb files.
- [ ] 6.2 Ensure the client does not retry-storm on `RESOURCE_EXHAUSTED` — treat it as a terminal, user-facing state, not an auto-retried error.

## 7. Tests

- [ ] 7.1 Rust unit tests for the guard helper using `FakeCache`: burst cap rejects; play-aware allowance = `min(hard_ceiling, base_floor + k*plays)`; ratio-healthy user (downloads ∝ plays) is never rejected; download-heavy/play-light user stops at the floor; hard ceiling backstops a high play count; new user (0 plays) allowed up to the floor; per-user isolation; scope-matched bypass — `music/admin` and `global/admin` bypass all limits, while `music/moderator`, a `live`-only admin, and regular users are still enforced; kill-switch disables enforcement; defaults applied when unconfigured; play-data-unavailable falls back to the floor.
- [ ] 7.2 Rust handler-level tests: `get_catalog_score_bytes` / `get_rating_preview_bytes` return `RESOURCE_EXHAUSTED` on breach and do not read storage; `search_catalog` / `list_rating_deck` throttled while page-size clamp still applies.
- [ ] 7.3 Flutter test: score client maps `RESOURCE_EXHAUSTED` to the localized message and does not auto-retry.
- [ ] 7.4 Keep coverage ≥ 80% for both ecosystems (Rust `cargo llvm-cov`, Flutter `flutter test --coverage`).

## 8. Validation & docs

- [ ] 8.1 `openspec validate add-catalog-access-limits --strict` passes.
- [ ] 8.2 Backend green: `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, tests pass.
- [ ] 8.3 Flutter green: `melos run analyze`, `dart format`, `dart run custom_lint`, tests pass.
- [ ] 8.4 Document the default thresholds (floor/`k`/ceiling/windows), the play signal used, and the runtime kill-switch (how operators tune/disable) alongside the config.
