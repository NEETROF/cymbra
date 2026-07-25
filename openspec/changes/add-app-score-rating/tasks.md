## 1. Rating storage (backend)

- [ ] 1.1 Add a migration `backend/music/migrations/` creating `music.score_ratings (user_id UUID, catalog_score_id UUID REFERENCES catalog_scores(id) ON DELETE CASCADE, verdict TEXT CHECK (verdict IN ('pass','like','love')), stars SMALLINT CHECK (stars BETWEEN 1 AND 5), updated_at TIMESTAMPTZ NOT NULL DEFAULT now(), PRIMARY KEY (user_id, catalog_score_id))` + index on `catalog_score_id`.
- [ ] 1.2 (Optional, per design D3) add denormalized `rating_avg`, `rating_count` columns on `catalog_scores` if hub sort-by-rating is needed; otherwise compute on demand.
- [ ] 1.3 Add a `needs_review` signal for the hybrid flag (a column on `catalog_scores` or a derived query) that #3 will consume; thresholds read from config.

## 2. Rating RPC & aggregate (backend)

- [ ] 2.1 Add a `SubmitScoreRating` RPC to the score proto (verdict + optional stars); regenerate Rust + Dart bindings.
- [ ] 2.2 Implement the repo upsert (`ON CONFLICT (user_id, catalog_score_id) DO UPDATE`) with the swipe↔stars reconciliation (design D2); reject rating a non-`accepted`/nonexistent score.
- [ ] 2.3 Implement the per-score aggregate read (avg effective value + count + verdict breakdown), indexed by `catalog_score_id`.
- [ ] 2.4 Implement the hybrid re-review flag: when `rating_count ≥ N` and `avg ≤ T` (config), mark the score eligible for re-review without touching `moderation_status`.

## 3. Deck UI (app)

- [ ] 3.1 Add a deck screen + `@riverpod` notifier + Freezed state, modeled on `CatalogSearch`/`catalog_search_notifier.dart`; source `accepted` cards via `catalogService.search`, prioritizing un-rated scores.
- [ ] 3.2 Implement the swipe interaction (vetted card-swiper package OR custom `GestureDetector`/`AnimatedBuilder`) with left=pass / right=like / up=love, reusing `ScoreCard` for the card visual.
- [ ] 3.3 Add on-screen pass/like/love buttons that perform the identical actions (accessibility parity).
- [ ] 3.4 Add the tap-to-open 1–5 star rating, reconciled with the swipe verdict into one submitted rating.
- [ ] 3.5 Wire a `catalogService`-style rating service seam calling `SubmitScoreRating`; optimistic update then persist (pattern from `toggleSave`).

## 4. In-card read-only preview (app)

- [ ] 4.1 Add a read-only preview mode to the player render: drive the existing horizontal game-score render/playback with a flag that disables all input judging, editing, wait-mode, and scoring.
- [ ] 4.2 Add a Play control on the card that opens the read-only preview and a stop path back to the deck; keep the native render behind the existing injectable seam.
- [ ] 4.3 Assert (test) that no scoring/performance events fire while in preview mode.

## 5. Discoverability

- [ ] 5.1 One-time coach-mark on first deck open (persist via `shared_preferences`), plus a subtle first-card hint.

## 6. Tests & verification

- [ ] 6.1 Rust: rating upsert (single row per user+score), reject non-accepted target, aggregate correctness, re-review flag thresholds (flag only at count ≥ N and avg ≤ T; not below count). `cargo llvm-cov ... --fail-under-lines 80`.
- [ ] 6.2 Flutter: notifier records verdict/stars via a fake rating service; buttons mirror swipes; deck shows only accepted cards and empties correctly; preview fires no scoring. `flutter test --coverage` ≥ 80%.
- [ ] 6.3 `melos run analyze` + `dart format` + `cargo fmt`/`clippy` clean; regenerate codegen (`build_runner`, proto/frb) as needed.
- [ ] 6.4 `openspec validate add-app-score-rating --strict` passes.
