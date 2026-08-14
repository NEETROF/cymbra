## ADDED Requirements

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
