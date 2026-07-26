## ADDED Requirements

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
(left / right / both), adjust the playback tempo and toggle the metronome, and
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

#### Scenario: MIDI device is shown and selectable

- **WHEN** the modal is shown
- **THEN** an auto option and a row per available MIDI input port are shown and
  can be selected; when no device is detected a no-device state is shown

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
