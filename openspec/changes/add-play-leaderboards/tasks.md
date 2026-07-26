## 1. Personal-best store & maintenance (backend)

- [ ] 1.1 Migration: `leaderboard_bests` (user_id, catalog_score_id, mode `tempo`/`reaction`, best_subscore, tiebreak_metric, achieved_at, PRIMARY KEY (user_id, catalog_score_id, mode)) + index by (catalog_score_id, mode, best_subscore desc). FK to users `ON DELETE CASCADE`.
- [ ] 1.2 In #5's `RecordPlaySession` ingest path, after integrity checks, **monotonic upsert** the best per (user, piece, mode) only if the new sub-score is better — idempotent under at-least-once/replay; only for `accepted` catalog scores (not user uploads / non-accepted).
- [ ] 1.3 Extend account-deletion erasure (#5 cascade / `purge_user`) to remove the user's `leaderboard_bests`.

## 2. Integrity checks (backend)

- [ ] 2.1 Server-side invariants before board eligibility: sub-scores in [0,100], onset counts consistent with the piece, timing metrics within plausible bounds. Failing results are stored as sessions but excluded from boards and logged.

## 3. Leaderboard reads (backend)

- [ ] 3.1 `GetLeaderboard(piece, mode, page)` RPC: ranked **public + age-eligible** entries only (reuse #5's visibility/eligibility gate, fail-closed), by sub-score desc with tie-break (timing metric, then earliest achieved); no sensitive fields in entries; top-N paging.
- [ ] 3.2 Always return the **caller's own** personal best and own rank among public entries for that piece+mode, even when the caller is private/ineligible.

## 4. App leaderboard views

- [ ] 4.1 A leaderboard view (Riverpod notifier + service seam) for a piece: tempo/reaction toggle, ranked public entries; reachable from the score, the profile, and the end-of-session summary.
- [ ] 4.2 Always show the viewer's own rank + personal best; highlight the viewer's own entry when present.
- [ ] 4.3 In the `session_summary_modal`, surface the player's post-session standing (rank + personal best on that piece, for the run's mode(s)) with a link to open the full board; a private/under-age player still sees their own standing.

## 5. Tests & verification

- [ ] 5.1 Rust: monotonic PB (better raises, worse/replay no-ops, no duplicates); ranking + tie-break order; public-only listing + own-rank-always (private caller sees self, not listed to others); integrity rejection excludes from boards; erasure removes bests. `cargo llvm-cov ... --fail-under-lines 80`.
- [ ] 5.2 Flutter: view shows ranked public entries, tempo/reaction toggle, own rank/PB shown even when private, own entry highlighted (via fakes). `flutter test --coverage` ≥ 80%.
- [ ] 5.3 `cargo fmt`/`clippy` + `melos run analyze`/`dart format` clean; regenerate codegen as needed.
- [ ] 5.4 `openspec validate add-play-leaderboards --strict` passes.
