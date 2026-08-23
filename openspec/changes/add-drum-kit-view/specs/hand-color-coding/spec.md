## MODIFIED Requirements

### Requirement: Hand Colour Convention

The player SHALL use a fixed two-colour convention to tell the hands apart: the
right hand (staff 1) SHALL be drawn in a cool **blue** and the left hand (staff
2 and above) in a warm **amber**. These hand colours SHALL be visually distinct
from the "correct" state (green) and the "extra key pressed" state (purple) used
by the feedback.

For a **percussion** score the same two colours SHALL distinguish **hands from
feet**, keyed to the note's **voice** rather than to its staff: a drum part is
written on a single staff in two voices — hands with stems up, feet with stems
down — so a staff-based split would put everything in one colour. Hand-struck
pieces take the blue, foot-struck ones the amber. The distinction from the
correct/pressed feedback colours is unchanged.

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

### Requirement: Hand Colours In The Render Modes

Every mode that draws notes SHALL colour each note by its hand (right = blue,
left = amber) — the Synthesia waterfall, the scrolling Staff, and the Partition
engraving. A note at the playhead SHALL be emphasised: green once its key is held,
otherwise a brighter tint of its hand colour.

For a percussion score the same rule applies with hands/feet in place of
right/left, and it extends to the cascade's **foot bars**, which take the amber of
the foot voice — so the colour carries the same meaning whether an event is drawn
as a note in a lane or as a bar across the width.

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
