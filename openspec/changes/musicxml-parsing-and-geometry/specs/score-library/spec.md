## ADDED Requirements

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
