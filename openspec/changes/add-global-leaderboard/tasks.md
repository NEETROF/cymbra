## 1. Season-best store & maintenance (backend)

- [x] 1.1 Migration: `global_season_bests` (user_id, season_id, catalog_score_id, mode `tempo`/`reaction`, best_subscore, achieved_at, PRIMARY KEY (user_id, season_id, catalog_score_id, mode)) + index by (season_id, mode). FK to users `ON DELETE CASCADE`.
- [x] 1.2 In the #6 ingest hook, monotonic upsert the current-season best per (user, season, piece, mode) only if the new sub-score is better (idempotent under at-least-once); accepted catalog scores only; respects the #6 integrity checks.
- [x] 1.3 Difficulty weight from catalog `level` (config map: e.g. beginner 1.0 / intermediate 1.5 / advanced 2.0 / unleveled 1.0); config best-N and season length.

## 2. Seasons (backend)

- [x] 2.1 Season model: config length (default ~monthly), UTC boundaries; derive the current season id from a timestamp.
- [x] 2.2 End-of-season snapshot job (reuse `cymbra-worker`): at season rollover, snapshot final global standings into a lightweight history; start a new season. Idempotent.

## 3. Global ranking read (backend)

- [x] 3.1 Global-score computation: difficulty-weighted best-N over a user's season bests for a mode.
- [x] 3.2 `GetGlobalLeaderboard(mode, season, page)` RPC: ranked **public + age-eligible** entries only (reuse #5 gate, fail-closed), by global season score desc with tie-break (contributing pieces, then earliest); no sensitive fields; paging. Always return the caller's own rank/score. Viewing open to any authenticated user.
- [x] 3.3 Extend account-deletion erasure (#5/#6 cascade / `purge_user`) to `global_season_bests` and snapshot entries.

## 4. App destination + profile standing

- [x] 4.1 A Community/Leaderboards screen (Riverpod notifier + service seam): tempo/reaction toggle + season selector (current + snapshotted past seasons); ranked public entries; own rank/score shown and highlighted.
- [x] 4.2 On the profile, a global-rank standing (current-season rank + score per mode) linking to the Community screen; private/under-age players still see their own standing.

## 5. Tests & verification

- [x] 5.1 Rust: best-N + difficulty weighting (volume alone doesn't raise score; harder piece worth more); monotonic season best (better raises, worse/replay no-op); ranking + tie-break; season rollover + snapshot (fresh season, per-piece all-time bests untouched, idempotent); listing gate + own-rank; erasure. `cargo llvm-cov ... --fail-under-lines 80`.
- [x] 5.2 Flutter: Community screen (mode toggle, season selector, ranked entries, own highlighted); profile global standing shown even when private (via fakes). `flutter test --coverage` ≥ 80%.
- [x] 5.3 `cargo fmt`/`clippy` + `melos run analyze`/`dart format` clean; regenerate codegen as needed.
- [x] 5.4 `openspec validate add-global-leaderboard --strict` passes.
