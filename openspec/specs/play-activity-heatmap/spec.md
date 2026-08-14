# play-activity-heatmap Specification

## Purpose
TBD - created by archiving change add-play-activity-profile. Update Purpose after archive.
## Requirements
### Requirement: Daily play heatmap on the profile

The profile SHALL show a GitHub-style contribution grid of the player's play activity, with
**one cell per day**. Each day a player played, the cell SHALL be **colored by that day's
average overall synchronization percentage** (the session success score from
`performance-scoring`, the color weighting requested), and the day's **play count** SHALL be
conveyed by the cell (intensity/size and a tooltip showing count and the exact average
percentage). Days with no play SHALL render as empty cells. The grid SHALL be driven through
injectable state so it is testable without the native library or a live backend.

#### Scenario: Cell color reflects the day's success score

- **WHEN** a day has recorded sessions
- **THEN** its cell is colored according to that day's average overall synchronization percentage

#### Scenario: Count is conveyed alongside color

- **WHEN** a user inspects a played day's cell
- **THEN** the number of songs played that day (and the exact average percentage) is available via the cell's intensity and tooltip

#### Scenario: Empty days are blank

- **WHEN** a day has no recorded sessions
- **THEN** its cell renders empty

#### Scenario: Grid reflects synced activity

- **WHEN** new sessions have been recorded and aggregated
- **THEN** the heatmap reflects them on the corresponding days

### Requirement: Practice count on the daily grid

The daily activity grid SHALL convey, for each day, a **practice count** — the number of
practice (selective) sessions that day — **distinct from** the scored-play count. Because a
practice carries **no synchronization percentage**, it SHALL NOT contribute to the day's success
**color**: a day with only practice (no scored plays) SHALL render as a neutral "active" cell,
**not** as a low-score/failure color. The practice count SHALL be available through the cell's
intensity and/or tooltip alongside the scored-play information.

#### Scenario: A practice-only day is neutral, not a failure color
- **WHEN** a day has practice sessions but no scored plays
- **THEN** its cell renders as a neutral active day and is not colored as a low success score

#### Scenario: Practice count is conveyed on the cell
- **WHEN** a user inspects a day that had practice sessions
- **THEN** the number of practice sessions that day is available via the cell's intensity/tooltip,
  distinct from the scored-play count

#### Scenario: Practice does not change the day's success color
- **WHEN** a day has both scored plays and practice sessions
- **THEN** the day's color reflects only the scored plays' average synchronization percentage,
  and the practice count is shown separately

#### Scenario: Grid reflects synced practice activity
- **WHEN** new practice records have been delivered and aggregated
- **THEN** the heatmap reflects the practice counts on the corresponding days

