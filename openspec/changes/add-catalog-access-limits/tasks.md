## 1. Config & thresholds

- [x] 1.1 Add rate-limit config knobs to `backend/platform/src/config.rs` beside the existing throttle config: download burst (max + window); engagement-aware volume allowance (`base_floor`, engagement multiplier `k` = `volume_per_engagement`, `hard_ceiling`, allowance window); enumeration (max + window); and an enable/kill-switch boolean. *(Added `CatalogLimitsConfig` + `catalog_limits()` parser.)*
- [x] 1.2 Give every knob a documented non-zero default (permissive; generous for real human use) so egress is never unlimited when overrides are absent. *(burst 20/1m, floor 30, k 3, ceiling 500, window 24h, enum 60/1m, enabled=true.)*
- [ ] 1.3 Expose the thresholds and kill-switch through the runtime feature-flags / config platform (`cymbra-feature-flags`) so they can be changed without a redeploy. **DEFERRED** — env-config knobs + the `enabled` kill-switch cover the operational need now; hot-reload via the flags registry is a follow-up (declare keys in `feature-flags` registry + read on the hot path).

## 2. Wire the limiter into the score service

*(Chosen the design's option (b): a dedicated `CatalogAccessLimiter` holding the `Cache` + play port, enforced in `grpc.rs` where identity is already extracted — instead of threading `Cache` through `ScoreModule`, which owns only score-storage logic and has no identity.)*

- [x] 2.1 Add `CatalogAccessLimiter` (`backend/music/src/catalog_limits.rs`) holding `Arc<dyn Cache>` + `Arc<dyn PlayRepo>` + `CatalogLimitsConfig`; hold an optional limiter on `ScoreGrpc` via `with_limiter`.
- [x] 2.2 Construct the limiter at the composition root (`backend/server/src/main.rs`) with the always-on `cache` handle + a shared `PgPlayRepo`, and attach it to `ScoreGrpc`.

## 3. Engagement allowance input (plays + ratings)

- [x] 3.1 Compute `engagement_in_window` = in-window `PlayRepo::session_points` count **plus** in-window rating count. Added `ScoreRatingRepo::count_recent_by_user(user_id, window)` (Pg: `COUNT(*) … WHERE user_id=$1 AND updated_at >= now() - interval`; fake: counts the user's rows). Rationale: the swipe-rating deck's preview returns full bytes, so rating is genuine engagement that earns download headroom like playing.
- [x] 3.2 Cache the combined `engagement_in_window` per user in the shared cache (`catengage:{user}`) with a short TTL (60s) to avoid DB reads on every download; on cache/repo unavailability each source contributes 0 ⇒ fall back toward the `base_floor` (fail-safe).

## 4. Rate-limit enforcement in ScoreService

- [x] 4.1 Guard helper (`CatalogAccessLimiter::check_download` / `check_enumeration`) calls `cymbra_platform::ratelimit::check`, honours the kill-switch, and short-circuits (no limit) for the **back-office audience** (`id.audience == BACKOFFICE_AUDIENCE`, the curator console reuses `GetCatalogScoreBytes` per #155) and for a **music-scope admin** (`id.has_role_in_scope("music", "admin")`), applying limits normally to every other caller — music-app moderators and `live`-only admins included.
- [x] 4.2 Enforce the **burst cap** (pure rate, scope `cat_dl_burst`) in `get_catalog_score_bytes`, before any storage read; reject with `RESOURCE_EXHAUSTED` when exceeded.
- [x] 4.3 Enforce the **engagement-aware volume allowance** in `get_catalog_score_bytes` (scope `cat_dl_vol`): count vs `effective = min(hard_ceiling, base_floor + k * engagement_in_window)` where engagement = plays + ratings; reject with `RESOURCE_EXHAUSTED` on breach.
- [x] 4.4 Apply the same burst + play-aware guardrail to `get_rating_preview_bytes` (shares the download counters).
- [x] 4.5 Enforce the enumeration request-rate cap (scope `cat_enum`) in `search_catalog`, `get_catalog_score`, and `list_rating_deck`; the existing page-size clamp on `search_catalog` is untouched.
- [x] 4.6 Key every counter/allowance on `AuthIdentity.user_id` (via the existing `owner()` / `identity()` helpers) so limits are per-user and isolated.
- [x] 4.7 Redis-error posture: the play read fails safe to the floor; `ratelimit::check` runs before the module call, so a rejected request never touches the object store.

## 5. Observability

- [x] 5.1 `RESOURCE_EXHAUSTED` rejections surface per-method in the existing `ObserveLayer` RED metrics; added a targeted `tracing::warn!(user_id, limit)` on each rejection for operator alerting.

## 6. Client (Flutter, apps/music)

- [x] 6.1 `RESOURCE_EXHAUSTED` is already mapped to `AuthError.rateLimited` at the service seam; forwarded it through the download/open path — added `ScoreLoadFailure.rateLimited` (`notation_data.dart`), mapped it in `notation_notifier._classify`, and rendered `l10n.playerScoreRateLimited` in `score_load_message.dart`. Added the `playerScoreRateLimited` key to all four arbs (en/fr/es/it).
- [x] 6.2 No retry-storm risk on the score/catalog path — the channel has no retry interceptor and `authedCall` retries only on `UNAUTHENTICATED`, never `RESOURCE_EXHAUSTED`; the rate-limit failure is a terminal, user-facing state. (No change needed.)

## 7. Tests

- [x] 7.1 Rust unit tests (`catalog_limits.rs`) using `FakeCache`: burst cap rejects; engagement-aware allowance = `min(hard_ceiling, base_floor + k*engagement)`; downloads ∝ engagement never rejected; **ratings alone unlock headroom** and **plays + ratings add up**; download-heavy/engagement-light user stops at the floor; hard ceiling backstops; new user (0 engagement) allowed up to the floor; per-user isolation; exemptions — back-office audience (even a moderator) and `music/admin`/`global/admin` bypass, while `music/moderator` + `live`-only admin + regular users are enforced; kill-switch disables enforcement; defaults verified in `config.rs` tests.
- [x] 7.2 Rust handler-level test (`grpc.rs`): `get_catalog_score_bytes` / `get_rating_preview_bytes` return `RESOURCE_EXHAUSTED` on breach; `search_catalog` throttled while the page-size clamp still applies.
- [x] 7.3 Flutter tests: `notation_notifier_test` asserts an `AuthError.rateLimited` classifies as `ScoreLoadFailure.rateLimited`; `player_load_feedback_test` asserts the localized "slow down" message renders (no raw enum/gRPC text). Full non-golden suite green (640 tests).
- [x] 7.4 New code is covered by the added unit/handler/widget tests; the 80% line-coverage gate itself runs in CI (`cargo llvm-cov` + `very_good_coverage`) — not re-measured locally here.

## 8. Validation & docs

- [ ] 8.1 `openspec validate add-catalog-access-limits --strict` passes.
- [x] 8.2 Backend green: `cargo fmt --all --check`, `cargo clippy -p cymbra-music -p cymbra-platform -p cymbra-server --all-targets -- -D warnings`, tests pass.
- [x] 8.3 Flutter green: `flutter analyze` clean, `dart format` clean, `dart run custom_lint` clean, full non-golden suite passes (640).
- [x] 8.4 Documented the knobs, defaults, the play-aware formula, and the kill-switch in `backend/.env.example` (per-user catalog access limits section).
