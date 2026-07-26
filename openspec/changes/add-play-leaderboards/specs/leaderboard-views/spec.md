## ADDED Requirements

### Requirement: View a piece's leaderboards in the app

The app SHALL let a signed-in user view a validated piece's leaderboards, switching between
the **tempo** and **reaction** boards, showing the ranked public players and each entry's
score. The view SHALL be reachable from the score (and the profile) and SHALL be driven
through injectable state so it is testable without the native library or a live backend.

#### Scenario: Open a piece's board and switch mode

- **WHEN** a user opens a piece's leaderboard and toggles between tempo and reaction
- **THEN** the corresponding ranked public entries are shown

#### Scenario: Reachable from the score

- **WHEN** a user is on a validated score
- **THEN** they can open that score's leaderboards

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
