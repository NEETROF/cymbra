## ADDED Requirements

### Requirement: Hand Colour Convention

The player SHALL use a fixed two-colour convention to tell the hands apart: the
right hand (staff 1) SHALL be drawn in a cool **blue** and the left hand (staff
2 and above) in a warm **amber**. These hand colours SHALL be visually distinct
from the "correct" state (green) and the "extra key pressed" state (purple) used
by the feedback.

#### Scenario: Right and left hands use distinct colours
- **WHEN** a right-hand note and a left-hand note are shown together
- **THEN** the right-hand note is blue and the left-hand note is amber

#### Scenario: Hand colours differ from the feedback states
- **WHEN** the hand colours are shown alongside the correct/pressed feedback
- **THEN** blue and amber are distinct from the green "correct" and purple
  "pressed" colours

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
