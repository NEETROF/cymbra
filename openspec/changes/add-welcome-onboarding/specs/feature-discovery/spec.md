## ADDED Requirements

### Requirement: Progressive first-use coaching

The app SHALL introduce each feature with a **one-time**, **dismissible** coaching hint shown at
the **first relevant use** of that feature, through a single shared mechanism, rather than a
large upfront tour. A hint's "seen" state SHALL be persisted so it is not shown again, and the
hint MUST NOT block the underlying action. The first-run hints for the rating deck, reward
feedback, and the public-sharing age gate SHALL be delivered through this shared mechanism.

#### Scenario: A feature's hint appears at first use

- **WHEN** the user reaches a feature for the first time
- **THEN** a dismissible coaching hint introduces it, without blocking the action

#### Scenario: A hint is shown only once

- **WHEN** the user has already seen and dismissed a feature's hint
- **THEN** it is not shown again on later uses

#### Scenario: Hints do not block

- **WHEN** a coaching hint is visible
- **THEN** the user can still perform the underlying action and dismiss the hint

### Requirement: Discovery is re-findable via help/tips

The app SHALL provide a help/tips surface, reachable from a stable entry point, where a user can
re-read how the app's systems work (the core loop, ratings, points, shop and badges, the
profile, leaderboards, and going public). This lets users recover explanations that were shown
once as first-use hints.

#### Scenario: User re-reads how a system works

- **WHEN** a user opens the help/tips surface
- **THEN** they can read explanations of the app's systems, including ones previously shown as one-time hints

#### Scenario: Help is reachable from a stable place

- **WHEN** a user looks for help after dismissing the first-use hints
- **THEN** the help/tips surface is reachable from a stable entry point

### Requirement: Onboarding copy is localized and accessible

All welcome, coaching, and help copy SHALL be localized through the app's localization system and
SHALL be accessible (dismissible without relying on a single gesture, adequate contrast, screen-
reader friendly), consistent with the app's responsive/landscape layout.

#### Scenario: Copy follows the app language

- **WHEN** the app is used in a supported language
- **THEN** the welcome, coaching, and help copy appear in that language

#### Scenario: Hints are dismissible accessibly

- **WHEN** a user relies on assistive input
- **THEN** they can dismiss a coaching hint without depending on a single specific gesture
