## 1. Data model (backend)

- [x] 1.1 Migration: append-only `curation_points` ledger (user_id, award_kind coverage/honesty/adjustment, amount, catalog_score_id nullable, created_at) + index by user.
- [x] 1.2 Migration: per-rating settlement state (settled flag + settled-against source consensus/moderator + settled_at), and per-score consensus-settlement state, so honesty is awarded once and idempotently.
- [x] 1.3 Migration: redemption events on the ledger (kind `redeem`, negative-to-balance, reward key) so spendable balance = lifetime earned − redeemed; grant/badge state (user_id, key, granted_at); optional denormalized lifetime/balance/level for cheap reads.
- [x] 1.4 Config for point values, diminishing curve, daily cap, level thresholds, consensus settlement minimum, floor amount, and **reward-shop item costs**. Seed the straw-man defaults from design.md "Starting Configuration": coverage bands 10/6/3/1/0 (by existing ratings 0 / 1–4 / 5–19 / 20–49 / ≥50), daily cap 60; honesty aligned 8 (moderator) / 5 (consensus), floor 1, alignment midpoint 3.0 with ±0.25 ambiguous band, consensus min 8 raters; levels 50/150/350/700/1200/2000/3000 then +1200 (from **lifetime** points); shop piano costs ~50/150/300, future premium item ~500 (not redeemable); badges First Note, Curator I/II/III (10/100/500), Sharp Ear I/II (25/100 aligned), Trailblazer (20 first-rater).

## 2. Coverage award (backend)

- [x] 2.1 On the app rating path (#2), award coverage points: diminishing by the score's existing rating count, capped per user per day, only when the engagement gate passed.
- [x] 2.2 Record the engagement signal (score previewed/opened before rating) so the gate is checkable server-side; a rating without it earns no coverage points.
- [x] 2.3 Ensure BO moderation endpoints award nothing; awarding keys off the app rating path only (role-independent).

## 3. Honesty settlement (backend)

- [x] 3.1 Settle honesty when a moderator decides (hook into #3's `SetModerationStatus`): award aligned raters the full bonus, misaligned the non-negative floor; skip the deciding user's own rating (no self-settlement).
- [x] 3.2 Settle honesty by community consensus: when a score crosses the consensus minimum, freeze the aggregate as truth and settle each unsettled rating (worker sweep on `cymbra-worker`, idempotent by settlement state). Moderator truth outweighs/ supersedes consensus.
- [x] 3.3 Guarantee award-once and idempotency via the settlement state; support an optional single re-settlement adjustment when a late moderator decision overrides a consensus settlement (appended as a correcting ledger entry).

## 4. Levels, unlocks, badges (backend + app)

- [x] 4.1 Derive **lifetime points, spendable balance, and level** (monotonic, config thresholds) from the ledger; expose them + next-level progress via an RPC.
- [x] 4.2 Reward shop: list redeemable items (pianos/SoundFonts) with costs; a `RedeemReward` RPC that checks balance, deducts (append a `redeem` ledger event), is idempotent, and grants the item durably wired into `piano-sound-selection`; refuse if balance insufficient; premium item listed but not redeemable.
- [x] 4.3 Grant badges at milestones (first ratings, N aligned ratings, rare-score coverage…); earned only, durable, not purchasable.
- [ ] 4.5 App: a **full-screen curator profile** (Riverpod notifier + service seam) — header (level, lifetime points, progress), **spendable balance + reward-shop entry**, badge grid (earned + locked with hints), and personal stats (rating count, coverage contribution, own alignment rate).
- [ ] 4.6 App: a persistent level/points **chip** in the hub + rating-deck app bars that opens the curator profile; reflects live standing.
- [ ] 4.7 App: **immediate "+N"** coverage cue on rating, and a **celebration** modal on level-up / reward redeemed / badge earned (reuse `gamified-feedback` / `session_summary_modal` patterns).
- [ ] 4.8 App: surface **deferred honesty awards** — a notification cue on the profile entry point + a recent-activity list stating each award's amount and source (consensus vs moderator). Needs a backend read for recent award events (from the ledger).
- [ ] 4.9 App: the **reward shop** screen (redeem chosen items with balance) and, in `piano-sound-selection`, show not-yet-redeemed pianos with a lock affordance, their **cost**, and a redeem action.

## 5. Curator reliability indicator (BO)

- [x] 5.1 Backend read: per-user total ratings, coverage contribution, and alignment rate over settled ratings; guarded to `moderator`/`admin`.
- [ ] 5.2 Vue back office: read-only per-user reliability panel (surfaced from the users/roles view); no automation.

## 6. Tests & verification

- [x] 6.1 Rust: coverage diminishing + daily cap + engagement gate; honesty aligned vs floor (never negative) + award-once; moderator-outweighs-consensus; no self-settlement; consensus sweep idempotent; ledger lifetime vs spendable balance; redemption deducts once, refuses when balance insufficient, never negative, spending doesn't lower level. `cargo llvm-cov ... --fail-under-lines 80`.
- [x] 6.2 Rust: staff app-rating earns points; BO actions award nothing.
- [ ] 6.3 Flutter (via fakes): curator profile (level/lifetime points/balance/shop entry/badges/stats); app-bar chip opens it; "+N" cue on rating; deferred-award activity list + notification cue; reward shop redeems an affordable item and refuses an unaffordable one; locked piano shows cost and becomes selectable once redeemed. `flutter test --coverage` ≥ 80%.
- [ ] 6.4 Vue: reliability panel renders and is gated to moderator/admin (own test setup).
- [ ] 6.5 `cargo fmt`/`clippy` + `melos run analyze`/`dart format` clean; regenerate codegen as needed.
- [ ] 6.6 `openspec validate add-curation-rewards --strict` passes.
