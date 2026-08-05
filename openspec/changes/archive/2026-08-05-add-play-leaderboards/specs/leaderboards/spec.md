## ADDED Requirements

### Requirement: Per-piece, per-mode leaderboards

The system SHALL maintain, for each **validated (`accepted`) catalog score**, two
leaderboards: a **tempo** board fed by the free-run synchronization sub-score and a
**reaction** board fed by the Wait-Mode synchronization sub-score. A run that produced a
given mode's sub-score SHALL be eligible for that mode's board; a `mixed` run (both
sub-scores) SHALL be eligible for both, a pure run for one. User uploads (owner-private
scores) SHALL NOT have shared leaderboards.

#### Scenario: A run feeds the matching mode board(s)

- **WHEN** a run finalizes with a tempo sub-score, a reaction sub-score, or both
- **THEN** it is eligible for the tempo board, the reaction board, or both, for that piece

#### Scenario: Only accepted catalog scores have boards

- **WHEN** a piece is a `pending`/`rejected` catalog score or an owner-private upload
- **THEN** it has no shared leaderboard

### Requirement: Monotonic personal best per player, piece, and mode

The system SHALL keep, per (player, piece, mode), the player's **best** result — the highest
sub-score with its tie-break metric and the time achieved. On each ingested session the best
SHALL be updated **only if the new result is better**, so the update is monotonic and
**idempotent under at-least-once ingestion**: a replayed or duplicated session MUST NOT lower
a best or create a duplicate. Boards SHALL be derived from these stored bests so they do not
depend on retained raw session detail.

#### Scenario: A better result raises the personal best

- **WHEN** a player records a session whose sub-score beats their stored best for that piece and mode
- **THEN** their personal best is updated to the new result

#### Scenario: A worse or replayed result does not change the best

- **WHEN** a session with a lower sub-score, or a duplicate of an already-counted session, is ingested
- **THEN** the personal best is unchanged and no duplicate best is created

#### Scenario: Best survives detail pruning

- **WHEN** the raw detailed session behind a best is later pruned by retention
- **THEN** the personal best remains available for ranking

### Requirement: Ranking by sub-score with a defined tie-break

The system SHALL rank a board in descending order of the per-mode synchronization sub-score,
breaking ties by the mode's timing quality — smaller mean tempo offset on the tempo board,
faster mean reaction time on the reaction board — and then by earliest achieved. The ranking
metric and tie-break order SHALL be configuration.

#### Scenario: Higher sub-score ranks above lower

- **WHEN** two players have different best sub-scores on a board
- **THEN** the higher sub-score is ranked above the lower

#### Scenario: Ties broken by timing then recency

- **WHEN** two players have equal best sub-scores
- **THEN** the one with the better timing metric ranks higher, and if still equal the one who achieved it earlier

### Requirement: Public listing gated by profile visibility and age eligibility

A leaderboard listed to **other** players SHALL include only players whose profile is
**public and age-eligible** per the profile visibility and minimum-age safeguard; a private
or ineligible player MUST NOT be listed to others. **Viewing** a leaderboard, however, SHALL
be available to **any authenticated user** — including under-age or private users — since it
only reads already-public entries; being under-age or private restricts *appearing* on a
board, not *reading* one. The system SHALL nonetheless always return to a caller **their own**
personal best and **their own rank** among the public entries. Leaderboard entries SHALL never
expose a sensitive field (no email, curator alignment/reliability, or moderation state).

#### Scenario: Private player is not listed to others

- **WHEN** another player views a board that includes a private or age-ineligible player
- **THEN** that player is not shown in the listing

#### Scenario: Player always sees their own rank and best

- **WHEN** a private or ineligible player views a board they have played
- **THEN** they see their own personal best and their own rank among the public entries

#### Scenario: Under-age user may view but is not listed

- **WHEN** an under-age (or private) authenticated user opens a leaderboard
- **THEN** they can read the ranked public entries, but they themselves do not appear in the listing shown to others

#### Scenario: No sensitive fields on a board

- **WHEN** any leaderboard listing is returned
- **THEN** it contains no email, curator alignment/reliability, or moderation state

### Requirement: Basic score integrity checks

The system SHALL validate an ingested result against cheap server-side invariants before it
becomes board-eligible: sub-scores within range, onset counts consistent with the piece, and
timing metrics within plausible bounds. A result that fails SHALL be excluded from the boards
(it may still be stored as a session) and SHALL be logged. Robust anti-cheat is not required
by this change.

#### Scenario: Impossible result excluded from boards

- **WHEN** an ingested result violates a server-side invariant (e.g. an out-of-range sub-score)
- **THEN** it does not affect any leaderboard and the rejection is logged

#### Scenario: Valid result is board-eligible

- **WHEN** an ingested result passes the integrity checks
- **THEN** it is eligible to update the relevant personal best and boards
