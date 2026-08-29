# hand-color-coding Specification

## Purpose
TBD - created by archiving change hand-color-coding. Update Purpose after archive.
## Requirements
### Requirement: Hand Colour Convention

The player SHALL use a fixed two-colour convention to tell the hands apart: the
right hand (staff 1) SHALL be drawn in a cool **blue** and the left hand (staff
2 and above) in a warm **amber**. These hand colours SHALL be visually distinct
from the "correct" state (green) and the "extra key pressed" state (purple) used
by the feedback.

For a **percussion** score the same two colours SHALL distinguish **hands from
feet**, keyed to the note's **voice** rather than to its staff — a drum part is
written on a single staff, so a staff-based split would put everything in one
colour. The convention is stated here once, normatively, and every capability
that needs the hands/feet split (the cascade's foot bars, the hand filter)
references it rather than restating it: **in a drum part written in two voices,
voice 1 — the stems-up voice — is the hands, and voice 2 — the stems-down
voice — is the feet.** For a **single-voice** file — common in real exports,
where everything including the kick sits in voice 1 — the voice cannot
discriminate, and the split SHALL fall back to the note's General MIDI number:
the kick (35 and 36) and the pedal hi-hat (44) are feet, everything else is
hands. Hand-struck events take the blue, foot-struck ones the amber; the
distinction from the correct/pressed feedback colours is unchanged.

#### Scenario: Right and left hands use distinct colours
- **WHEN** a right-hand note and a left-hand note are shown together
- **THEN** the right-hand note is blue and the left-hand note is amber

#### Scenario: Hand colours differ from the feedback states
- **WHEN** the hand colours are shown alongside the correct/pressed feedback
- **THEN** blue and amber are distinct from the green "correct" and purple
  "pressed" colours

#### Scenario: Hands and feet are distinguished on a single-staff drum part
- **WHEN** a percussion score shows a hand-struck and a foot-struck note together
- **THEN** the hand-struck note is blue and the foot-struck one amber, although
  both are on the same staff

#### Scenario: A single-voice drum file still splits hands from feet
- **WHEN** a percussion score is written in a single voice, kick included
- **THEN** the kick classifies as feet by its General MIDI number and its bar is
  amber, while the hand-struck notes stay blue

### Requirement: Hand Colours On The Keyboard

The on-screen keyboard SHALL tint a key that is expected at the playhead but not
yet held by its hand — blue for a right-hand note, amber for a left-hand note —
rather than a single "expected" colour. A key that is required and held SHALL
still read as "correct" (green), and a held key that is not required SHALL still
read as "pressed" (purple).

#### Scenario: Expected key tinted by hand
- **WHEN** a right-hand note is expected and its key is not held
- **THEN** that key is shown in the right-hand (blue) colour; a left-hand expected
  key is shown amber

#### Scenario: Correct and pressed unchanged
- **WHEN** an expected key is held, or a non-expected key is held
- **THEN** it shows the correct (green) or pressed (purple) colour respectively

### Requirement: Hand Colours In The Render Modes

Every mode that draws notes SHALL colour each note by its hand (right = blue,
left = amber) — the Synthesia waterfall, the scrolling Staff, and the Partition
engraving. A note at the playhead SHALL be emphasised: green once its key is held,
otherwise a brighter tint of its hand colour.

For a percussion score the rule SHALL apply with hands/feet in place of
right/left — classified by the voice convention above — **in the cascade only**,
extending to its **foot bars**, which take the amber of the feet, so the colour
carries the same meaning whether an event is drawn as a note in a lane or as a
bar across the width. What the Staff and Partition modes do for a percussion
score is deferred, with the rest of percussion notation, to
`add-drum-notation-render`; in the interim those modes are not offered for a
percussion score at all (see `music-drum-kit-view`).

#### Scenario: Falling notes coloured by hand (Synthesia)
- **WHEN** the Synthesia waterfall shows a right-hand and a left-hand note
- **THEN** the right-hand note column is blue and the left-hand one amber

#### Scenario: Staff and Partition note heads coloured by hand
- **WHEN** a grand-staff piece is shown in Staff or Partition mode
- **THEN** treble (right) note heads are blue and bass (left) note heads are amber

#### Scenario: Playhead note emphasised
- **WHEN** a note is at the playhead
- **THEN** it is green if its key is held, otherwise a brighter tint of its hand
  colour

#### Scenario: Foot bars carry the foot colour
- **WHEN** the cascade draws a kick bar alongside hand notes
- **THEN** the bar is amber and the hand notes are blue, matching the convention
  used everywhere else

