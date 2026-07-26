## ADDED Requirements

### Requirement: Levels derived from points

The system SHALL derive a user's **level** from their accumulated points against a set of
configured thresholds. A user's level SHALL only ever rise as points accrue (it does not
fall), and the thresholds SHALL be configuration.

#### Scenario: Crossing a threshold raises the level

- **WHEN** a user's accumulated points cross a configured level threshold
- **THEN** their level increases to that tier

#### Scenario: Level does not regress

- **WHEN** a user's points are unchanged or only increase over time
- **THEN** their level never decreases

### Requirement: Pianos and badges unlocked by level

Reward tiers SHALL unlock content: **pianos/SoundFonts** become available to the user at
configured levels (integrating with the existing piano-selection catalog), and **badges**
are granted at defined milestones (for example a first set of ratings, a count of aligned
ratings, or coverage of rarely-rated scores). Unlocks SHALL be **durable** (once unlocked,
a piano stays available; once earned, a badge is kept). **Temporary premium access** is
declared as a **future** tier and is NOT granted by this change.

#### Scenario: Reaching a level unlocks a piano

- **WHEN** a user reaches a level configured to unlock a piano
- **THEN** that piano becomes available to select in the app and stays available thereafter

#### Scenario: Milestone grants a badge

- **WHEN** a user meets a badge milestone (e.g. a number of aligned ratings)
- **THEN** the badge is granted and retained

#### Scenario: Premium tier is not granted yet

- **WHEN** a user reaches the tier reserved for temporary premium
- **THEN** no premium access is granted (the tier is declared for the future, not implemented)

### Requirement: Progress surfaced in the app

The app SHALL show the signed-in user their **points, current level, next unlock and
progress toward it, and their badges**, so the reward loop is visible and motivating.
This surface SHALL be driven through injectable state so it is testable without the native
library or a live backend.

#### Scenario: User sees points and next unlock

- **WHEN** a signed-in user opens their reward/progress surface
- **THEN** they see their points, level, the next unlock, and progress toward it

#### Scenario: Earned badges are shown

- **WHEN** a user has earned badges
- **THEN** those badges are displayed on their progress surface
