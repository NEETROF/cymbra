## MODIFIED Requirements

### Requirement: Setup modal contents

The modal SHALL show the score's information — title, composer, difficulty, and
the key signature, time signature and tempo when known (missing values are
omitted, not shown as defaults). It SHALL let the user choose which hands to play
(left / right / both), adjust the playback tempo, toggle the metronome, choose the
note reading aid level (off / note name / note name and rhythm), and see and
select the MIDI input device. The hand choice SHALL be offered only when the piece
has more than one staff; for a single-staff piece it SHALL be omitted and the
selection stays at both.

#### Scenario: Score information is shown

- **WHEN** the modal is shown for a score with a known composer, difficulty, key,
  time signature and tempo
- **THEN** those details are displayed, and any detail the score does not carry is
  omitted rather than shown as a placeholder/default

#### Scenario: Hand choice hidden for a single-staff piece

- **WHEN** the modal is shown for a single-staff piece
- **THEN** the hand chooser is not offered and the selection stays at both

#### Scenario: MIDI device is shown and selectable

- **WHEN** the modal is shown
- **THEN** an auto option and a row per available MIDI input port are shown and
  can be selected; when no device is detected the platform-appropriate guidance
  is shown (the Android USB-OTG / charge-only-cable advice, otherwise a plain
  no-device state)

#### Scenario: Reading aid level is shown and selectable

- **WHEN** the modal is shown
- **THEN** the note reading aid offers the three levels (off / note name / note
  name and rhythm), with the currently persisted level preselected

### Requirement: Setup choices persist across scores and restarts

The system SHALL remember the play settings (hands, tempo/playback speed,
metronome, the note reading aid level, and the chosen MIDI device) on the device.
Applied choices SHALL persist across scores and SHALL survive an app restart, and
each newly opened score SHALL be seeded from them. The same settings SHALL be
editable both in the modal and in the in-game settings, and a change made in
either place SHALL update the shared, persisted value. A stored value that is
missing or unrecognized SHALL fall back to that setting's default rather than
discarding the whole stored record.

#### Scenario: Choices are remembered on the next score

- **WHEN** the user sets the hands/tempo/metronome/reading aid/device (in the
  modal or the in-game settings) and later opens another score
- **THEN** that score is seeded with the same settings

#### Scenario: Choices survive an app restart

- **WHEN** the user has set play settings and restarts the app
- **THEN** the restored settings are in effect when a score is opened

#### Scenario: In-game changes update the same settings

- **WHEN** the user changes a setting from the in-game settings while playing
- **THEN** the shared persisted value is updated (the next score and the next
  launch reflect it)

#### Scenario: Unknown stored setting falls back to its default

- **WHEN** the persisted settings were written by an earlier version and carry no
  reading aid level, or carry a value the app does not recognize
- **THEN** the reading aid falls back to its default (off) while the other stored
  settings are still restored
