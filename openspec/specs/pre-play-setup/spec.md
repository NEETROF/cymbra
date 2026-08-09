# pre-play-setup Specification

## Purpose
TBD - created by archiving change add-pre-play-setup-modal. Update Purpose after archive.
## Requirements
### Requirement: Pre-play setup modal on player entry

The player SHALL present a centered setup modal each time it opens a score, once
the score's notation has loaded. The modal SHALL be shown over the play surface
(the score does not begin until the modal is dismissed) and SHALL be presented
once per opened score.

#### Scenario: Modal shown when a score opens

- **WHEN** the player opens a score and its notation has loaded
- **THEN** the pre-play setup modal is shown centered over the player before play
  begins

#### Scenario: Shown again on each open

- **WHEN** the user opens a score, dismisses the modal, leaves the player, and
  opens a score again
- **THEN** the modal is shown again for the new opening

### Requirement: Setup modal contents

The modal SHALL show the score's information — title, composer, difficulty, and
the key signature, time signature and tempo when known (missing values are
omitted, not shown as defaults). It SHALL let the user choose which hands to play
(left / right / both), adjust the playback tempo and toggle the metronome,
choose the score size (small / medium / large), and
see and select the MIDI input device. The hand choice SHALL be offered only when
the piece has more than one staff; for a single-staff piece it SHALL be omitted
and the selection stays at both.

#### Scenario: Score information is shown

- **WHEN** the modal is shown for a score with a known composer, difficulty, key,
  time signature and tempo
- **THEN** those details are displayed, and any detail the score does not carry is
  omitted rather than shown as a placeholder/default

#### Scenario: Hand choice hidden for a single-staff piece

- **WHEN** the modal is shown for a single-staff piece
- **THEN** the hand chooser is not offered and the selection stays at both

#### Scenario: Score size is offered

- **WHEN** the modal is shown
- **THEN** a score size chooser with small / medium / large is offered,
  reflecting the current persisted value

#### Scenario: MIDI device is shown and selectable

- **WHEN** the modal is shown
- **THEN** an auto option and a row per available MIDI input port are shown and
  can be selected; when no device is detected the platform-appropriate guidance
  is shown (the Android USB-OTG / charge-only-cable advice, otherwise a plain
  no-device state)

### Requirement: Validate applies, close keeps current settings

The modal SHALL have a Validate action that applies the chosen settings and
closes the modal, and a close (X) action that dismisses the modal keeping the
current settings unchanged. In both cases the user remains on the player.

#### Scenario: Validate applies the chosen settings

- **WHEN** the user changes the hands, tempo, metronome or device and taps
  Validate
- **THEN** the changes are applied to the player and the modal closes, leaving the
  user on the player

#### Scenario: Close keeps current settings

- **WHEN** the user makes changes in the modal and taps the close (X) action
- **THEN** the modal closes without applying those changes, the previous settings
  remain in effect, and the user stays on the player

### Requirement: Setup choices persist across scores and restarts

The system SHALL remember the play settings (hands, tempo/playback speed,
metronome, score size, and the chosen MIDI device) on the device. Applied
choices SHALL
persist across scores and SHALL survive an app restart, and each newly opened
score SHALL be seeded from them. The same settings SHALL be editable both in the
modal and in the in-game settings, and a change made in either place SHALL update
the shared, persisted value.

#### Scenario: Choices are remembered on the next score

- **WHEN** the user sets the hands/tempo/metronome/device/score size (in the
  modal or the in-game settings) and later opens another score
- **THEN** that score is seeded with the same settings

#### Scenario: Choices survive an app restart

- **WHEN** the user has set play settings and restarts the app
- **THEN** the restored settings are in effect when a score is opened

#### Scenario: In-game changes update the same settings

- **WHEN** the user changes a setting from the in-game settings while playing
- **THEN** the shared persisted value is updated (the next score and the next
  launch reflect it)

