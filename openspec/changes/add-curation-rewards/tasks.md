## 1. Data model (backend)

- [ ] 1.1 Migration: append-only `curation_points` ledger (user_id, award_kind coverage/honesty/adjustment, amount, catalog_score_id nullable, created_at) + index by user.
- [ ] 1.2 Migration: per-rating settlement state (settled flag + settled-against source consensus/moderator + settled_at), and per-score consensus-settlement state, so honesty is awarded once and idempotently.
- [ ] 1.3 Migration: unlock/badge state (user_id, unlock/badge key, granted_at) and, if needed, a denormalized user balance/level for cheap reads.
- [ ] 1.4 Config for point values, diminishing curve, daily cap, level thresholds, consensus settlement minimum, and floor amount. Seed the straw-man defaults from design.md "Starting Configuration": coverage bands 10/6/3/1/0 (by existing ratings 0 / 1–4 / 5–19 / 20–49 / ≥50), daily cap 60; honesty aligned 8 (moderator) / 5 (consensus), floor 1, alignment midpoint 3.0 with ±0.25 ambiguous band, consensus min 8 raters; levels 50/150/350/700/1200/2000/3000 then +1200; piano unlocks at L1/L3/L5, "Patron" premium tier at L6 (future, not granted); badges First Note, Curator I/II/III (10/100/500), Sharp Ear I/II (25/100 aligned), Trailblazer (20 first-rater).

## 2. Coverage award (backend)

- [ ] 2.1 On the app rating path (#2), award coverage points: diminishing by the score's existing rating count, capped per user per day, only when the engagement gate passed.
- [ ] 2.2 Record the engagement signal (score previewed/opened before rating) so the gate is checkable server-side; a rating without it earns no coverage points.
- [ ] 2.3 Ensure BO moderation endpoints award nothing; awarding keys off the app rating path only (role-independent).

## 3. Honesty settlement (backend)

- [ ] 3.1 Settle honesty when a moderator decides (hook into #3's `SetModerationStatus`): award aligned raters the full bonus, misaligned the non-negative floor; skip the deciding user's own rating (no self-settlement).
- [ ] 3.2 Settle honesty by community consensus: when a score crosses the consensus minimum, freeze the aggregate as truth and settle each unsettled rating (worker sweep on `cymbra-worker`, idempotent by settlement state). Moderator truth outweighs/ supersedes consensus.
- [ ] 3.3 Guarantee award-once and idempotency via the settlement state; support an optional single re-settlement adjustment when a late moderator decision overrides a consensus settlement (appended as a correcting ledger entry).

## 4. Levels, unlocks, badges (backend + app)

- [ ] 4.1 Derive level from points (monotonic, config thresholds); expose points/level/next-unlock via an RPC.
- [ ] 4.2 Grant piano/SoundFont unlocks at tiers and wire them into `piano-sound-selection`; keep them durable.
- [ ] 4.3 Grant badges at milestones (first ratings, N aligned ratings, rare-score coverage…); durable.
- [ ] 4.4 Declare the temporary-premium tier as future (no premium granted now).
- [ ] 4.5 App: a reward/progress surface (Riverpod notifier + service seam) showing points, level, next unlock + progress, and badges.

## 5. Curator reliability indicator (BO)

- [ ] 5.1 Backend read: per-user total ratings, coverage contribution, and alignment rate over settled ratings; guarded to `moderator`/`admin`.
- [ ] 5.2 Vue back office: read-only per-user reliability panel (surfaced from the users/roles view); no automation.

## 6. Tests & verification

- [ ] 6.1 Rust: coverage diminishing + daily cap + engagement gate; honesty aligned vs floor (never negative) + award-once; moderator-outweighs-consensus; no self-settlement; consensus sweep idempotent; ledger balance. `cargo llvm-cov ... --fail-under-lines 80`.
- [ ] 6.2 Rust: staff app-rating earns points; BO actions award nothing.
- [ ] 6.3 Flutter: progress surface via a fake service (points/level/next-unlock/badges); unlocked piano appears in selection. `flutter test --coverage` ≥ 80%.
- [ ] 6.4 Vue: reliability panel renders and is gated to moderator/admin (own test setup).
- [ ] 6.5 `cargo fmt`/`clippy` + `melos run analyze`/`dart format` clean; regenerate codegen as needed.
- [ ] 6.6 `openspec validate add-curation-rewards --strict` passes.
