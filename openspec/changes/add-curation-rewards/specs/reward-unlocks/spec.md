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

### Requirement: Full-screen curator profile

The app SHALL provide a **full-screen** "curator profile" that shows the signed-in user:
their **current level, total points, and a progress bar toward the next level/unlock**; a
**next-unlock** card naming what unlocks next and how many points remain; a **badge grid**
showing earned badges and locked ones (locked shown with the milestone hint); and their
**personal curation stats** — number of ratings, coverage contribution, and their own
**alignment rate** (the same reliability figure the back office shows moderators, here as a
self-view). This screen SHALL be driven through injectable state so it is testable without
the native library or a live backend.

#### Scenario: Profile shows level, points and next unlock

- **WHEN** a signed-in user opens the curator profile
- **THEN** they see their level, total points, progress toward the next unlock, and the points remaining

#### Scenario: Badge grid shows earned and locked

- **WHEN** the user opens the profile
- **THEN** earned badges are shown as earned and unearned badges are shown locked with their milestone hint

#### Scenario: Personal stats include alignment rate

- **WHEN** the user opens the profile
- **THEN** it shows their rating count, coverage contribution, and their own alignment rate

### Requirement: Persistent reward indicator entry point

The app SHALL show a compact, persistent level/points indicator in the hub (and the rating
deck) app bar that opens the full-screen curator profile when tapped, so progress is
glanceable from the main surfaces.

#### Scenario: Indicator opens the profile

- **WHEN** a user taps the level/points indicator in the app bar
- **THEN** the full-screen curator profile opens

#### Scenario: Indicator reflects current standing

- **WHEN** the user's points or level change
- **THEN** the indicator reflects the updated level/points

### Requirement: Immediate and milestone reward feedback

When a user earns coverage points by rating, the app SHALL give **immediate** feedback (a
"+N" points cue on the action). When a user crosses a level, unlocks content, or earns a
badge, the app SHALL show a **celebration** moment for that event.

#### Scenario: Coverage points shown immediately on rating

- **WHEN** a user rates a score and earns coverage points
- **THEN** a "+N" points cue is shown on the rating action

#### Scenario: Level-up / unlock / badge is celebrated

- **WHEN** the user crosses a level, unlocks a piano, or earns a badge
- **THEN** the app shows a celebration for that event

### Requirement: Deferred honesty rewards are surfaced

The app SHALL surface deferred honesty awards after the fact — because the honesty bonus is
awarded later at settlement, not at rating time — via a notification cue on the profile
entry point when new awards have landed, and a **recent-activity** list in the profile
stating what was awarded and why (community consensus or a moderator decision).

#### Scenario: Notification cue for new deferred awards

- **WHEN** a user's ratings have been settled and awarded honesty points since they last looked
- **THEN** a notification cue appears on the profile entry point

#### Scenario: Activity list explains each award

- **WHEN** the user opens the profile's recent-activity list
- **THEN** each deferred award states the amount and whether it came from community consensus or a moderator decision

### Requirement: Locked unlocks are visible where they are used

Content gated behind a level SHALL be shown in the place it is used with its locked state
and the level required. In particular, the piano/SoundFont selection UI SHALL show a locked
piano with a lock affordance and the level needed to unlock it, so the reward is visible at
the point of spending.

#### Scenario: Locked piano shows its unlock level

- **WHEN** a user opens the piano selection with a piano they have not yet unlocked
- **THEN** that piano is shown locked with the level required to unlock it

#### Scenario: Unlocked piano is selectable

- **WHEN** a user has reached the level that unlocks a piano
- **THEN** that piano appears unlocked and selectable in the selection UI
