# music-score-daily-access Specification

## Purpose
TBD - created by archiving change add-score-daily-access-rewards. Update Purpose after archive.
## Requirements
### Requirement: Daily free-open quota on the catalog player-open

A user SHALL be able to fully open (receive the MusicXML of) up to **N distinct
catalog pieces per day** for free, where N is a runtime-configurable value read at
call time through the feature-flag platform (tunable without deploy, settable to 0).
The whole gate SHALL sit behind a boolean flag that is **off by default** and, when
off, SHALL leave every catalog open served as before while retaining stored data. The
day boundary SHALL be the **server day**, the same clock as the play-award daily cap;
the access state SHALL carry the next reset instant so the app can show it. The set of
catalog pieces a user has opened during the current day SHALL be remembered.
**Re-opening a piece already in that day's set SHALL be free** and SHALL NOT consume
any quota, and a served open SHALL be recorded idempotently.

The open decision SHALL be a pure, host-testable function whose inputs are the piece
id, the day's state (opened-set, free-used count), the configuration (enabled, quota,
cost) and the caller kind (regular / exempt / subscriber / contributor) — so each branch
is unit-tested without a DB.

#### Scenario: First opens of the day are free up to the quota
- **WHEN** a user opens their 1st … Nth distinct catalog piece on a given server day
- **THEN** each is served (MusicXML delivered) and added to the day's opened-set

#### Scenario: Re-opening a piece opened earlier today is free
- **WHEN** a user re-opens a piece already in today's opened-set (e.g. after leaving it)
- **THEN** it is served for free and does not consume a free slot

#### Scenario: The set resets on the server day
- **WHEN** the server day advances
- **THEN** the opened-set is empty again and the free quota is fully available

#### Scenario: Quota reached without points or subscription
- **WHEN** a user has used all N free opens and opens an (N+1)th distinct piece, with no subscription and insufficient points
- **THEN** the MusicXML is refused and the piece is presented as locked

#### Scenario: Gate off serves everything and keeps data
- **WHEN** the gate flag is off
- **THEN** every catalog open is served regardless of quota, and previously recorded day-access rows are retained

#### Scenario: Reset time is exposed
- **WHEN** the app reads the caller's access state
- **THEN** it includes the instant at which the day's quota resets

### Requirement: Locked answer is a state, not an error, and never a cache hit

When a catalog player-open is refused by the quota, the bytes response SHALL return
**no MusicXML** and SHALL NOT report the content as unchanged; it SHALL carry an
explicit **access state** (locked, free quota / used, day-slot cost, caller's spendable
balance, reset instant, subscriber and upsell signals) as data rather than a gRPC
error, so the client branches on state and an offline cache never mistakes it for a
transient failure. A locked open SHALL NOT record the coverage engagement signal (no
bytes were served, nothing was played).

#### Scenario: Locked response carries state and no bytes
- **WHEN** an over-quota user opens a distinct piece
- **THEN** the response has empty data, is not marked unchanged, and its access state says locked with the cost and reset instant

#### Scenario: Conditional fetch of a locked piece is still locked
- **WHEN** an over-quota user requests a piece with a matching content hash (conditional fetch)
- **THEN** the response is the locked state, not "unchanged"

#### Scenario: Locked open records no engagement
- **WHEN** a piece is refused as locked
- **THEN** no coverage engagement is recorded for that (user, piece)

### Requirement: Access state is readable in one call

The system SHALL expose a read of the caller's daily access state — enabled, free
quota and used count, reset instant, day-slot cost, spendable balance, subscriber and
upsell signals, the ids opened today and which of them were paid — so the catalog
surfaces can show remaining free opens, mark opened-today pieces and show the unlock
cost without probing each piece's bytes.

#### Scenario: Hub reads the state once
- **WHEN** the app opens the hub or library
- **THEN** one call returns the caller's quota, used count, reset instant, cost and today's opened ids

### Requirement: Points unlock an extra piece for the day

Opening a distinct piece **beyond** the free quota SHALL cost curation points, at
open time and only after an **explicit user confirmation** — never a silent debit.
A successful unlock SHALL, atomically: write a points **ledger debit** for the cost
(`redeem` kind, a `score_day_slot` reward key, the piece id, and a ledger idempotency
key derived from the piece and the day), and record the piece into today's opened-set
as a **paid** slot. The charge-once guarantee SHALL be enforced by the ledger's
idempotency key, not by the read-then-write decision alone. The access so obtained is
**consumable for the current day only** — it SHALL NOT create a permanent ownership
grant, so the same piece is free to re-open for the rest of that day but costs again
on a later day. The unlock SHALL be refused, writing nothing, if the caller's
spendable balance is insufficient.

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

#### Scenario: Two confirmations for the same piece and day charge once
- **WHEN** two unlock confirmations for the same piece on the same day reach the server at once
- **THEN** exactly one points debit is written and the piece is open for today

#### Scenario: No silent debit
- **WHEN** a user reaches the quota
- **THEN** no points are spent unless they explicitly confirm the unlock

### Requirement: Audiences outside the quota

The quota SHALL apply only to the authenticated **player-open of a catalog piece**.
It SHALL NOT apply to: bundled scores and a user's own uploads (which never go through
the catalog bytes path); the back-office audience and music-scope moderators/admins
(reviewing is not consumption); the **contributor** of an accepted user-proposed piece
opening their own contribution from the catalog; and the rating deck's preview bytes
and catalog metadata reads, which stay ungated. The abuse rate limits of
`catalog-access-limits` SHALL keep running **before** the quota decision.

#### Scenario: Moderator and back office are not quota'd
- **WHEN** a music-scope moderator/admin or a back-office caller opens catalog bytes
- **THEN** it is served without consuming or checking the daily quota

#### Scenario: A contributor opens their own accepted piece free
- **WHEN** the proposer of an accepted user-proposed piece opens it from the catalog
- **THEN** it is served without consuming a free slot or costing points

#### Scenario: Metadata and deck stay open
- **WHEN** a user searches the catalog, reads a piece's metadata or previews a deck card
- **THEN** the daily quota is neither checked nor consumed

#### Scenario: Abuse cap fires first
- **WHEN** a user exceeds the download burst or volume cap
- **THEN** the request is rejected as resource-exhausted before any quota decision

### Requirement: Online the server decides before a cached favourite plays

The app SHALL, when **online** and holding a cached (favourited) copy of a catalog
piece, ask the server (a conditional fetch by content hash) **before** playing from
the cache and SHALL treat that answer as the access decision: unchanged or served → play
(and refresh as before); locked → present the locked flow and do NOT play the cached
copy (which is kept, since access is per-day). When **offline** (no network / server
unreachable), a cached favourite SHALL remain playable — the offline grace is an
accepted, documented soft-limit trade-off.

#### Scenario: Online cached favourite is gated
- **WHEN** an over-quota user opens a cached favourite while online
- **THEN** the server answers locked, the app shows the locked flow and does not play the cached bytes

#### Scenario: Online cached favourite within quota plays and counts
- **WHEN** a user within quota opens a cached favourite while online
- **THEN** the server records today's open, answers unchanged, and the app plays from cache

#### Scenario: Offline cached favourite still plays
- **WHEN** the app is offline and the user opens a cached favourite
- **THEN** it plays from the local encrypted copy without a network request

### Requirement: Subscription bypass seam and upsell hook

The open gate SHALL consult a subscription seam (`has_active_subscription(caller)`).
A subscriber SHALL have **unlimited** opens (the daily quota does not apply). The seam
SHALL be implemented over the plan entitlements: `has_active_subscription(caller)` is
true exactly when the caller's **effective plan grants the `catalog.unlimited` unlock**
(i.e. the effective plan is `premium`, whatever its source — store, web, trial or admin).
The quota logic MUST NOT know plan names or beta campaigns — it consumes the seam only.
When the quota is reached / an unlock is offered, the access state SHALL carry an
**upsell** signal that the client renders as a real, platform-appropriate call to action
to the paywall. While `plans.enabled` is off the seam SHALL answer false, exactly as
before.

#### Scenario: A subscriber bypasses the quota
- **WHEN** the caller's effective plan grants `catalog.unlimited`
- **THEN** any piece is served regardless of the daily quota or points balance

#### Scenario: A premium-trial tester bypasses the quota like a subscriber
- **WHEN** the caller holds an active premium trial row
- **THEN** any piece is served regardless of the daily quota

#### Scenario: A feature-beta member on the free plan stays under quota
- **WHEN** the caller is a member of a feature beta but has no premium row
- **THEN** the quota applies normally

#### Scenario: Seam is inert while plans are disabled
- **WHEN** `plans.enabled` is off
- **THEN** `has_active_subscription` returns false and the quota applies normally

#### Scenario: Quota-reached response carries the upsell signal
- **WHEN** a non-subscriber reaches the quota and is offered the points unlock
- **THEN** the response includes an upsell signal the client renders as a call to action to the paywall for that platform

### Requirement: Day-access data follows the account and is pruned

A user's day-access rows SHALL be deleted when their account is purged, SHALL cascade
with the deletion of the piece, and SHALL be pruned by retention once older than the
current day's usefulness (they are only ever read for the current day).

#### Scenario: Purged account leaves no day-access rows
- **WHEN** a user's account is purged
- **THEN** their day-access rows are deleted

#### Scenario: Old rows are pruned
- **WHEN** the retention prune runs
- **THEN** day-access rows older than the retention window are removed and today's rows are kept

