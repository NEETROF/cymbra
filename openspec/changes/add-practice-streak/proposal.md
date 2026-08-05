## Why

The reward system (`add-curation-rewards`) rewards **curation**, not **practice** —
and nothing today pulls a user back day after day. A **practice streak** (consecutive
days with at least one play) is the classic daily-retention loop: a highly visible
counter that hurts to lose, plus an evening reminder before it breaks, plus a way to
protect it. This turns the app's core act (playing) into a habit and gives curation
points a second, daily use (freezing a streak).

## What Changes

- **Server-tracked streak** — on `RecordPlaySession` the server maintains
  `current_streak`, `longest_streak`, and `last_played_date`, using the **client-offset
  local date** (the same day convention as `play_core`). The server is the source of
  truth (anti-cheat, multi-device correct); the client only displays.
- **Highly visible in the app-bar chip** — the existing curator standing pill (level ·
  points) gains a **🔥 flame + day-count** segment, so the streak is always in view on
  the home screen.
- **Streak protection = points freeze (confirmed spend)** — a missed day breaks the
  streak, **unless** the user spends points to freeze it. No silent auto-debit: on the
  next launch after a missed day, within a short grace window, the app offers
  *"recover your N-day streak for X points?"* — a confirmed spend (a ledger debit,
  like the score day-slot), mirroring the unlock-confirmation pattern.
- **Evening reminder via the push platform** — the streak registers a
  **notification type** on `add-push-notifications`: a daily worker job at a
  **back-office-configurable hour** (a hot-reloadable flag) targets users with
  `current_streak > 0 AND last_played_date < today` (server-side truth), on the
  FCM platforms (iOS/Android/macOS). Windows/Linux get no push; the streak stays
  visible in-app.
- **Streak badges** — a `PracticeStreak` badge family (e.g. 7 / 30 / 100 days),
  measured against `longest_streak`, added to the existing `BADGES` catalogue.

Out of scope: the push platform itself (this consumes it); per-play volume rewards
(streak is about *days*, not count); Windows/Linux app-closed reminders; auto-freeze
without confirmation; leaderboards (separate feature).

## Capabilities

### New Capabilities
- `practice-streak`: the consecutive-day practice streak — server-maintained on the
  play-record path (client-offset day), shown in the app-bar chip as a flame +
  day-count, protected by a confirmed points **freeze**, reminded by a
  flag-scheduled push (on the push platform) targeting at-risk users, and rewarded by
  a `PracticeStreak` badge family.

### Modified Capabilities
<!-- The badge catalogue gains a new family; it is additive (a new metric + entries),
     tracked here as part of the new practice-streak capability rather than a
     requirement change to curation-rewards. -->

## Impact

- **Depends on** `add-push-notifications` (device tokens, timezone, consent, the
  worker dispatch + `PushSender`) and `add-curation-rewards` (points ledger/balance,
  the `BADGES` catalogue + `earned_badges`). Reuses `play_core`'s client-offset date.
- **Backend**: streak state columns + a host-testable streak-transition core updated
  on `RecordPlaySession`; a confirmed **freeze** RPC (ledger debit + streak restore,
  within a grace window); a `PracticeStreak` badge metric + catalogue entries; the
  streak-reminder notification **type** (candidate query + message + schedule-hour
  flag) registered on the push worker.
- **App**: the flame + day-count segment in the standing pill; the recover/freeze
  confirmation flow; surfacing streak state (current/longest/at-risk).
- **Back office**: the reminder-hour flag exposed in the notifications panel.
- **Coverage**: Rust ≥ 80% for the streak-transition + freeze-decision + badge cores
  (host-testable); gRPC/worker glue excluded. App ≥ 80% via fakes for the chip +
  recover flow; BO under its test setup.
