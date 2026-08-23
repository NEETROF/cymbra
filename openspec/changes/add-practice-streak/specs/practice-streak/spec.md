## ADDED Requirements

### Requirement: Server-tracked consecutive-day streak

The server SHALL maintain, per user, a `current_streak`, a `longest_streak`, and a
`last_played_date`, advanced on **both** the `RecordPlaySession` and
`RecordPractice` paths using the caller's **client-offset local date** (the same
day convention as play activity). A day counts when the user either played a
scored run **or** practised a passage: the streak measures showing up at the
keyboard, matching the activity heatmap and the consistency badges, and never
punishes the player on the day they worked hardest. A second activity on the
**same** local day SHALL NOT change the streak; an activity on the **next** day
SHALL increment it; an activity after a **gap** SHALL reset the current run to 1.
`longest_streak` SHALL never decrease. The server is the source of truth; the client
displays but does not compute the streak.

#### Scenario: First play starts a streak
- **WHEN** a user records their first-ever play
- **THEN** current_streak = 1 and longest_streak = 1

#### Scenario: Practice alone secures the day
- **WHEN** a user records only a practice session (an unscored selective run) on a day
- **THEN** the streak advances exactly as a scored run would

#### Scenario: Same-day replay does not advance
- **WHEN** a user records a second play on the same local day
- **THEN** current_streak is unchanged

#### Scenario: A practice and a play on the same day count once
- **WHEN** a user practises a passage and then plays the piece through on the same local day
- **THEN** current_streak advances by one day in total, not two

#### Scenario: Next-day play increments
- **WHEN** a user plays on the day after last_played_date
- **THEN** current_streak increases by 1 and longest_streak tracks the max

#### Scenario: A gap resets the run
- **WHEN** a user plays after skipping one or more whole days (without a freeze)
- **THEN** current_streak resets to 1 while longest_streak is retained

### Requirement: A practice reaches the server without a clean ending

A practice session SHALL be captured into the client's durable outbox as soon as
it becomes countable (its first sounded onset), not when it ends. A session that
never gets a clean ending — the app killed mid-loop, the device dying — SHALL
still be delivered. Capture SHALL remain once per practice session, never per
lap, so looping cannot inflate the day's activity.

#### Scenario: An interrupted practice still counts
- **WHEN** a user drills a passage and the app is killed without the range ever being closed
- **THEN** the practice record is already durable and is delivered on the next drain

#### Scenario: Looping does not multiply records
- **WHEN** a practice session loops several times
- **THEN** exactly one practice record exists for that session

### Requirement: Streak shown in the app-bar standing chip

The app SHALL show the current streak as a **flame + day-count** segment inside the
existing curator standing pill (alongside level and points), so it is always visible
on the home screen. A zero streak SHALL render a muted/hint state rather than being
hidden.

The number reported to the app SHALL be the **live** run — a run whose last
activity was today or yesterday — and SHALL be zero once the run is broken, even
though `current_streak` is still stored (nothing decays the row, and
`longest_streak` is measured from it). Liveness is a property of the stored state
**and** the reader's local day, resolved on every read: a run is otherwise
displayed as alive forever, and the lit flame contradicts the very dialog offering
to buy the streak back.

#### Scenario: Active streak is visible
- **WHEN** a user with current_streak = N opens the home screen
- **THEN** the standing pill shows a flame with N

#### Scenario: Zero streak shows a hint
- **WHEN** a user has current_streak = 0
- **THEN** the pill shows a muted flame/hint (start a streak), not a hidden control

#### Scenario: A broken run stops being displayed
- **WHEN** a user whose last activity was two or more local days ago opens the home screen
- **THEN** the pill shows the muted zero state, while longest_streak still reflects the run
  and any recovery offer still names what it would restore

### Requirement: Streak protected by a confirmed points freeze

A missed day SHALL break the streak unless the user restores it by spending points,
**with explicit confirmation** — never a silent auto-debit. Within a defined grace
window after a break, the app SHALL offer to recover the pre-break streak for a
(flag-configured) point cost. On confirmation the server SHALL atomically write a
points ledger debit and restore `current_streak`, setting `last_played_date` to
today; it SHALL reject if the balance is insufficient. A freeze SHALL be charged
AT MOST ONCE per user per local day, enforced by the ledger's idempotency key
rather than by the read-then-write decision alone. Beyond the grace window the
streak SHALL NOT be recoverable.

A declined offer SHALL be remembered **on the device for the local day it was
declined on**, so the same question is not re-opened on the next launch, nor when
the user reaches another screen that hosts the streak listener. Since the grace
window is one local day, declining silences that offer for its whole life; a break
on a later day is a new question and is asked again.

#### Scenario: A declined offer stays declined
- **WHEN** a user answers "not this time" and later relaunches the app while the same
  break is still inside the grace window
- **THEN** the confirmation is not shown again and nothing is debited

#### Scenario: Recover within the grace window
- **WHEN** a user with enough points confirms recovering an N-day streak broken within the grace window
- **THEN** a ledger debit is written and current_streak is restored to N

#### Scenario: No silent debit
- **WHEN** a user's streak breaks
- **THEN** no points are spent unless the user explicitly confirms the recovery

#### Scenario: Insufficient balance is refused
- **WHEN** a user tries to recover but lacks the required points
- **THEN** no debit is written and the streak stays broken

#### Scenario: Two confirmations of the same day charge once
- **WHEN** two recovery confirmations for the same local day reach the server at once
- **THEN** exactly one points debit is written and the streak is restored

#### Scenario: Beyond the grace window there is no recovery
- **WHEN** the grace window has elapsed since the break
- **THEN** the recovery offer is not shown and the streak cannot be restored

### Requirement: Evening reminder to at-risk users via the push platform

The streak SHALL register a `streak_reminder` notification **category** on the push
platform: a daily send at a **back-office-configurable hour** (a hot-reloadable flag)
targeting only users whose streak is **live and unsecured** — `current_streak > 0`
AND `last_played_date` is exactly the user's own yesterday (the server-side at-risk
set). A run last touched two or more days ago is already broken and SHALL NOT be
targeted: there is nothing left to defend, and "keep your N-day streak" about a
lost streak is wrong on its face. Consent, the kill-switch, and platform selection
(iOS/Android/macOS) are enforced by the push platform. Users who already played today
SHALL NOT be reminded. Windows/Linux users (no push token) SHALL NOT receive it and
SHALL keep the in-app streak cue.

#### Scenario: At-risk user is reminded
- **WHEN** the reminder job runs at the configured hour and a user has a streak but has not played today
- **THEN** that user receives the streak reminder (subject to consent/flags)

#### Scenario: Already-played user is not reminded
- **WHEN** the reminder job runs and a user already played today
- **THEN** that user is not sent a reminder

#### Scenario: A user whose streak is already broken is not reminded
- **WHEN** the reminder job runs and a user's last activity is two or more of their own
  local days ago
- **THEN** that user is not sent a reminder

#### Scenario: Reminder hour is changed from the back office
- **WHEN** an admin changes the reminder-hour flag
- **THEN** subsequent sends use the new local hour without a deploy

### Requirement: One definition of a streak across the app

This capability SHALL count a streak day exactly as the `Consistency` badge family
does: a local day on which the user played **or** practised. It SHALL NOT declare a
badge family of its own — the achievement registry already declares the streak
badges and derives its own `longest_streak` from the session tables — so the chip,
the heatmap and the badge grid never disagree about what a day of activity is.

#### Scenario: The chip and the badge agree on a practice-only day
- **WHEN** a user practises (but does not play) on a day
- **THEN** both the app-bar streak and the consistency badge counter treat that day as active

#### Scenario: No duplicate streak badge is declared here
- **WHEN** the badge grid is rendered
- **THEN** the streak badges shown are the registry's, declared exactly once
