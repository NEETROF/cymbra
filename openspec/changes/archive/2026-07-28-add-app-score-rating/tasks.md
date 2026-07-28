## 1. Rating storage (backend)

- [x] 1.1 Add a migration `backend/music/migrations/` creating `music.score_ratings (user_id UUID, catalog_score_id UUID REFERENCES catalog_scores(id) ON DELETE CASCADE, verdict TEXT CHECK (verdict IN ('dislike','like','love')), stars SMALLINT CHECK (stars BETWEEN 1 AND 5), updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), PRIMARY KEY (user_id, catalog_score_id))` + index on `catalog_score_id`. — `0009_score_ratings.sql`.
- [x] 1.2 (Optional, per design D3) add denormalized `rating_avg`, `rating_count` columns on `catalog_scores` if hub sort-by-rating is needed; otherwise compute on demand. — **Compute on demand** (D3): no denormalized column; aggregate is a grouped query indexed by `catalog_score_id`.
- [x] 1.3 Add a `needs_review` signal for the hybrid flag (a column on `catalog_scores` or a derived query) that #3 will consume; thresholds read from config. — **Derived query** (`ScoreModule::needs_review`) over the on-demand aggregate + `RatingConfig` (N/T); no column, no `moderation_status` change.

## 2. Rating RPC & aggregate (backend)

- [x] 2.1 Add a `SubmitScoreRating` RPC to the score proto (verdict + optional stars); regenerate Rust + Dart bindings. — Proto RPC added; Rust bindings regenerated via `build.rs` (compiles). Dart bindings regenerated with the app work (`melos run gen-grpc`).
- [x] 2.2 Implement the repo upsert (`ON CONFLICT (user_id, catalog_score_id) DO UPDATE`) with the swipe↔stars reconciliation (design D2); reject rating a non-`accepted`/nonexistent score. — `ScoreRatingRepo::upsert` (`FakeScoreRatingRepo` + `PgScoreRatingRepo`); `ScoreModule::submit_rating` gates on the accepted-only `object_key` path.
- [x] 2.3 Implement the per-score aggregate read (avg effective value + count + verdict breakdown), indexed by `catalog_score_id`. — `ScoreRatingRepo::aggregate` → `RatingAggregate`; SQL `AVG(CASE …)` + `COUNT(*) FILTER` mirrors `effective_value`.
- [x] 2.4 Implement the hybrid re-review flag: when `rating_count ≥ N` and `avg ≤ T` (config; defaults N=5, T=2.0 on the 1–5 scale), mark the score eligible for re-review without touching `moderation_status`. Effective value per rating: explicit stars, else verdict-implied (dislike≈1.5 / like≈3.5 / love≈5). — `is_flagged_for_review` + `ScoreModule::needs_review`, thresholds in `RatingConfig`.

## 3. Deck UI (app)

- [x] 3.1 Add a deck screen + `@riverpod` notifier + Freezed state, modeled on `CatalogSearch`/`catalog_search_notifier.dart`; source `accepted` cards via `catalogService.search`, prioritizing un-rated scores. — `RatingDeck`/`RatingDeckState` + `RatingDeckScreen`; paginates, tracks `seenIds` so re-queries skip judged cards.
- [x] 3.2 Implement the swipe interaction (vetted card-swiper package OR custom `GestureDetector`/`AnimatedBuilder`) with left=dislike / right=like / up=love, reusing `ScoreCard` for the card visual. — **Custom** `SwipeCard` (D6 fallback; zero swipe deps); `RatingCard` reuses `ScoreCard`.
- [x] 3.3 Add on-screen dislike/like/love buttons that perform the identical actions (accessibility parity), plus a Skip control that advances without recording a rating. — `RatingDeckControls` (dislike/skip/like/love), all calling the same notifier methods as the swipes.
- [x] 3.4 Add the tap-to-open 1–5 star rating, reconciled with the swipe verdict into one submitted rating. — `showRatingStarsSheet` → `RatingDeck.rateStars` derives the verdict (5→love, 3–4→like, 1–2→dislike).
- [x] 3.5 Wire a `catalogService`-style rating service seam calling `SubmitScoreRating`; optimistic update then persist (pattern from `toggleSave`). — `RatingService`/`GrpcRatingService` + `ratingServiceProvider`; optimistic advance, revert on failure.

## 4. In-card read-only preview (app)

- [x] 4.1 Add a read-only preview mode to the player render: drive the existing horizontal game-score render/playback with a flag that disables all input judging, editing, wait-mode, and scoring. — **In-card** `InCardPreview`: drives the player's `StaffPainter` (the horizontal game-score render) + the audio seam via a local ticker, read-only — no notifier, no input, no scorer.
- [x] 4.2 Add a Play control on the card that opens the read-only preview and a stop path back to the deck; keep the native render behind the existing injectable seam. — `RatingCard` Play toggles the preview over the card's cover region (title stays visible); a Stop control returns to the card. Bytes/parse go through the injectable `catalogService`/`notationEngine` seams (`cardPreviewScoreProvider`).
- [x] 4.3 Assert (test) that no scoring/performance events fire while in preview mode. — `in_card_preview_test.dart`: the preview sounds the score (audio seam receives note-ons) while `performanceScorer` stays inactive with no `lastResult`.

## 5. Discoverability

- [x] 5.1 One-time coach-mark on first deck open (persist via `shared_preferences`), plus a subtle first-card hint. — `RatingCoachMark` (tri-state, persisted) + `_CoachMarkOverlay`.

## 6. Tests & verification

- [x] 6.1 Rust: rating upsert (single row per user+score), reject non-accepted target, aggregate correctness, re-review flag thresholds (flag only at count ≥ N and avg ≤ T; not below count). `cargo llvm-cov ... --fail-under-lines 80`. — Passes at 82.85% (score_rating 95%, module 99%, grpc 92%).
- [x] 6.2 Flutter: notifier records verdict/stars via a fake rating service; buttons mirror swipes; Skip advances without recording; deck shows only accepted cards and empties correctly; preview fires no scoring. `flutter test --coverage` ≥ 80%. — 512 tests pass; aggregate 83.3% (unit/widget, before integration merge).
- [x] 6.3 `melos run analyze` + `dart format` + `cargo fmt`/`clippy` clean; regenerate codegen (`build_runner`, proto/frb) as needed. — analyze + custom_lint + dart format clean; `cargo fmt`/`clippy` clean; proto (Rust + Dart) and build_runner regenerated.
- [x] 6.4 `openspec validate add-app-score-rating --strict` passes. — "Change 'add-app-score-rating' is valid".
