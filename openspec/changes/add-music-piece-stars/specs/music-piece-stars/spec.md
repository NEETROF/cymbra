# Spec Delta

## Purpose

A piece shows one to three stars derived from the player's best overall synchronisation on
it, so progress on a piece is visible wherever the piece appears and a next star is always
in view.

## ADDED Requirements

### Requirement: Stars derive from the best overall synchronisation
A piece SHALL show three stars filled according to the player's best overall
synchronisation percentage on it: one star from 60 %, two from 80 %, three from 95 %, none
below 60 %. The best SHALL be the one already kept for the player's standings, so stars
SHALL never decrease. A piece with no scored run SHALL show three empty stars.

#### Scenario: Thresholds
- **WHEN** the best is 59, 60, 79, 80, 94 or 95 %
- **THEN** the stars shown are 0, 1, 1, 2, 2 and 3 respectively

#### Scenario: A lower run changes nothing
- **WHEN** a player with two stars on a piece ends a run at 65 %
- **THEN** the piece still shows two stars

#### Scenario: An unplayed piece invites
- **WHEN** a piece has no scored run for the player
- **THEN** it shows three empty stars

### Requirement: Stars appear wherever the piece does
Stars SHALL be shown on score cards in the library and the hub, on the learning path's
piece nodes, on the pieces of today's session and in the end-of-run summary. The summary
SHALL show the stars reached by the run that just ended. Signed out, cards SHALL show no
stars.

#### Scenario: The card shows the stars
- **WHEN** a signed-in player views a score card for a piece with a best of 85 %
- **THEN** the card shows two filled stars

#### Scenario: The summary shows this run's stars
- **WHEN** a run ends at 96 %
- **THEN** the summary shows three stars for that run

#### Scenario: A guest sees no stars
- **WHEN** the player is signed out
- **THEN** score cards show no star row

### Requirement: A new star raises the celebration cue once
When a run's stars exceed the stars of the player's best known before the run, the app
SHALL raise the celebration cue once for that run. A run that does not add a star SHALL
raise none.

#### Scenario: First run earns stars
- **WHEN** a player with no standing on a piece ends a run at 82 %
- **THEN** the cue is raised for two new stars

#### Scenario: Matching the best adds nothing
- **WHEN** a player with two stars ends a run at 83 %
- **THEN** no new-star cue is raised

### Requirement: Stars refresh when the run is delivered
Stars on cards and nodes SHALL update once the just-played session is delivered to the
server, without a manual refresh. Practice runs over a measure range SHALL NOT affect
stars.

#### Scenario: Cards update after delivery
- **WHEN** a run that raises the best is delivered
- **THEN** the piece's card and path node show the new stars on their next display

#### Scenario: Practice leaves stars alone
- **WHEN** the player drills a measure range
- **THEN** the piece's stars are unchanged and the practice summary shows no star row
