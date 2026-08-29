## MODIFIED Requirements

### Requirement: Guided in-context coaching for the player's controls

The app SHALL provide a **guided, in-context** coaching sequence in the player that points the
user, **one control at a time** (a directed "here is where to tap" style), to the key play
controls: **selecting the instrument's sound**, **viewing the connected MIDI instrument and
selecting one manually**, **choosing what to practise** (the hand selector on a keyboard, the
per-piece focus control on a kit), and **rewinding by measure
and opening the measure-selection mode** (the transport measure-rewind control — one step
covering both the tap and the long-press gesture). It SHALL run the first time the user reaches
the player, SHALL be **skippable** and MUST NOT permanently block playing, and SHALL be
**replayable** later from the help/tips surface. Each highlighted control SHALL point at the
real control in place (for example within the player settings drawer); when a control is not
currently visible, the step SHALL still deliver its explanation (untargeted fallback).

On a **percussion** score the tour points at the same four positions, but three of them mean
something else, and the copy SHALL say what the control under it actually does: the sound is a
**drum kit**, the connected device is an **electronic kit** rather than a MIDI keyboard, and the
third step is the **per-piece focus** control (`music-kit-piece-focus`) rather than the hand
selector, which a drum score does not offer at all. Teaching the keyboard's vocabulary there is
not a rough edge but a contradiction — the words would say "right hand, left hand" while the
control under the spotlight is a list of kit pieces, and the tour is precisely the moment a
first-time player takes the app at its word. The measure-rewind step is instrument-neutral and
SHALL be shared unchanged.

#### Scenario: First player visit walks the key controls

- **WHEN** the user opens the player for the first time
- **THEN** a guided sequence highlights, one at a time, the sound selection, the
  connected-MIDI/device selection, the practice-scope control for the loaded instrument, and the
  measure-rewind control

#### Scenario: The drum tour names the kit, the e-kit and the pieces

- **WHEN** the first player visit happens on a percussion score
- **THEN** the sound step names the drum kit, the device step names an electronic kit, and the
  third step points at the per-piece focus control — none of them naming a piano, a MIDI
  keyboard, a right and left hand, or hands and feet

#### Scenario: The rewind step teaches both gestures

- **WHEN** the guided sequence reaches the measure-rewind control
- **THEN** one step explains both the tap (rewind one measure) and the long-press (open the
  measure-selection mode)

#### Scenario: The guided sequence is skippable

- **WHEN** the user chooses to skip the guided coaching
- **THEN** it stops and the user can play immediately, without it blocking them

#### Scenario: Replayable from help

- **WHEN** the user wants to see the guided coaching again later
- **THEN** they can replay it from the help/tips surface

#### Scenario: Highlights point at the real controls

- **WHEN** a step highlights a control (e.g. selecting a piano or a MIDI device)
- **THEN** it points at that actual control in place, so the user learns where it is
