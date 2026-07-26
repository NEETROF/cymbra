## ADDED Requirements

### Requirement: Community leaderboards destination screen

The app SHALL provide a dedicated Community/Leaderboards screen — a top-level destination —
showing the global **tempo** and **reaction** boards with a mode toggle and a **season
selector** (current plus past snapshotted seasons). The screen SHALL be driven through
injectable state so it is testable without the native library or a live backend.

#### Scenario: Open the global boards and switch mode

- **WHEN** a user opens the Community/Leaderboards screen and toggles tempo/reaction
- **THEN** the corresponding global ranking is shown

#### Scenario: Browse a past season

- **WHEN** a user selects a past season in the season selector
- **THEN** that season's snapshotted standings are shown

### Requirement: Global-rank standing on the profile

The profile SHALL show the player their **global standing** — their current-season global rank
and score (per mode) — as an entry into the Community/Leaderboards screen. A private or under-age
player SHALL still see **their own** global standing (they are simply not listed to others).

#### Scenario: Profile shows the player's global rank

- **WHEN** a user opens their profile
- **THEN** they see their current-season global rank and score, linking to the Community/Leaderboards screen

#### Scenario: Private player still sees their own standing

- **WHEN** a private or under-age player opens their profile
- **THEN** they see their own global rank and score, without being listed to others
