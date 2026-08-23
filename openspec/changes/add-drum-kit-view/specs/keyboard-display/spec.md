## MODIFIED Requirements

### Requirement: Keyboard Range Modes

The on-screen keyboard SHALL support a set of range modes: an **auto** mode that
fits the loaded piece, and fixed presets of 25, 37, 49, 61, 76, and 88 keys. The
current mode SHALL be held in immutable state, default to the full **88-key**
piano, and be changeable at runtime through a notifier method.

For a **percussion** score the range apparatus SHALL NOT apply and SHALL NOT be
offered: a drum kit is an unordered set of pieces rather than an interval of
pitches, so there is no window to size and nothing to fit. The controller is the
pad strip instead (see `music-drum-kit-view`), and the range mode retains its value
untouched for the next keyboard score.

#### Scenario: Default mode is the full 88-key piano
- **WHEN** the app starts
- **THEN** the keyboard range mode is the 88-key preset (A0..C8)

#### Scenario: Mode changed at runtime
- **WHEN** the user selects a different range mode
- **THEN** the state updates to that mode and the keyboard re-renders at the new
  range

#### Scenario: Range modes are absent for a percussion score
- **WHEN** a percussion score is loaded
- **THEN** no range mode chooser is offered, and the stored mode is left unchanged
  for the next keyboard score

### Requirement: Shared Range For Keyboard And Waterfall

The keyboard painter and the Synthesia waterfall SHALL render against the same
range so note columns stay aligned to their keys. Changing the range mode SHALL
update both together.

For a **percussion** score the same contract holds over a different quantity: the
pad strip and the cascade SHALL render against the same **lane order**, so a piece
sits in the same horizontal position in both. Any change to that order — including
the inverted-kit setting — SHALL update both together. The requirement is
unchanged in substance: the controller and the falling notes are one mapping, never
two.

#### Scenario: Waterfall stays aligned to keys
- **WHEN** the range mode changes
- **THEN** the falling-note columns and the keyboard keys use the same horizontal
  mapping

#### Scenario: Cascade stays aligned to pads
- **WHEN** the lane order changes for a percussion score
- **THEN** the cascade lanes and the pad strip use the same horizontal mapping

### Requirement: Adaptive Keyboard Height

The on-screen keyboard height SHALL be derived from the available viewport height
rather than a fixed constant, so the keyboard scales down on small phone
viewports and stays proportionate on tablets/desktops. The computed height SHALL
be clamped between a legible minimum and a maximum so the keys remain playable
without the keyboard dominating the render area, and the render area above the
keyboard SHALL always retain a non-zero, usable height. On a viewport too short
for the fixed minimum (e.g. a shrunken desktop window), the minimum SHALL yield
proportionally so the notation above keeps the large majority of the height —
the keyboard never dominates a crushed render area.

The pad strip SHALL follow the same height policy, for the same reasons. Its height
SHALL NOT be derived from the number of pieces: unlike keys, pads keep a usable
touch target however few or many there are, and letting a sparse kit inflate the
strip would take height from the cascade for no gain.

#### Scenario: Keyboard shrinks on small phone viewports
- **WHEN** the player renders on a phone-class landscape viewport shorter than a
  tablet
- **THEN** the keyboard height is smaller than on a tablet/desktop viewport while
  remaining at or above the legible minimum

#### Scenario: Keyboard clamped on tall viewports
- **WHEN** the player renders on a very tall viewport
- **THEN** the keyboard height is clamped to its maximum so it does not dominate
  the render area

#### Scenario: Render area retains usable height
- **WHEN** the keyboard height is computed for any supported viewport
- **THEN** the render area above the keyboard keeps a non-zero, usable height

#### Scenario: The floor yields on very short viewports
- **WHEN** the viewport is too short for the fixed keyboard minimum
- **THEN** the keyboard shrinks below it proportionally and the notation keeps
  the majority of the height

#### Scenario: Pad strip height ignores the piece count
- **WHEN** a percussion score uses three pieces and another uses seven
- **THEN** the pad strip has the same height in both, governed by the viewport
