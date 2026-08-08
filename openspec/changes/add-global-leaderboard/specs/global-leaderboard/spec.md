## ADDED Requirements

### Requirement: Difficulty-weighted, best-N global season score

The system SHALL compute, per player and per mode, a **global season score** equal to the sum,
over the player's **best-N pieces in the current season** (N configurable), of that piece's
**best season sub-score** scaled to 0–1 and multiplied by a **difficulty weight** derived from
the piece. Only a player's N strongest pieces SHALL count, so playing more than N pieces adds
nothing beyond replacing a weaker contribution — a player climbs by playing **harder pieces
better**, not by volume. The best-N size and the difficulty weights SHALL be configuration.

#### Scenario: Only the best N pieces count

- **WHEN** a player has strong results on more than N pieces
- **THEN** only their best N contributions count toward the global season score

#### Scenario: Harder pieces are worth more

- **WHEN** two players have the same sub-score, one on an advanced piece and one on a beginner piece
- **THEN** the advanced-piece result contributes more to the global season score

#### Scenario: Volume alone does not raise the score

- **WHEN** a player plays many additional easy pieces without improving their best N
- **THEN** their global season score does not increase

### Requirement: Tempo and reaction global boards with tie-break

The system SHALL provide two global boards — **tempo** and **reaction** — ranking players by
their global season score for that mode in descending order. Ties SHALL be broken by the number
of contributing pieces, then by the earliest time the score was reached. A `mixed` run SHALL
contribute its tempo and reaction sub-scores to the respective mode aggregates.

#### Scenario: Ranked by global season score

- **WHEN** two players have different global season scores for a mode
- **THEN** the higher score is ranked above the lower on that mode's global board

#### Scenario: Tie broken by contributing pieces then recency

- **WHEN** two players have equal global season scores
- **THEN** the one with more contributing pieces ranks higher, and if still equal the one who reached it earlier

### Requirement: Seasons with end-of-season snapshot

The global board SHALL run in **seasons** of a configurable length (default about monthly), with
**UTC** boundaries. At the end of a season the system SHALL snapshot the final standings into a
lightweight history and start a new season with fresh accumulation. Per-piece all-time bests
(the per-piece leaderboards) SHALL NOT be reset by a season rollover — only the per-season global
accumulation rolls over.

#### Scenario: New season starts fresh

- **WHEN** a season ends and a new one begins
- **THEN** the new season's global scores start fresh while per-piece all-time bests are unchanged

#### Scenario: Final standings are snapshotted

- **WHEN** a season ends
- **THEN** its final global standings are recorded in a history that later seasons do not overwrite

### Requirement: An archived season freezes consent but not the age safeguard

When a season is snapshotted the system SHALL record, per player, whether that player was
**listable at the moment the season closed**, and SHALL use that recorded consent when the
archived season is later read — so a past ranking does not change because a player's visibility
changed afterwards. The **minimum-age safeguard** SHALL NOT be frozen with it: an archived entry
SHALL be listed only when the player was listable then **and** is age-eligible now. A caller
SHALL always see their own archived standing.

#### Scenario: Going private later does not rewrite a past season

- **WHEN** a player who was listed when a season closed makes their profile private afterwards
- **THEN** the archived season still shows them, and the other players' archived ranks are unchanged

#### Scenario: Going public later does not add someone to a past season

- **WHEN** a player who was private when a season closed makes their profile public afterwards
- **THEN** they are still absent from that archived season's listing

#### Scenario: The age safeguard still removes an archived entry

- **WHEN** a player recorded as listable is no longer age-eligible
- **THEN** they are not listed in the archived season either

### Requirement: Season best maintained monotonically on ingest

The system SHALL keep, per (player, season, piece, mode), the player's **best season sub-score**,
updated on each ingested session **only if the new sub-score is better** — a monotonic,
**idempotent** update under at-least-once ingestion (a replayed session never lowers a season
best or duplicates it). Global scores SHALL be derived from these season bests.

#### Scenario: A better in-season result raises the season best

- **WHEN** a player records a session this season whose sub-score beats their season best for that piece and mode
- **THEN** their season best is updated

#### Scenario: A replayed or worse session does not change it

- **WHEN** a worse or duplicate session is ingested
- **THEN** the season best is unchanged and not duplicated

### Requirement: Global listing gated; viewing open; own rank always

**Viewing** the global boards SHALL be available to **any authenticated user**, including
under-age or private users, since it reads already-public entries. **Being listed** on a global
board SHALL require the player's profile to be **public and age-eligible** (the profile
visibility and minimum-age safeguard); a private or ineligible player MUST NOT be listed to
others. The system SHALL always return to a caller **their own** global rank and score among the
public entries. Global entries SHALL never expose a sensitive field.

#### Scenario: Private player is not listed but can view and see themselves

- **WHEN** a private or under-age player opens a global board
- **THEN** they can read the public ranking and see their own rank and score, but are not listed to others

#### Scenario: Only public eligible players are listed

- **WHEN** a global board is shown to other players
- **THEN** only public, age-eligible players appear in the listing

#### Scenario: No sensitive fields on the global board

- **WHEN** a global board listing is returned
- **THEN** it contains no email, curator alignment/reliability, or moderation state
