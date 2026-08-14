## MODIFIED Requirements

### Requirement: Guided in-context coaching for the player's controls

The app SHALL provide a **guided, in-context** coaching sequence in the player that points the
user, **one control at a time** (a directed "here is where to tap" style), to the key play
controls: **selecting a piano sound**, **viewing the connected MIDI instrument and selecting one
manually**, **choosing which hand(s) to play** (right / left / both), and **rewinding by measure
and opening the measure-selection mode** (the transport measure-rewind control — one step
covering both the tap and the long-press gesture). It SHALL run the first time the user reaches
the player, SHALL be **skippable** and MUST NOT permanently block playing, and SHALL be
**replayable** later from the help/tips surface. Each highlighted control SHALL point at the
real control in place (for example within the player settings drawer); when a control is not
currently visible, the step SHALL still deliver its explanation (untargeted fallback).

#### Scenario: First player visit walks the key controls

- **WHEN** the user opens the player for the first time
- **THEN** a guided sequence highlights, one at a time, the piano-sound selection, the connected-MIDI/device selection, the hand selection, and the measure-rewind control

#### Scenario: The rewind step teaches both gestures

- **WHEN** the guided sequence reaches the measure-rewind step
- **THEN** its copy explains both that a tap rewinds one measure and that a long-press opens the measure-selection mode

#### Scenario: The guided sequence is skippable

- **WHEN** the user chooses to skip the guided coaching
- **THEN** it stops and the user can play immediately, without it blocking them

#### Scenario: Replayable from help

- **WHEN** the user wants to see the guided coaching again later
- **THEN** they can replay it from the help/tips surface

#### Scenario: Highlights point at the real controls

- **WHEN** a step highlights a control (e.g. selecting a piano or a MIDI device)
- **THEN** it points at that actual control in place, so the user learns where it is
