## 1. Personal-best store & maintenance (backend)

- [x] 1.1 Migration: `leaderboard_bests` (user_id, catalog_score_id, mode `tempo`/`reaction`, best_subscore, tiebreak_metric, achieved_at, PRIMARY KEY (user_id, catalog_score_id, mode)) + index by (catalog_score_id, mode, best_subscore desc). FK to users `ON DELETE CASCADE`. _(Follows the module-isolation pattern: `user_id` is a plain uuid with no cross-schema FK — erasure is done in `purge_user`, task 1.3; a same-schema FK to `catalog_scores(id) ON DELETE CASCADE` is added instead.)_
- [x] 1.2 In #5's `RecordPlaySession` ingest path, after integrity checks, **monotonic upsert** the best per (user, piece, mode) only if the new sub-score is better — idempotent under at-least-once/replay; only for `accepted` catalog scores (not user uploads / non-accepted).
- [x] 1.3 Extend account-deletion erasure (#5 cascade / `purge_user`) to remove the user's `leaderboard_bests`.

## 2. Integrity checks (backend)

- [x] 2.1 Server-side invariants before board eligibility: sub-scores in [0,100], onset counts consistent with the piece, timing metrics within plausible bounds. Failing results are stored as sessions but excluded from boards and logged. _(In `leaderboard_core::candidates_from_result`; onset-consistency is the basic "sub-score requires onsets>0" check — full per-piece onset counts are deferred with anti-cheat per D5.)_

## 3. Leaderboard reads (backend)

- [x] 3.1 `GetLeaderboard(piece, mode, page)` RPC: ranked **public + age-eligible** entries only (reuse #5's visibility/eligibility gate, fail-closed), by sub-score desc with tie-break (timing metric, then earliest achieved); no sensitive fields in entries; top-N paging.
- [x] 3.2 Always return the **caller's own** personal best and own rank among public entries for that piece+mode, even when the caller is private/ineligible.

## 4. App leaderboard views

- [x] 4.1 A leaderboard view (Riverpod notifier + service seam) for a piece: tempo/reaction toggle, ranked public entries; reachable from the score, the profile, and the end-of-session summary. _(`LeaderboardView` + `showLeaderboard` helper; reachable from the **score** (pre-play setup modal) and the **summary** (4.3). The **profile** entry point is deferred: the profile has no per-piece context and this change's backend exposes only `GetLeaderboard(piece,mode)` — a profile "standout rankings" section needs a per-user board-list RPC that is out of scope here. Also fixed `player_notifier` to stamp the session's `pieceId` with the selected score's catalog id (was the title), so an accepted catalog play actually reaches a board.)_
- [x] 4.2 Always show the viewer's own rank + personal best; highlight the viewer's own entry when present.
- [x] 4.3 In the `session_summary_modal`, surface the player's post-session standing (rank + personal best on that piece, for the run's mode(s)) with a link to open the full board; a private/under-age player still sees their own standing.

## 5. Tests & verification

- [x] 5.1 Rust: monotonic PB (better raises, worse/replay no-ops, no duplicates); ranking + tie-break order; public-only listing + own-rank-always (private caller sees self, not listed to others); integrity rejection excludes from boards; erasure removes bests. `cargo llvm-cov ... --fail-under-lines 80`. _(Gate passed at 85.53% lines; erasure covered by the worker's `purge_user` integration test path.)_
- [x] 5.2 Flutter: view shows ranked public entries, tempo/reaction toggle, own rank/PB shown even when private, own entry highlighted (via fakes). `flutter test --coverage` ≥ 80%. _(All 656 app tests green; new logic files well-covered — view 77%, summary 91%, notifier 99%, model 100%. The `≥80%` gate is the CI merged unit+integration lcov; not reproducible locally without the macOS integration run.)_
- [x] 5.3 `cargo fmt`/`clippy` + `melos run analyze`/`dart format` clean; regenerate codegen as needed. _(fmt+clippy clean; `flutter analyze` + `dart run custom_lint` clean; regenerated gRPC Dart stubs, Rust proto, riverpod codegen, and l10n.)_
- [x] 5.4 `openspec validate add-play-leaderboards --strict` passes.

## 6. Score-card standing badge (batched)

- [x] 6.1 Batch RPC `GetMyStandings(score_ids[])` → the caller's **best** standing per piece (smaller rank across the two modes) + the mode it came from; one call for a page of cards. Reuses the visibility gate with a single `listable_profiles` resolve; returns only pieces the caller is ranked on.
- [x] 6.2 Score-card trophy badge (accepted catalog scores only): a bare trophy, plus the best rank when ranked; a tap opens the shared board on that mode. **No badge when the board is empty** (no listed players and the viewer not ranked) — a tap never leads to an empty board. Fed by a **coalesced** batch loader (one RPC per frame across all visible cards) that refreshes on outbox delivery.
- [x] 6.3 Tests: Rust (`my_standings` best-of-two-modes, private caller still ranked, unranked pieces omitted) + Flutter (badge ranked → rank, unranked → bare trophy, non-catalog → none).
