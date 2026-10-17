# Spec Delta

## Purpose

The Music home has two explicit doors, Play and Learn, so a player chooses between playing
freely and following the piano learning path, and the app remembers that choice.

## ADDED Requirements

### Requirement: The home offers a Play door and a Learn door
The home SHALL present a persistent, always-visible switcher with two doors, **Play** and
**Learn**. *Play* SHALL show the library as it exists without this change (favorites by
level and the entry points to the hub, rating deck and community). *Learn* SHALL show the
piano learning path: the continue card and the full path with its units and lesson nodes,
with a reserved slot above them for a daily session. Switching doors SHALL NOT navigate
away from the home.

#### Scenario: Play shows the library
- **WHEN** the player selects the Play door
- **THEN** the home body shows the favorites grouped by level and the library entry points, with no courses card above them

#### Scenario: Learn shows the path
- **WHEN** the player selects the Learn door
- **THEN** the home body shows the continue card and the learning path with its units and lesson nodes

#### Scenario: Learn works signed out
- **WHEN** a guest selects the Learn door
- **THEN** the path and the lessons open exactly as they do for a guest today, with completion kept locally until sign-in

### Requirement: The chosen door is remembered
The app SHALL persist the last door the player chose and SHALL reopen the home on it, on
the same device, across restarts. The first launch SHALL open on Play.

#### Scenario: The door survives a restart
- **WHEN** the player selects Learn and later relaunches the app
- **THEN** the home opens on the Learn door

#### Scenario: First launch opens on Play
- **WHEN** the app is launched for the first time on a device
- **THEN** the home opens on the Play door

### Requirement: The Learn door is keyboard-only
While the instrument context is drums, the switcher SHALL offer Play only and the home SHALL
behave as it does today under drums. Returning to the keyboard context SHALL restore the
Learn door and the remembered choice.

#### Scenario: Drums hide the Learn door
- **WHEN** the instrument context is drums
- **THEN** the switcher shows Play only and no learning content is shown on the home

#### Scenario: Keyboard restores the remembered door
- **WHEN** the player returns to the keyboard context after having chosen Learn earlier
- **THEN** the home reopens on the Learn door

### Requirement: Doors and their destinations transition with motion
Switching doors SHALL animate the body change (a fade combined with a short slide), and
opening the path, a lesson or a piece from a card SHALL animate from that card into the
destination. When the platform requests reduced motion, these transitions SHALL complete
without animation.

#### Scenario: Switching doors animates
- **WHEN** the player switches from Play to Learn
- **THEN** the previous body fades and slides out while the new body fades and slides in, and both doors stay interactive during the transition

#### Scenario: Reduced motion disables the transitions
- **WHEN** the platform reports reduced motion
- **THEN** door switches and card openings complete immediately with no animated frames

### Requirement: The two-door home can be switched off from the server
The two-door home SHALL be governed by the `music.learn_door.enabled` runtime flag,
defaulting to on. When the flag is off, the switcher SHALL be absent and the home SHALL
show the single body it had before this change, including the courses card above the
favorites.

#### Scenario: Flag off restores the single home
- **WHEN** `music.learn_door.enabled` evaluates to off for the caller
- **THEN** no switcher is shown and the home shows the favorites with the courses card above them

#### Scenario: A missing flag means on
- **WHEN** the flag is absent from the caller's effective flags
- **THEN** the two-door home is shown
