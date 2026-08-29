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
- [x] 7.4 Manual: play → streak increments; skip a day → break → recover for points (confirmed) restores.
      DONE on a live stack (server + Postgres, migrations 0026 + jobs 0015) on
      macOS, signed in:
      - a scored run played to the end takes the chip to 1; a second run the same
        local day leaves it there;
      - a measure-range practice alone secures the day, and a practice followed by
        a full play on the same day counts as one day, not two;
      - a practice interrupted by killing the app mid-loop is still delivered on
        the next drain (the durability fix);
      - a break inside the grace window offers the recovery: declining spends
        nothing, confirming writes the `streak_freeze` debit and restores the run,
        and an insufficient balance is refused without a debit;
      - past the grace window there is no offer and the RPC refuses.
- [x] 7.5 `openspec validate add-practice-streak --strict` passes.
- [ ] 7.6 Manual (deferred): the push reminder fires at the configured local hour
      for at-risk users only, and never for someone who already played today.
      NOT DONE — needs a Firebase project (`CYMBRA_FCM_SERVICE_ACCOUNT_JSON` on the
      worker) and a device holding a token; Windows/Linux never have one. It also
      needs an operator to store overrides for `notifications.enabled` and
      `notifications.category.practice_streak.enabled`, which ship **off** by
      design: merging this code messages nobody.
      The *selection* is unit-tested (`streak_core::at_risk_user_ids` /
      `reminder_groups`), including the already-played-today exclusion across
      timezones and the per-(locale, streak) batching. What stays unverified here is
      the hand-off to FCM — which this change does not own, and which
      `add-push-notifications` validated end-to-end on a device.

## 8. Resolved after rebasing onto #220 / #222

Two changes landed on main while this one was in flight and redefined what a
"streak" is; both are now reconciled in code rather than left as a divergence.

- [x] 8.1 **A practice day holds the streak.** `advance_streak` is called from
      `record_practice` as well as `record_play_session`, so the chip, the activity
      heatmap and the `Consistency` badge family (#220, whose counter is the union
      of play and practice days) all count the same thing. Without this, the player
      who spent the evening drilling the hardest bar earned practice points (#222)
      and kept their badge streak, but lost the chip streak — and could then spend
      those very points to buy it back. Spec + design + proposal amended; two gRPC
      tests cover practice-alone and practice-then-play-same-day.
- [x] 8.2 **An interrupted practice is no longer lost.** The practice record is
      captured into the durable outbox at its first sounded onset instead of at a
      clean ending, so an app killed mid-loop still delivers it. Still exactly one
      record per session (never per lap). This mattered before, but the streak makes
      it expensive: a dropped practice now costs a day the player actually earned.

## 9. Manual validation fallout: the offer that would not go away

Reported on-device (2026-08-23): the recovery confirmation re-opened on **every**
app start after "Pas cette fois", and the flame stayed at 1. Three defects, all in
the same seam — a stored count and a widget-local decision standing in for state
nobody owned.

- [x] 9.1 **The decline is persisted.** It lived in `_StreakListenerState._offered`,
      so it survived neither a cold start nor a navigation between the two screens
      that mount `StreakListener`. It is now `streakRecoveryDeclineProvider` — the
      local day of the refusal, written through the `PreferencesService` seam — and
      the offer decision moved wholesale into `streakRecoveryOfferedProvider`, which
      yields nothing until that decline has been read (offering while it loads is
      exactly how a refused question comes back). The at-risk nudge got the same
      treatment via `streakNudgeShownProvider`.
- [x] 9.2 **A broken run stops being displayed.** `StreakStanding::display_streak`
      returned `state.current` raw, and nothing decays that column — so a run lost
      weeks ago still lit the flame, while the dialog next to it announced the streak
      was broken. Liveness is now `StreakState::is_live(today)` and the standing
      carries the day it was read on; `recoverable_streak` still names what a freeze
      would restore.
- [x] 9.3 **A dead streak defends nothing.** `at_risk` matched every gap ≥ 1, so the
      in-app cue and the reminder sweep both fired on long-dead rows ("keep your
      1-day streak"). It is now exactly "played yesterday, not today", and
      `live_streaks()` pre-filters on `last_played_date >= CURRENT_DATE - 2` (coarse:
      the per-timezone decision stays in the core). `standing()` reads the balance on
      `is_broken` rather than `at_risk`, which the narrowing would otherwise have
      priced as unaffordable.

## 10. Beta feedback (2026-08-27): the offer that came back anyway

The drum beta's tester reported the recovery question returning "at every
launch" on a build that already carried §9. Two holes, both in *how long a "no"
lasts* rather than in whether it is recorded at all.

- [x] 10.1 `streak_listener.dart`: the refusal is recorded **before** the
  `mounted` check, not after. `ref.read` of the decline notifier is hoisted above
  the `showDialog` await, so backgrounding the app with the question on screen —
  or any rebuild that takes the subtree down — no longer loses the answer. The
  `mounted` guard stays where it is still needed (the confirm path, which
  reads other providers). Widget-tested by tearing the listener's subtree down
  while the dialog is up and popping the dialog's own route; the test fails
  against the previous ordering
- [x] 10.2 `streak_notifier.dart`: the decline is keyed to the **offer**
  (`recoverableStreak`), not to the local calendar day. §9 filed it under the day
  on the reasoning that the grace window is one day wide, so a second question
  could only be a second break — but `streak.grace_days` is a back-office value,
  and a wider one re-asks the identical question every morning. A recovery offer
  only exists while the user has not played since the break, so the run it would
  restore identifies it for its whole life. New prefs key
  (`streak_recovery_declined_run`), so a stored day string is never read back as
  a run count: an upgrading device has no recorded decline and is asked once
- [x] 10.3 Tests both ways: the same standing break stays answered across a
  later launch, and a genuinely different break is still asked about (notifier
  and widget level)
- [ ] 10.4 Back office: read `streak.grace_days` in production. If it is wider
  than the default `1`, the daily re-ask was legitimate behaviour under the old
  key and 10.2 is what fixes it; if it is `1`, 10.1 was the whole story. Record
  the value here either way
- [ ] 10.5 On-device with the tester: refuse the offer, force-quit, relaunch —
  the question is gone and stays gone
