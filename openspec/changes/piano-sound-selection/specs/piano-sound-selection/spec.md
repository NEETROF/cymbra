## ADDED Requirements

### Requirement: Selectable Piano SoundFonts

The app SHALL offer a catalog of two or more piano SoundFonts — including at least
one bundled, cleanly-licensed default available without a network — and SHALL let
the user select which one the synthesizer uses. The catalog MAY include bundled,
download-on-first-use, and user-imported pianos. Selecting a piano SHALL apply at
runtime so that subsequent notes — from the on-screen keyboard, the computer
keyboard, a MIDI device, or score playback — sound with the chosen timbre, without
restarting the app. The catalog SHALL be exposed through an injectable provider so
tests can supply a fake set of pianos.

#### Scenario: Choosing a piano changes the sound
- **WHEN** the user selects a different piano from the catalog
- **THEN** notes played afterward sound with the newly selected piano

#### Scenario: Catalog lists the available pianos
- **WHEN** the picker is shown
- **THEN** it lists every piano in the catalog with the current selection marked

#### Scenario: Selection applies to every input source
- **WHEN** a piano is selected and a note is then triggered from any source
- **THEN** that note sounds with the selected piano's timbre

### Requirement: Persisted Piano Selection

The selected piano SHALL be persisted and restored on the next launch, so the
user's choice survives across sessions. If the persisted selection refers to a
piano that is no longer available, the app SHALL fall back to the default piano
and persist that fallback. Selection state SHALL be held in an injectable provider
backed by persisted storage so tests can drive it with fakes.

#### Scenario: Selection survives a restart
- **WHEN** the user selects a piano and later relaunches the app
- **THEN** the previously selected piano is active

#### Scenario: Unknown persisted selection falls back to default
- **WHEN** the persisted selection refers to a piano absent from the catalog
- **THEN** the app uses the default piano and persists that as the new selection

### Requirement: Piano Picker In Settings Drawer

The app SHALL present the piano selection in the existing settings drawer as a
selectable list (not a flyout dropdown), showing each piano's label and the active
selection. Changing the selection in the drawer SHALL update the persisted choice
and the synthesizer's active SoundFont.

#### Scenario: Picking a piano in the drawer
- **WHEN** the user opens the settings drawer and taps a piano in the list
- **THEN** that piano becomes the active selection and is persisted

#### Scenario: Drawer reflects the active piano
- **WHEN** the settings drawer is opened
- **THEN** the currently active piano is shown as selected in the list

### Requirement: User-Imported SoundFonts

The app SHALL let the user import a SoundFont (`.sf2`) from their device so it
becomes a selectable piano in the catalog. The app SHALL validate that an imported
file is a loadable SoundFont before accepting it and SHALL reject an invalid file
without crashing. An accepted file SHALL be copied into the app's own storage so
the imported piano remains available across launches independent of the original
file, and SHALL be added to a persisted registry of imported pianos. The user
SHALL be able to remove an imported piano; removing the currently selected piano
SHALL fall back to the default. Imported SoundFonts SHALL remain on the device and
SHALL NOT be redistributed by the app.

#### Scenario: Importing a SoundFont adds it to the catalog
- **WHEN** the user imports a valid `.sf2` file
- **THEN** it appears in the catalog as a selectable piano and can be selected to
  play with that sound

#### Scenario: Imported piano survives a restart
- **WHEN** the user imports a SoundFont and later relaunches the app
- **THEN** the imported piano is still listed and selectable

#### Scenario: Invalid file is rejected gracefully
- **WHEN** the user picks a file that is not a loadable SoundFont
- **THEN** the app rejects it with a non-fatal message and the catalog is unchanged

#### Scenario: Removing an imported piano
- **WHEN** the user removes an imported piano that is currently selected
- **THEN** it is deleted from the catalog and the app falls back to the default piano

### Requirement: SoundFont Source And Graceful Fallback

The bytes for a selected piano SHALL be obtained through an injectable source:
bundled pianos load from the app's asset bundle, download-on-first-use pianos are
fetched once, cached, and loaded from cache thereafter, and user-imported pianos
load from their copied file in app storage. If a selected piano's bytes cannot be
obtained (e.g. a download fails or is offline, or an imported file is missing), the
app SHALL fall back to the bundled default piano and SHALL NOT crash or interrupt
playback. The default piano SHALL always be available without a network.

#### Scenario: Bundled piano loads from assets
- **WHEN** a bundled piano is selected
- **THEN** its SoundFont is loaded from the asset bundle and used by the synthesizer

#### Scenario: Failed download falls back to the default
- **WHEN** a download-on-first-use piano is selected but its bytes cannot be fetched
- **THEN** the app falls back to the default piano and continues without crashing

#### Scenario: Selection persists even when audio is unavailable
- **WHEN** audio is unavailable and the user selects a piano
- **THEN** the choice is still persisted and applies once audio becomes available
