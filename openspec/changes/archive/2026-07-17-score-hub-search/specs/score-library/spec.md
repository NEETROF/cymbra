## ADDED Requirements

### Requirement: Saved Catalog Scores In Library

When a user is signed in, the library SHALL additionally present the catalog
scores that user has saved from the Score Hub, fetched from the backend, as a
section distinct from both the bundled catalog and the user's own contributions.
Each saved entry SHALL show its title, composer, and difficulty level (when
known), and SHALL be selectable to open in the player like a catalog entry. When
no user is signed in, no saved-catalog section is shown. The bundled-catalog and
contributed-scores behavior is unchanged.

#### Scenario: Saved catalog scores listed when signed in

- **WHEN** a signed-in user opens the library and has saved catalog scores
- **THEN** the library shows those scores in a section distinct from the bundled
  catalog and the contributed-scores section, each with its title, composer, and
  difficulty level

#### Scenario: No saved section when signed out

- **WHEN** no user is signed in
- **THEN** the library shows no saved-catalog section

#### Scenario: Saved scores isolated per user

- **WHEN** a signed-in user's saved catalog scores are listed
- **THEN** only that user's saved scores appear

### Requirement: Open A Saved Catalog Score In The Player

Selecting a saved catalog score in the library SHALL record it as the selected
score and navigate to the player, loading the score's bytes from the backend
(object store, public-corpus prefix) rather than the asset bundle, and displaying
it through the same notation and playback path as bundled and contributed scores.

#### Scenario: Selecting a saved catalog score opens the player

- **WHEN** the user taps a saved catalog score in the library
- **THEN** that score becomes the selected score and the app navigates to the
  player, which loads its bytes from the backend and renders the notation

#### Scenario: Player retains keyboard and MIDI controls

- **WHEN** the player is shown for a saved catalog score
- **THEN** the on-screen piano keyboard, MIDI device selection and transport
  controls remain available

### Requirement: Remove A Saved Catalog Score From The Library

The library SHALL let the signed-in user remove a saved catalog score from their
library directly from the saved-catalog section, removing it through the backend
so it no longer appears in the section. Removing a saved score SHALL NOT delete
the public catalog entry and SHALL NOT affect the bundled catalog or contributed
scores.

#### Scenario: Removing a saved score drops it from the section

- **WHEN** the user removes a saved catalog score from the library
- **THEN** the score is removed through the backend and no longer appears in the
  saved-catalog section

#### Scenario: Removal leaves the public catalog intact

- **WHEN** the user removes a saved catalog score
- **THEN** only the user's save is removed; the public catalog entry remains and can
  be found and saved again from the Score Hub

### Requirement: Score Hub Entry Point From The Library

The library SHALL expose an entry point to the Score Hub, available only when a
user is signed in (hidden or disabled otherwise), mirroring the contribution
entry point. Activating it SHALL open the Score Hub screen.

#### Scenario: Hub entry point available when signed in

- **WHEN** a signed-in user views the library
- **THEN** a Score Hub entry point is available and opens the hub screen

#### Scenario: Hub entry point hidden when signed out

- **WHEN** no user is signed in
- **THEN** the library shows no Score Hub entry point
