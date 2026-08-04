## ADDED Requirements

### Requirement: View a piece's leaderboards in the app

The app SHALL let a signed-in user view a validated piece's leaderboards, switching between
the **tempo** and **reaction** boards, showing the ranked public players and each entry's
score. The view SHALL be reachable from the score, from the **end-of-session summary**, and
from the profile, and SHALL be driven through injectable state so it is testable without the
native library or a live backend.

#### Scenario: Open a piece's board and switch mode

- **WHEN** a user opens a piece's leaderboard and toggles between tempo and reaction
- **THEN** the corresponding ranked public entries are shown

#### Scenario: Reachable from the score

- **WHEN** a user is on a validated score
- **THEN** they can open that score's leaderboards

### Requirement: Post-session standing in the end-of-session summary

At the end of a scored run on a validated piece, the end-of-session summary SHALL surface the
player's **standing on that piece** — their rank among the public entries and their personal
best, for the mode(s) the run produced — with a way to open the full board. This is the
primary hook to compare and replay.

#### Scenario: Summary shows the player's rank right after playing

- **WHEN** a scored run on a validated piece finishes
- **THEN** the summary shows the player's rank and personal best on that piece for the run's mode(s)

#### Scenario: Summary links to the full board

- **WHEN** the player is shown their post-session standing
- **THEN** they can open the full leaderboard for that piece from the summary

#### Scenario: Private player still sees their own standing

- **WHEN** a private or under-age player finishes a run
- **THEN** the summary still shows their own rank and personal best, without listing them to others

### Requirement: The viewer's own rank and personal best are shown

The leaderboard view SHALL always show the viewer **their own personal best** and **their own
rank** among the public entries for that piece and mode, even when the viewer's profile is
private (in which case others do not see them, but they still see themselves).

#### Scenario: Own rank shown even when private

- **WHEN** a user with a private profile opens a board they have played
- **THEN** they see their own personal best and their own rank among the public entries

#### Scenario: Own entry highlighted

- **WHEN** the viewer appears among the shown entries
- **THEN** their own entry is distinguishable from the others

### Requirement: Standing badge on score cards

A score shown as a card — in the score hub and the home/library — SHALL display a
compact leaderboard badge **only when it is an accepted catalog score** (the only
kind with a shared board): a trophy, plus the player's **best rank across the two
modes** when they are ranked. Cards for bundled scores or user uploads SHALL NOT
show the badge. Tapping the badge SHALL open the same leaderboard view as the
pre-play surface, initialised on the mode that produced the shown rank. Per-card
standings SHALL be resolved through a **single batched read** for a page of cards,
not one read per card, and SHALL refresh after a played session is delivered.

#### Scenario: Ranked player sees their rank on the card

- **WHEN** a signed-in user views a catalog score they are ranked on
- **THEN** the card shows a trophy with their best rank across the two modes

#### Scenario: Unranked catalog card shows a bare trophy

- **WHEN** a signed-in user views a catalog score they have not scored on
- **THEN** the card shows a bare trophy with no rank

#### Scenario: Non-catalog card shows no badge

- **WHEN** a card is a bundled score or a user upload
- **THEN** it shows no leaderboard badge

#### Scenario: Tapping the badge opens the board

- **WHEN** the user taps a card's leaderboard badge
- **THEN** the piece's leaderboard opens on the mode of the shown rank
