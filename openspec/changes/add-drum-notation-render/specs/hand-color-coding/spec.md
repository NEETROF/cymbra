## MODIFIED Requirements

### Requirement: Hand Colours In The Render Modes

Every mode that draws notes SHALL colour each note by its hand (right = blue,
left = amber) — the Synthesia waterfall, the scrolling Staff, and the Partition
engraving. A note at the playhead SHALL be emphasised: green once its key is held,
otherwise a brighter tint of its hand colour.

For a percussion score the rule SHALL apply with hands/feet in place of
right/left — classified by the voice convention stated normatively in the
`Hand Colour Convention` requirement — in **every** mode that draws the score:
the cascade (including its foot bars, which take the amber of the feet), and
now the scrolling Staff and the engraved Partition, whose percussion drawing
this change (`add-drum-notation-render`) delivers. This resolves the deferral
`add-drum-kit-view` recorded here: an engraved hand-struck note takes the
blue and a foot-struck one the amber, so a kick at F4 and a hi-hat at G5 carry
the same colours on the staff as their bar and lane carry in the cascade —
one meaning per colour, whatever the surface. The playhead emphasis rule
applies unchanged. The convention colours the **app's** notation surfaces; the
console's renderer stays monochrome ink, as it is for keyboard scores — the
colour coding is a player aid, not part of the browser preview's fidelity
contract.

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

#### Scenario: Engraved percussion notes coloured hands/feet
- **WHEN** a percussion score is shown in Staff or Partition mode
- **THEN** hand-struck note heads are blue and foot-struck ones amber, split by
  the voice convention (single-voice fallback included), matching the cascade's
  colours for the same events
