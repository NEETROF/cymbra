## 1. Streak state (backend)

- [x] 1.1 Migration: streak columns (`current_streak`, `longest_streak`, `last_played_date`) per user (music schema).
- [x] 1.2 Host-testable `advance(state, today) -> StreakState` (same-day no-op, next-day increment, gap reset, longest tracking); unit-test each branch.
- [x] 1.3 Wire `advance` into `RecordPlaySession` ([backend/music/src/play_grpc.rs]) using the `play_core` client-offset date; persist the new state.
- [x] 1.4 A streak read RPC / extend an existing read: expose `current`, `longest`, and at-risk (played today?) for the app.

## 2. Freeze / recovery (backend)

- [x] 2.1 Flags: `streak_freeze_cost` (points) + `streak_grace_days` (hot-reloadable).
- [x] 2.2 Host-testable freeze-decision core: within grace window? enough spendable balance? → allow/deny; unit-test boundaries.
- [x] 2.3 `RecoverStreak` RPC: atomic ledger debit (`reason=streak_freeze`) + restore `current_streak` + set `last_played_date=today`; reject on insufficient balance / outside grace.

## 3. Reminder notification type (backend, on the push platform)

- [x] 3.1 Register a `streak_reminder` category with a schedule-hour flag (BO hot-reload) on the push worker dispatch.
- [x] 3.2 Candidate query: `current_streak > 0 AND last_played_date < today` (server-side at-risk set), grouped by user local hour.
- [x] 3.3 Localised message ("garde ta série de N jours"); rely on the platform for consent/kill-switch/FCM-platform selection.

## 4. Badges (backend)

- [x] 4.1 Add `BadgeMetric::PracticeStreak` + catalogue entries (`streak_7`/`streak_30`/`streak_100`) measured against `longest_streak`.
      SUPERSEDED by `add-achievement-badges` (#220), landed on main first. The old
      `BADGES` array this task targeted no longer exists; the cross-domain registry
      (`badges_core::REGISTRY`) already declares `BadgeMetric::LongestStreak` with
      `streak_1/2/3` at 3/7/30 days in the `Consistency` family. My entries were
      dropped in the rebase rather than duplicated.
- [x] 4.2 Feed the streak counter into `earned_badges`; unit-test earn-at-threshold + retained-after-drop.
      SUPERSEDED likewise: the registry derives `longest_streak` itself, from the
      UNION of `play_sessions` and `practice_sessions` days (`badges.rs::fold`), so
      no port from this module is needed. **NOTE:** that makes the registry's streak
      and this change's streak two different quantities — see the open question at
      the end of this file.

## 5. App (Flutter)

- [x] 5.1 Add the 🔥 flame + day-count segment to `CuratorStandingPill` (muted at 0); source from the streak read.
- [x] 5.2 Recover flow: on launch, if broken within grace, offer "recover N-day streak for X points?" → confirm → `RecoverStreak`; react to state (no await-and-branch).
- [x] 5.3 Optional in-app at-risk nudge at launch for platforms without push (Windows/Linux).
- [x] 5.4 Widget/state tests: chip renders streak/muted; recover spends + restores on confirm; no debit without confirm; insufficient balance handled.

## 6. Back office (Vue)

- [x] 6.1 Surface the `streak_reminder` hour flag in the notifications panel (from `add-push-notifications`).
      No BO code change: the panel is registry-driven (it renders every declared
      `notifications.*` key), so declaring the `practice_streak` triplet in
      `cymbra-feature-flags` is what makes the row — enable, local hour,
      foreground — appear. Verified against the existing store/tests, which
      already use `practice_streak` as their fixture category.

## 7. Coverage, gates & verification

- [x] 7.1 Rust: `cargo fmt` + `clippy -D warnings`; `cargo llvm-cov --workspace --fail-under-lines 80` (advance + freeze-decision + badge cores covered; gRPC/worker glue excluded).
      Clippy clean, 930 tests green, workspace lines 93.3 %. Cores: `streak_core`
      99.5 %, `streak_module` 95.8 %, `streak` (port + fake) 86.1 %. `pg_streak.rs`
      added to the CI ignore regex alongside the other SQL adapters.
- [x] 7.2 Flutter: `dart run build_runner build`; `melos run analyze` + `dart run custom_lint`; `flutter test --coverage` ≥ 80%.
      1388 tests green (16 new), analyze + custom_lint clean, app lines 86.6 %.
- [x] 7.3 Back office: `yarn lint` + `yarn test` green.
      209 tests green, lint clean. (`yarn gen` first — a fresh worktree has no
      generated stubs.)
- [ ] 7.4 Manual: play → streak increments; skip a day → break → recover for points (confirmed) restores; reminder fires at the flag hour only for at-risk users; longest-streak badge earned.
      NOT DONE — needs a live stack: Postgres (migrations 0026 + jobs 0015), the
      worker, a Firebase project, and a real device. The reminder additionally
      needs an operator to store overrides for `notifications.enabled` and
      `notifications.category.practice_streak.enabled`, which ship **off** by
      design: merging this code messages nobody.
- [x] 7.5 `openspec validate add-practice-streak --strict` passes.

## 8. Open question after rebasing onto #220 / #222

Two changes landed on main while this one was in flight, and they redefine what a
"streak" is:

- **#220 (achievement badges)** declares `streak_1/2/3` (3/7/30 days) whose counter
  is the longest run of days with **a play OR a practice** — "did you sit down at
  the keyboard".
- **#222 (play rewards)** pays points for a scored run *and* a once-a-day practice
  award, on the same ingest.

This change's streak counts **scored plays only**: `advance_streak` is called from
`record_play_session`, never from `record_practice`. So a player who spends an
evening drilling a measure range:

  1. earns practice points (#222),
  2. keeps their badge streak alive (#220),
  3. **loses** the chip streak this change draws — and can then spend the points
     they just earned to buy it back.

That is incoherent as it stands. The fix is one line (call `advance_streak` from
`record_practice` too) plus a spec amendment, but it is a product decision, not a
rebase decision, so it is left open here rather than made silently.

Related: a practice session is only recorded once it is **flushed**
(`_flushPracticeSession`), which does not happen if the app is killed mid-loop —
so an hour of real practice can still leave no trace on either streak.
