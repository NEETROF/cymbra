## 1. Config, seams & storage (backend)

- [x] 1.1 Declare the flag keys in `backend/feature-flags/src/registry.rs` (app `music`): `catalog.daily_access.enabled` (bool, default false), `catalog.daily_access.free_quota` (int, 3), `catalog.daily_access.day_slot_cost` (int, 20), `catalog.preview.max_ms` (int, 30000), `catalog.preview.soundfont_id` (string, ""). Safe defaults, doc strings, staff-only rollout available.
- [x] 1.2 Add music-crate traits `DailyAccessConfigSource` (enabled/quota/cost read at call time) and `SubscriptionSource` (`has_active_subscription(user_id) -> bool`, stub `false`, documented for billing); implement both in `backend/server/src/flags.rs` over the flag service (mirror `FlagStreakConfig`) and wire them into `ScoreModule`.
- [x] 1.3 Migration: `music.catalog_day_access (user_id, catalog_id FK ON DELETE CASCADE, day DATE, paid BOOL, opened_at, PK (user_id, catalog_id, day))` + index `(user_id, day)`; `ALTER music.catalog_scores ADD COLUMN preview_rendered_at TIMESTAMPTZ`.
- [x] 1.4 Repo port + Pg impl: `day_state(user_id, day)`, `record_open(user_id, catalog_id, day, paid=false)` (idempotent upsert, never downgrades a paid row), `opened_today(user_id, day)`, and the atomic `spend_day_slot(user_id, catalog_id, day, cost, award_key) -> bool` transaction (advisory xact lock per user → re-read `SUM(amount)` → `INSERT curation_points (redeem, -cost, reward_key='score_day_slot', award_key, piece_id) ON CONFLICT (user_id, award_key) … DO NOTHING` → upsert paid row) shaped like `PgStreakRepo::spend_and_restore`.

## 2. Daily-access core & gate (backend)

- [x] 2.1 Create the host-testable `catalog_daily_access_core`: `decide_open(piece, &DayState, &DailyAccessConfig, CallerKind) -> Open {Serve|ServeFree|Locked{cost, upsell}}` + `day_slot_decision(already_open, balance, cost, enabled)`; unit tests for every branch incl. gate-off, re-open-is-free, quota 0, cost 0, exempt/subscriber/contributor, next-day recharge.
- [x] 2.2 Wire the decision into `ScoreModule::catalog_bytes_for_player` **before** the conditional-fetch short-circuit and **before** `record_engagement`; caller kind from `AuthIdentity` (back-office audience / music-scope moderator-admin → Exempt), `proposed_by == caller` → Contributor, `SubscriptionSource` → Subscriber. On Serve/ServeFree upsert the day row; on Locked return the state with empty data, `unchanged=false`, no engagement.
- [x] 2.3 Proto: `CatalogAccessState` message; `GetCatalogScoreBytesResponse.access`; `GetCatalogDailyAccess` RPC (state + `opened_today` + `paid_today`); `UnlockCatalogScoreForToday` RPC; `CatalogHit.has_preview`; admin catalog search `has_preview` filter param. Regenerate stubs (`melos run gen-grpc` — app **and** `packages/cymbra_flags`; BO `yarn gen`).
- [x] 2.4 Handler `UnlockCatalogScoreForToday`: identity → decision core → `spend_day_slot` → return state; insufficient balance = `FAILED_PRECONDITION` (nothing written). Keep `guard_download` order (abuse cap first) on the bytes RPC.
- [x] 2.5 Handler `GetCatalogDailyAccess`: enumeration guard, then state (quota/used/reset instant/cost/spendable balance/subscriber/upsell + today's ids).
- [x] 2.6 Verify `ListRatingDeck` / `GetRatingPreviewBytes` (rater path), `GetCatalogScore` / `SearchCatalog` (metadata) and `GetScoreBytes` (own uploads) stay ungated; grpc tests: locked state shape, conditional fetch of a locked piece is locked (not unchanged), moderator/BO exempt, contributor free, gate-off serves.
- [x] 2.7 Data lifecycle: `purge_user` job deletes the user's `catalog_day_access` rows; add a 30-day prune of `catalog_day_access` to the existing prune schedule (test both).

## 3. Score audio preview pipeline (backend + worker)

- [x] 3.1 Pure helper `preview_sequence(&PlaybackSchedule, max_ms) -> SampleSequence` (from `cymbra_musicxml_core::playback::schedule`; clip + truncate held notes; deterministic) + `score_preview_object_key(catalog_id)` (`catalog-preview/{id}.wav`); unit-test bounding, determinism, empty score, encoder validity.
- [x] 3.2 Job spec `SCORE_PREVIEW_RENDER` (`Channel::parallel("music","preview")`, retry policy) in `backend/jobs/src/registry.rs`; worker handler in `backend/worker/src/handlers.rs`: load MusicXML (score store) → parse → schedule → sequence → font bytes from the SoundFont store (`catalog.preview.soundfont_id`, must be accepted) → `render_preview_pcm` + `encode_preview` → `put` → stamp `preview_rendered_at`. Unset font → log + no-op (dormant).
- [x] 3.3 Enqueue on accept: `SetModerationStatus(accepted)` uses `transactional_enqueue` inside the status-write transaction (Pg adapter, one tx — best-effort: an enqueue failure logs and the accept still commits; no Pg-backed test infra in this repo, so atomicity is by construction + the manual pass in 6.4).
- [x] 3.4 HTTP routes in `backend/server` (mirror `soundfont.rs` `serve_preview`/`regenerate_preview`): `GET /scores/{id}/preview` (authenticated, moderation-visible, no quota gate, 404 when absent) and `POST /scores/{id}/preview` (admin/moderator, inline render, overwrite + stamp). Route tests for visibility split, 404, non-privileged refusal.
- [x] 3.5 Backfill: `--enqueue-missing-previews` mode of the maintenance binary (or scheduler entry) enqueuing `SCORE_PREVIEW_RENDER` for accepted rows with `preview_rendered_at IS NULL`; idempotent re-run.
- [x] 3.6 Listing: `CatalogHit.has_preview` from `preview_rendered_at` (public + admin reads); admin search filter `has_preview` (`""|"yes"|"no"`).

## 4. App (Flutter)

- [x] 4.1 `CatalogService`: `fetchScoreBytes` result carries `access` (Freezed `CatalogAccessState`); add `dailyAccess()` and `unlockForToday(id)`; `CatalogHit.hasPreview`.
- [x] 4.2 `ScorePreviewService` (twin of `SoundFontPreviewService`: `GET /scores/{id}/preview` with bearer, 404 → false) behind a provider; playback via `soundClipPlayerProvider`.
- [x] 4.3 `catalogDailyAccessProvider` (keepAlive, identity-scoped; refreshed on served open, unlock, app resume, auth change) + `catalogUnlockProvider` notifier (`unlock(id)` fire-and-observe, `AsyncValue` state, monotonic `seq`).
- [x] 4.4 Notation load: on `access.locked` expose `ScoreLoadFailure.locked` (no error snackbar, no engagement); **online cache-hit path performs the conditional fetch first** and treats locked as "do not play from cache"; offline keeps playing from cache.
- [x] 4.5 Unlock sheet from `openScore`: title/composer, "écouter un extrait" (audition; greyed when `!hasPreview`), "débloquer pour X pts aujourd'hui" (disabled with shortfall when balance < cost), upsell placeholder line reusing the shop's premium "coming later" copy; confirm → `unlock(id)`.
- [x] 4.6 Dedicated listener widget: unlock success → re-select the score (served) + bump `rewardRevisionProvider`; failure → localized snackbar; never await-and-branch in the UI.
- [x] 4.7 Surfaces: hub/library header chip "N ouvertures gratuites · réinit. dans Xh"; opened-today mark on `ScoreCard` (cover overlay — bottom-left slot is taken by offline/status tags); locked cost hint on cards.
- [x] 4.8 Activity feed: label the `score_day_slot` redeem with the piece title (no raw kind/reward key in UI).
- [x] 4.9 Analytics actions in `usage_actions.dart`: `catalog_quota_reached`, `catalog_day_slot_unlock`, `catalog_preview_audition`.
- [x] 4.10 l10n en/fr/it/es for every new string.
- [x] 4.11 Tests (mockito mocks via provider overrides): quota chip; locked open shows the sheet and never fetches/plays MusicXML; audition plays the clip / greyed without preview; unlock spends and re-opens; re-open same day free; online cached favourite locked does not play, offline plays; gate-off path unchanged.

## 5. Back office (Vue)

- [x] 5.1 `regenerateScorePreview(id)` injectable transport (`POST /scores/{id}/preview`, `setRegenerateScorePreviewForTest`) in the catalog store with a dedicated `Async<void>` `preview` ref + optimistic `hasPreview`.
- [x] 5.2 "Generate sample" action in the score detail / table row (play when `hasPreview`, generate otherwise — SoundFonts pattern), state via `match(...).exhaustive()`; toasts.
- [x] 5.3 `hasPreview` filter on `FiltersBar` (`""|"yes"|"no"`) → admin search param; e2e fake seam entries.
- [x] 5.4 Vitest (success/error/filter) + Playwright spec through the seam; en/fr strings.

## 6. Coverage, gates & verification

- [ ] 6.1 Rust: `cargo fmt` + `clippy -D warnings`; `cargo llvm-cov --workspace --fail-under-lines 80` (decision + day-slot + sequence cores and seams covered; gRPC/route/synth/worker glue excluded).
- [ ] 6.2 Flutter: `dart run build_runner build`; `melos run analyze` + `dart run custom_lint`; `flutter test --coverage` ≥ 80%.
- [x] 6.3 Back office: `yarn lint` + `yarn test` + e2e green.
- [ ] 6.4 Manual (staff-only rollout, real backend): exhaust the quota → (N+1)th piece locked, clip plays, MusicXML refused; spend points → opens + re-open free; cached favourite locked online / plays offline; server-day rollover recharges; accept a piece → job renders a preview; BO "Generate sample" + "no sample" filter; backfill enqueues the corpus.
- [ ] 6.5 `openspec validate add-score-daily-access-rewards --strict` passes.
