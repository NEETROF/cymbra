## MODIFIED Requirements

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

The offer SHALL be **reachable on the user's own initiative** for as long as the
server allows the recovery — not only at the moment the app chooses to raise it.
The recovery window closes on the next play, because resuming restarts the run
and leaves nothing to restore, so an offer the user can reach only through an
unsolicited prompt is one they lose by doing the thing they opened the app to do.

A declined offer SHALL be remembered **per break**, keyed to the run a recovery
would restore, so the same question is not re-opened on the next launch, nor when
the user reaches another screen that hosts the streak listener. It SHALL NOT be
keyed to the local day: the grace window is a back-office flag
(`streak.grace_days`), so a window wider than one day would re-ask the same
question every morning — a configuration change must not be able to turn a
refusal into a daily prompt.

Declining SHALL silence the prompt for that break and SHALL NOT withdraw the
offer: the user is answering "stop asking", not "forfeit the streak", and the
recovery stays reachable until the server refuses it.

#### Scenario: A declined offer stays declined
- **WHEN** a user answers "not this time" and later relaunches the app while the same
  break is still inside the grace window
- **THEN** the confirmation is not shown again and nothing is debited

#### Scenario: A declined offer is still reachable
- **WHEN** a user has declined the prompt for a break the server still allows
  recovering
- **THEN** the recovery remains available from the streak surface, with its cost

#### Scenario: A wider grace window does not re-ask
- **WHEN** `streak.grace_days` is configured wider than one day and a user declines
  an offer, then relaunches on a later day with the same break still recoverable
- **THEN** the question is not re-opened, because the refusal is keyed to the break
  rather than to the day

#### Scenario: A new break is a new question
- **WHEN** a user declines one break's offer, later rebuilds a streak, and breaks it
  again
- **THEN** the offer for the new break is raised, because it is a different run

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

### Requirement: Streak shown in the app-bar standing chip

The app SHALL show the current streak in the standing chip, as a flame and the day
count, on every surface that hosts the standing pill. It SHALL be present at zero
rather than hidden, rendered muted, so a user who has not started a streak can see
that there is one to start.

The chip SHALL be the **entry point to the streak**: activating it SHALL open a
surface stating the current standing, and — whenever the server allows a recovery
— offering it with its point cost behind the same explicit confirmation any other
spend uses. The chip SHALL respond whether or not a recovery is available, so the
control does not teach users that it usually does nothing.

#### Scenario: The streak is visible at a glance
- **WHEN** a user is on a surface hosting the standing pill
- **THEN** the chip shows the flame and the current day count

#### Scenario: Zero is shown, not hidden
- **WHEN** a user has no streak
- **THEN** the chip renders muted with zero rather than disappearing

#### Scenario: The chip opens the streak surface
- **WHEN** a user activates the chip
- **THEN** the streak surface opens and states the current standing

#### Scenario: The recovery is offered where the streak is
- **WHEN** a user activates the chip while a break is recoverable
- **THEN** the surface offers the recovery with its cost, and spending still
  requires an explicit confirmation
