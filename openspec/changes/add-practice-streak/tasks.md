## 1. Streak state (backend)

- [ ] 1.1 Migration: streak columns (`current_streak`, `longest_streak`, `last_played_date`) per user (music schema).
- [ ] 1.2 Host-testable `advance(state, today) -> StreakState` (same-day no-op, next-day increment, gap reset, longest tracking); unit-test each branch.
- [ ] 1.3 Wire `advance` into `RecordPlaySession` ([backend/music/src/play_grpc.rs]) using the `play_core` client-offset date; persist the new state.
- [ ] 1.4 A streak read RPC / extend an existing read: expose `current`, `longest`, and at-risk (played today?) for the app.

## 2. Freeze / recovery (backend)

- [ ] 2.1 Flags: `streak_freeze_cost` (points) + `streak_grace_days` (hot-reloadable).
- [ ] 2.2 Host-testable freeze-decision core: within grace window? enough spendable balance? → allow/deny; unit-test boundaries.
- [ ] 2.3 `RecoverStreak` RPC: atomic ledger debit (`reason=streak_freeze`) + restore `current_streak` + set `last_played_date=today`; reject on insufficient balance / outside grace.

## 3. Reminder notification type (backend, on the push platform)

- [ ] 3.1 Register a `streak_reminder` category with a schedule-hour flag (BO hot-reload) on the push worker dispatch.
- [ ] 3.2 Candidate query: `current_streak > 0 AND last_played_date < today` (server-side at-risk set), grouped by user local hour.
- [ ] 3.3 Localised message ("garde ta série de N jours"); rely on the platform for consent/kill-switch/FCM-platform selection.

## 4. Badges (backend)

- [ ] 4.1 Add `BadgeMetric::PracticeStreak` + catalogue entries (`streak_7`/`streak_30`/`streak_100`) measured against `longest_streak`.
- [ ] 4.2 Feed the streak counter into `earned_badges`; unit-test earn-at-threshold + retained-after-drop.

## 5. App (Flutter)

- [ ] 5.1 Add the 🔥 flame + day-count segment to `CuratorStandingPill` (muted at 0); source from the streak read.
- [ ] 5.2 Recover flow: on launch, if broken within grace, offer "recover N-day streak for X points?" → confirm → `RecoverStreak`; react to state (no await-and-branch).
- [ ] 5.3 Optional in-app at-risk nudge at launch for platforms without push (Windows/Linux).
- [ ] 5.4 Widget/state tests: chip renders streak/muted; recover spends + restores on confirm; no debit without confirm; insufficient balance handled.

## 6. Back office (Vue)

- [ ] 6.1 Surface the `streak_reminder` hour flag in the notifications panel (from `add-push-notifications`).

## 7. Coverage, gates & verification

- [ ] 7.1 Rust: `cargo fmt` + `clippy -D warnings`; `cargo llvm-cov --workspace --fail-under-lines 80` (advance + freeze-decision + badge cores covered; gRPC/worker glue excluded).
- [ ] 7.2 Flutter: `dart run build_runner build`; `melos run analyze` + `dart run custom_lint`; `flutter test --coverage` ≥ 80%.
- [ ] 7.3 Back office: `yarn lint` + `yarn test` green.
- [ ] 7.4 Manual: play → streak increments; skip a day → break → recover for points (confirmed) restores; reminder fires at the flag hour only for at-risk users; longest-streak badge earned.
- [ ] 7.5 `openspec validate add-practice-streak --strict` passes.
