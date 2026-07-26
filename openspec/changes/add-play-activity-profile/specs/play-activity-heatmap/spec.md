## ADDED Requirements

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
