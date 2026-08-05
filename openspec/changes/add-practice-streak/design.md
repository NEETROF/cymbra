## Context

`RecordPlaySession` ([backend/music/src/play_grpc.rs](backend/music/src/play_grpc.rs))
is the single ingest for a play; the leaderboard already updates there, so it is the
natural place to advance a streak. `play_core.rs` already computes a **local date
from the client UTC offset** — the exact day-boundary a streak needs. The reward
system (`add-curation-rewards`) provides the points **ledger** + spendable balance
and a static `BADGES` catalogue with `earned_badges(counts)`. The
`add-push-notifications` platform provides device tokens, per-user timezone, consent,
and a worker dispatch that a feature parameterises with a **category** (candidate
query + message + schedule-hour flag). This change is the first consumer of that
platform.

The app-bar right slot already hosts the curator standing pill (`CuratorStandingPill`,
via `account_menu.dart`); the streak goes **into** that pill, not beside it.

## Goals / Non-Goals

**Goals:**
- A correct, server-authoritative consecutive-day streak on the play path.
- Always-visible streak (flame + days) inside the existing app-bar chip.
- A confirmed points **freeze** to protect a streak (no silent debit).
- An evening reminder to at-risk users, at a BO-configurable hour, via the push
  platform (iOS/Android/macOS).
- `PracticeStreak` badges on the existing catalogue.

**Non-Goals:**
- The push platform; local notifications; Windows/Linux app-closed reminders.
- Per-play volume rewards; leaderboards.
- Auto-freeze without confirmation.

## Decisions

### 1. Streak transition (host-testable core)
```
struct StreakState { current: i64, longest: i64, last_played: Date }
fn advance(state, today) -> StreakState
//  today == last_played           -> unchanged (already counted today)
//  today == last_played + 1 day   -> current += 1 (consecutive)
//  today >  last_played + 1 day   -> current = 1 (broken; a new run starts today)
//  longest = max(longest, current); last_played = today
```
`today` is the client-offset local date (reuse `play_core`). Pure and unit-tested
(same-day no-op, consecutive increment, gap reset, longest tracking). Called inside
`RecordPlaySession`; the DB read/write is excluded glue.

### 2. Freeze = confirmed points spend within a grace window
A missed day breaks the streak. Recovery is **user-initiated and confirmed** — never
a silent auto-debit:
- On launch, if `today > last_played + 1` **and** within a **grace window** (e.g. the
  streak broke ≤ 1 day ago), the app surfaces *"recover your N-day streak for X
  points?"*.
- On confirm, an atomic op: a **ledger debit** (`reason=streak_freeze`, referencing
  the day) + restore `current` to its pre-break value and set `last_played = today`.
  Rejected if the balance is insufficient. Beyond the grace window the streak is gone
  (no recovery). The freeze-decision (in window? enough balance?) is a covered core;
  the transaction is excluded glue. Freeze cost + grace length are flags.

This mirrors the score day-slot spend and keeps points a meaningful daily sink.

### 3. Display in the standing pill
`CuratorStandingPill` gains a **flame + current-streak** segment alongside level ·
points. One consolidated control. The value comes from the streak read (current,
longest, at-risk); `0` renders a muted/hint state (start a streak). No new app-bar
real estate.

### 4. Reminder as a push notification type
Register a `streak_reminder` **category** on the push platform:
- **Schedule**: a daily worker job at the category's **flag hour** (BO hot-reload).
- **Candidates**: `current_streak > 0 AND last_played_date < today` (server-side
  truth — only users actually at risk).
- **Message**: "Il te reste peu de temps pour garder ta série de N jours" (localised).
- Consent + kill-switch + FCM-platform selection are the platform's job; this feature
  only supplies the query, message, and schedule hour. Windows/Linux (no token) are
  naturally excluded; those users keep the in-app chip (and an optional launch-time
  at-risk nudge).

### 5. Streak badges
Add `BadgeMetric::PracticeStreak` (measured against `longest_streak`) and catalogue
entries (e.g. `streak_7` / `streak_30` / `streak_100`). `earned_badges` already
diffs earned-vs-granted; feeding it the streak counter is the only wiring. Durable
like all badges.

## Risks / Trade-offs

- **Streak anxiety / dark-pattern feel.** Loss-aversion loops can feel manipulative.
  Mitigation: the freeze (a forgiving recovery) softens it; reminders are opt-in and
  category-toggleable; the reminder-hour + freeze cost are tunable; a user can ignore
  it with only an in-app cue.
- **Timezone spoofing.** A client-set offset could fake "consecutive" days.
  Mitigation: reuse the vetted `play_core` date; the streak is a soft product
  mechanic, not a security boundary; longest_streak is monotonic so damage is capped.
- **Grace-window fairness.** Too short feels punishing; too long makes streaks
  meaningless. Mitigation: it's a flag — start ~1 day and tune with data.
- **Chip overload.** Level · points · flame in one pill risks clutter. Mitigation:
  compact segment, muted when streak is 0; validate the layout on device.
- **Points sink balance.** Freezes + score day-slots both drain points. Mitigation:
  freeze cost is a flag; balance earn/spend with the same levers as the reward
  economy.
