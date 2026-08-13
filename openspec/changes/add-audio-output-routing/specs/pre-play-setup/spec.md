## MODIFIED Requirements

### Requirement: Setup modal contents

The modal SHALL show the score's information — title, composer, difficulty, and
the key signature, time signature and tempo when known (missing values are
omitted, not shown as defaults). It SHALL let the user choose which hands to play
(left / right / both), adjust the playback tempo and toggle the metronome, and
see and select the MIDI input device. The hand choice SHALL be offered only when
the piece has more than one staff; for a single-staff piece it SHALL be omitted
and the selection stays at both. The modal SHALL additionally offer a sound
output section showing where the app's audio goes — the selectable output device
on desktop and Android (USB outputs labelled experimental on Android), or the
active route with access to the system route picker on iOS — together with the
instrument-sounds-itself setting. The output list SHALL be re-read each time the
modal opens, so a device plugged in since the app started is offered.

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

#### Scenario: Sound output is shown

- **WHEN** the modal is shown
- **THEN** a sound output section shows where the app's audio goes — the output
  device list on desktop and Android, or the active route plus access to the
  system route picker on iOS

#### Scenario: Output list is current

- **WHEN** the modal is opened after an output device was plugged or unplugged
- **THEN** the sound output section lists the outputs as they are now, not as
  they were when the app started

#### Scenario: Instrument-sounds-itself is offered with the sound output

- **WHEN** the modal is shown and a MIDI instrument is connected
- **THEN** the sound output section offers the instrument-sounds-itself setting,
  which is shown disabled with its reason when no instrument is connected
