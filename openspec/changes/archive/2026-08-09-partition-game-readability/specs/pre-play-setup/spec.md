# pre-play-setup delta — partition-game-readability

## MODIFIED Requirements

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
