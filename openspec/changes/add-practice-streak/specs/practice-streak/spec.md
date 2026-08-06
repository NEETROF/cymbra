## ADDED Requirements

### Requirement: Server-tracked consecutive-day streak

The server SHALL maintain, per user, a `current_streak`, a `longest_streak`, and a
`last_played_date`, advanced on the `RecordPlaySession` path using the caller's
**client-offset local date** (the same day convention as play activity). A second
play on the **same** local day SHALL NOT change the streak; a play on the **next**
day SHALL increment it; a play after a **gap** SHALL reset the current run to 1.
`longest_streak` SHALL never decrease. The server is the source of truth; the client
displays but does not compute the streak.

#### Scenario: First play starts a streak
- **WHEN** a user records their first-ever play
- **THEN** current_streak = 1 and longest_streak = 1

#### Scenario: Same-day replay does not advance
- **WHEN** a user records a second play on the same local day
- **THEN** current_streak is unchanged

#### Scenario: Next-day play increments
- **WHEN** a user plays on the day after last_played_date
- **THEN** current_streak increases by 1 and longest_streak tracks the max

#### Scenario: A gap resets the run
- **WHEN** a user plays after skipping one or more whole days (without a freeze)
- **THEN** current_streak resets to 1 while longest_streak is retained

### Requirement: Streak shown in the app-bar standing chip

The app SHALL show the current streak as a **flame + day-count** segment inside the
existing curator standing pill (alongside level and points), so it is always visible
on the home screen. A zero streak SHALL render a muted/hint state rather than being
hidden.

#### Scenario: Active streak is visible
- **WHEN** a user with current_streak = N opens the home screen
- **THEN** the standing pill shows a flame with N

#### Scenario: Zero streak shows a hint
- **WHEN** a user has current_streak = 0
- **THEN** the pill shows a muted flame/hint (start a streak), not a hidden control

### Requirement: Streak protected by a confirmed points freeze

A missed day SHALL break the streak unless the user restores it by spending points,
**with explicit confirmation** — never a silent auto-debit. Within a defined grace
window after a break, the app SHALL offer to recover the pre-break streak for a
(flag-configured) point cost. On confirmation the server SHALL atomically write a
points ledger debit and restore `current_streak`, setting `last_played_date` to
today; it SHALL reject if the balance is insufficient. Beyond the grace window the
streak SHALL NOT be recoverable.

#### Scenario: Recover within the grace window
- **WHEN** a user with enough points confirms recovering an N-day streak broken within the grace window
- **THEN** a ledger debit is written and current_streak is restored to N

#### Scenario: No silent debit
- **WHEN** a user's streak breaks
- **THEN** no points are spent unless the user explicitly confirms the recovery

#### Scenario: Insufficient balance is refused
- **WHEN** a user tries to recover but lacks the required points
- **THEN** no debit is written and the streak stays broken

#### Scenario: Beyond the grace window there is no recovery
- **WHEN** the grace window has elapsed since the break
- **THEN** the recovery offer is not shown and the streak cannot be restored

### Requirement: Evening reminder to at-risk users via the push platform

The streak SHALL register a `streak_reminder` notification **category** on the push
platform: a daily send at a **back-office-configurable hour** (a hot-reloadable flag)
targeting only users with `current_streak > 0 AND last_played_date < today` (the
server-side at-risk set). Consent, the kill-switch, and platform selection
(iOS/Android/macOS) are enforced by the push platform. Users who already played today
SHALL NOT be reminded. Windows/Linux users (no push token) SHALL NOT receive it and
SHALL keep the in-app streak cue.

#### Scenario: At-risk user is reminded
- **WHEN** the reminder job runs at the configured hour and a user has a streak but has not played today
- **THEN** that user receives the streak reminder (subject to consent/flags)

#### Scenario: Already-played user is not reminded
- **WHEN** the reminder job runs and a user already played today
- **THEN** that user is not sent a reminder

#### Scenario: Reminder hour is changed from the back office
- **WHEN** an admin changes the reminder-hour flag
- **THEN** subsequent sends use the new local hour without a deploy

### Requirement: Practice-streak badges

A `PracticeStreak` badge family SHALL be added to the badge catalogue, earned against
`longest_streak` at defined day thresholds (e.g. 7 / 30 / 100). Streak badges SHALL
be earned and durable like all badges (never revoked).

#### Scenario: Reaching a streak threshold earns the badge
- **WHEN** a user's longest_streak reaches a streak badge threshold
- **THEN** that badge is earned and shown on the profile badge grid

#### Scenario: A later drop does not revoke it
- **WHEN** the user's current streak later falls below the threshold
- **THEN** the earned streak badge is retained
