## ADDED Requirements

### Requirement: Daily free-open quota

A user SHALL be able to fully open (receive the MusicXML of) up to **N distinct
catalog pieces per local day** for free, where N is a runtime-configurable value
(a feature flag, tunable without deploy, and settable to 0 to disable the gate).
The set of catalog pieces a user has opened during the current day SHALL be
remembered. **Re-opening a piece already in that day's set SHALL be free** and
SHALL NOT consume any quota. The day boundary SHALL be the caller's local date,
computed from the client UTC offset (the same convention as play-activity dates).

The open decision SHALL be a pure, host-testable function whose inputs are the
piece id, the day's opened-set, the free quota, the free-used count, and whether
the caller is a subscriber — so each branch is unit-tested without a DB.

#### Scenario: First opens of the day are free up to the quota
- **WHEN** a user opens their 1st … Nth distinct catalog piece on a given local day
- **THEN** each is served (MusicXML delivered) and added to the day's opened-set

#### Scenario: Re-opening a piece opened earlier today is free
- **WHEN** a user re-opens a piece already in today's opened-set (e.g. after leaving it)
- **THEN** it is served for free and does not consume a free slot

#### Scenario: The set resets at local midnight
- **WHEN** the caller's local date advances to a new day
- **THEN** the opened-set is empty again and the free quota is fully available

#### Scenario: Quota reached without points or subscription
- **WHEN** a user has used all N free opens and opens an (N+1)th distinct piece, with no subscription and insufficient points
- **THEN** the MusicXML is refused and the piece is presented as locked

### Requirement: Points unlock an extra piece for the day

Opening a distinct piece **beyond** the free quota SHALL cost curation points, at
open time and only after an **explicit user confirmation**. A successful unlock
SHALL, atomically: write a points **ledger debit** for the cost (referencing the
piece and the day), and record the piece into today's opened-set as a **paid**
slot. The access so obtained is **consumable for the current day only** — it SHALL
NOT create a permanent ownership grant, so the same piece is free to re-open for
the rest of that day but costs again on a later day. The unlock SHALL be refused if
the caller's spendable balance is insufficient.

#### Scenario: Paying points unlocks the piece for today
- **WHEN** an over-quota user confirms spending the day-slot cost and has enough balance
- **THEN** a ledger debit is recorded, the piece is served, and it is added to today's set as paid

#### Scenario: Re-opening a paid piece the same day is free
- **WHEN** the user re-opens a piece they paid to unlock earlier today
- **THEN** it is served with no further points spent

#### Scenario: The same piece costs again the next day
- **WHEN** the user opens, on a later day, a piece they had paid to unlock previously
- **THEN** it counts against the new day's quota / requires points again (no permanent grant)

#### Scenario: Insufficient balance is refused
- **WHEN** an over-quota user tries to unlock a piece but lacks the required points
- **THEN** no debit is written and the piece stays locked

### Requirement: Subscription bypass seam and upsell hook

The open gate SHALL consult `has_active_subscription(caller)`. A subscriber SHALL
have **unlimited** opens (the daily quota does not apply). The subscription system
does not exist yet, so this check SHALL currently return false; the seam MUST be in
place so future billing enables it by implementing one function without changing the
quota logic. When the quota is reached / an unlock is offered, the response SHALL
carry an **upsell** signal (a placeholder now, no-op) so that, once subscriptions
exist, the client can nudge the user to subscribe at those moments without a
contract change.

#### Scenario: A subscriber bypasses the quota
- **WHEN** `has_active_subscription` returns true for the caller
- **THEN** any piece is served regardless of the daily quota or points balance

#### Scenario: Seam is inert until billing exists
- **WHEN** no subscription system is deployed
- **THEN** `has_active_subscription` returns false and the quota applies normally

#### Scenario: Quota-reached response carries the upsell signal
- **WHEN** a non-subscriber reaches the quota and is offered the points unlock
- **THEN** the response includes an upsell signal the client may act on (currently a placeholder)
