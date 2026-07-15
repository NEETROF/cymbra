# score-library Specification

## Purpose
TBD - created by archiving change musicxml-parsing-and-geometry. Update Purpose after archive.
## Requirements
### Requirement: Bundled Score Catalog

The app SHALL ship a catalog of multiple free / public-domain MusicXML scores
bundled as assets. Each catalog entry SHALL declare at least a stable id, a
display title, a composer, the asset path of its uncompressed `.musicxml`/`.xml`
file, and a practice level. The catalog SHALL be exposed through an injectable
provider so tests can supply a fake catalog without reading the asset bundle.

#### Scenario: Catalog lists bundled scores
- **WHEN** the app reads the score catalog
- **THEN** it returns the bundled entries, each with id, title, composer, asset
  path, and practice level

#### Scenario: Every entry's asset is bundled
- **WHEN** a catalog entry declares an asset path
- **THEN** that `.musicxml`/`.xml` file is present in the app assets and
  registered in `pubspec.yaml`

#### Scenario: Catalog overridable in tests
- **WHEN** a test overrides the catalog provider with in-memory entries
- **THEN** the library screen renders those entries without touching the asset
  bundle

### Requirement: Practice Levels

Each catalog entry SHALL carry exactly one practice level from a fixed set —
Beginner, Intermediate, Advanced. The catalog SHALL include at least one score
for each practice level.

#### Scenario: Levels cover the full range
- **WHEN** the catalog is read
- **THEN** at least one entry exists for each of Beginner, Intermediate, and
  Advanced

#### Scenario: Entry exposes its level
- **WHEN** a catalog entry is inspected
- **THEN** it reports a single practice level from the fixed set

### Requirement: Library Start Screen

The app SHALL start on a library screen that lists the catalog entries grouped or
labelled by practice level, showing each score's title, composer, and level. The
library screen SHALL be the application's initial route (`home`).

#### Scenario: App boots into the library
- **WHEN** the app launches
- **THEN** the first screen shown is the library, not the piano/partition screen

#### Scenario: Entries grouped by level
- **WHEN** the library screen renders a catalog with several levels
- **THEN** entries are presented grouped or labelled by Beginner / Intermediate /
  Advanced, each showing title and composer

### Requirement: Partition Selection And Navigation

Selecting a catalog entry on the library screen SHALL record it as the selected
score and navigate to the player screen, which SHALL load that score's MusicXML
and display it — including an engraved Partition view. The player screen SHALL
retain its on-screen piano keyboard, MIDI device selection and transport. The
selection SHALL be exposed through state so the player knows which asset to load.
Returning from the player screen SHALL bring the user back to the library.

#### Scenario: Selecting a score opens the player screen
- **WHEN** the user taps a catalog entry
- **THEN** that entry becomes the selected score and the app navigates to the
  player screen, which loads and displays its parsed notation

#### Scenario: Player screen loads the selected asset
- **WHEN** the player screen is shown for a selected entry
- **THEN** it loads the entry's asset path through the score-asset source and
  renders the resulting score (engraved partition and the derived playback views)

#### Scenario: Player retains keyboard and MIDI controls
- **WHEN** the player screen is shown for a selected entry
- **THEN** the on-screen piano keyboard, MIDI device selection and transport
  controls remain available

#### Scenario: Back returns to the library
- **WHEN** the user navigates back from the player screen
- **THEN** the library screen is shown again

### Requirement: User's Contributed Scores In Library

When a user is signed in, the library SHALL additionally present the scores that
user has contributed, fetched from the backend, as a section distinct from the
bundled catalog. Each contributed entry SHALL show its title and difficulty
level, and SHALL be selectable to open in the player like a catalog entry. When no
user is signed in, no contributed-scores section is shown. The bundled-catalog
behavior is unchanged.

#### Scenario: Contributed scores listed when signed in

- **WHEN** a signed-in user opens the library and has contributed scores
- **THEN** the library shows those scores in a section distinct from the bundled
  catalog, each with its title and difficulty level

#### Scenario: No contributed section when signed out

- **WHEN** no user is signed in
- **THEN** the library shows only the bundled catalog and no contributed-scores
  section

#### Scenario: Contributed scores isolated per user

- **WHEN** a signed-in user's contributed scores are listed
- **THEN** only that user's contributions appear

### Requirement: Open A Contributed Score In The Player

Selecting a contributed score in the library SHALL record it as the selected
score and navigate to the player, loading the score's bytes from the backend
(object store) rather than the asset bundle, and displaying it through the same
notation and playback path as bundled scores.

#### Scenario: Selecting a contributed score opens the player

- **WHEN** the user selects a contributed score
- **THEN** it becomes the selected score and the player opens, loading and
  rendering the score from its backend-provided bytes

### Requirement: Delete A Contributed Score From The Library

The library SHALL offer a delete action only for scores the signed-in user owns
(their contributed scores), and MUST NOT offer deletion for bundled-catalog
scores. Confirming deletion SHALL request removal from the backend and, on
success, remove the entry from the contributed section.

#### Scenario: Owner deletes a contributed score

- **WHEN** the owner triggers delete on one of their contributed scores and
  confirms
- **THEN** the app requests deletion from the backend and, on success, removes the
  entry from the library

#### Scenario: Bundled scores have no delete action

- **WHEN** a bundled-catalog entry is shown
- **THEN** no delete action is offered for it

