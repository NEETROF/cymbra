## MODIFIED Requirements

### Requirement: Keyboard Range Modes

The on-screen keyboard SHALL support a set of range modes: an **auto** mode that
fits the loaded piece, and fixed presets of 25, 37, 49, 61, 76, and 88 keys. The
current mode SHALL be held in immutable state, default to the full **88-key**
piano, and be changeable at runtime through a notifier method.

For a **percussion** score the range apparatus SHALL NOT apply and SHALL NOT be
offered: a drum kit is an unordered set of pieces rather than an interval of
pitches, so there is no window to size and nothing to fit. The controller is the
drawn kit instead (see `music-drum-kit-view`), and the range mode retains its value
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
drawn kit and the falling notes SHALL render against the same **lane order**, so a
piece sits in the same horizontal position in both. Any change to that order — including
the inverted-kit setting — SHALL update both together. The requirement is
unchanged in substance: the controller and the falling notes are one mapping, never
two.

#### Scenario: Waterfall stays aligned to keys
- **WHEN** the range mode changes
- **THEN** the falling-note columns and the keyboard keys use the same horizontal
  mapping

#### Scenario: The falling notes stay aligned to the drawn kit
- **WHEN** the lane order changes for a percussion score
- **THEN** the lanes and the drawn pieces use the same horizontal mapping

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

The band the kit is drawn in SHALL follow the same height policy, for the same
reasons. Its height SHALL NOT be derived from the number of pieces: unlike keys,
drawn pieces keep a usable touch target however few or many there are, and letting
a sparse kit inflate the band would take height from the falling notes for no gain.

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

#### Scenario: The kit's band ignores the piece count
- **WHEN** a percussion score uses three pieces and another uses seven
- **THEN** the kit's band has the same height in both, governed by the viewport

### Requirement: On-Screen Key Input

For a **keyboard** score the on-screen keyboard SHALL be playable by mouse and
touch at all times — **while playback is running and while it is stopped** — in
every render mode. A pointer-down on a key SHALL produce a note-on for that
key's pitch and the matching pointer-up (or pointer-cancel) SHALL produce a
note-off. On-screen play SHALL be handled identically to a physical MIDI key —
internally it reuses the player's note-on/note-off entry points — so it drives
the pressed-key feedback and the Wait Mode gate the same way, including while
Wait Mode is blocking.

For a **percussion** score the drawn kit SHALL be display-only in this change: a
tap on a piece SHALL produce no note, no feedback, and no gate effect. Kit input —
a tap emitting the piece's General MIDI note through the same note-on/off entry
points — arrives with `add-drum-input-mapping`. The interim is stated here
deliberately, so the archived spec never claims an input path the code does not
have; it is also why Wait Mode is not offered for a percussion score in the
meantime (see `music-drum-kit-view`).

#### Scenario: Tap plays a note
- **WHEN** the user presses a displayed key with mouse or finger
- **THEN** a note-on for that key's pitch is emitted and the key shows the pressed
  state

#### Scenario: Release stops the note
- **WHEN** the user lifts the pointer that pressed a key
- **THEN** a note-off for that pitch is emitted and the key leaves the pressed
  state

#### Scenario: On-screen play satisfies the gate
- **WHEN** Wait Mode awaits a note and the user presses that note on the on-screen
  keyboard
- **THEN** the gate is satisfied exactly as if the note arrived from MIDI

#### Scenario: Playable in every mode
- **WHEN** the player is in Synthesia, Staff, or Partition mode
- **THEN** the on-screen keys respond to pointer input

#### Scenario: Playable while stopped
- **WHEN** playback is stopped (the playhead is not advancing)
- **THEN** pressing on-screen keys still emits note-on/note-off and shows the
  pressed state, so the user can play freely

#### Scenario: A tap on a drum does nothing yet
- **WHEN** a percussion score is shown and the user taps a drawn piece
- **THEN** no note is emitted and no visual state changes — the kit is
  display-only until `add-drum-input-mapping`

### Requirement: Three-State Key Feedback

The keyboard SHALL render three visual states per key derived from the notes
required at the current playhead and the notes currently held: a key that is
required but not held SHALL show an "expected / press-this" color; a key that is
required and held SHALL show a distinct "correct" color; a key that is held but
not required SHALL show the "pressed" color. The required set SHALL be the same
gate used by Wait Mode at the current playback position, and SHALL include only
notes belonging to the selected hand(s): when a single hand is selected, notes of
the unselected hand SHALL NOT appear in the required set and SHALL NOT be shown in
the expected or correct state.

For a **percussion** score the drawn pieces SHALL carry none of these states in
this change: the kit is display-only, so there is nothing held to reflect. Piece
feedback — including the "validated piece" the exploration mockups sketch —
arrives with `add-drum-input-mapping`, together with the input it reflects.

#### Scenario: Expected key highlighted
- **WHEN** a note is required at the playhead and not currently held
- **THEN** its key shows the expected/press-this color

#### Scenario: Correct key highlighted distinctly
- **WHEN** a required note is currently held
- **THEN** its key shows the correct color, distinct from the expected color

#### Scenario: Extra pressed key
- **WHEN** a key is held that is not required
- **THEN** it shows the pressed color

#### Scenario: Narrow black key remains visible
- **WHEN** a required or correct key is a black key at a small on-screen width
- **THEN** its highlight remains visible (e.g. via an outline/cap)

#### Scenario: Unselected hand never expected
- **WHEN** a single hand is selected and a note of the other hand falls at the
  playhead
- **THEN** that note's key is not shown in the expected or correct state and is
  absent from the required set

#### Scenario: The drawn pieces carry no feedback states yet
- **WHEN** a percussion score is shown
- **THEN** the pieces render without expected, correct or pressed states — their
  feedback arrives with `add-drum-input-mapping`

### Requirement: Multi-Touch Polyphony

The on-screen keyboard SHALL support multiple simultaneous pointers so that
chords can be played where the platform reports multi-touch. Each pointer SHALL
track its own pressed pitch independently, so releasing one finger SHALL note-off
only that finger's pitch and leave the others sounding.

The requirement binds the **keyboard**; the drawn kit, display-only in this
change, has no pointer behaviour to make polyphonic. Multi-touch kit play — two
hands striking two pieces at once — arrives with `add-drum-input-mapping` and will
be specified there.

#### Scenario: Two keys held at once
- **WHEN** two pointers press two different keys simultaneously
- **THEN** both pitches are note-on and held together

#### Scenario: Independent release
- **WHEN** two keys are held by two pointers and one pointer lifts
- **THEN** only that pointer's pitch is note-off and the other remains held

### Requirement: Assisted Correct-Hand Keys

For a **keyboard** score the desktop keyboard SHALL provide two assist keys that
play the notes expected at the current playhead for one hand: the **left-hand**
key plays all expected staff-2 notes and the **right-hand** key plays all
expected staff-1 notes. The expected set is the same gate used by Wait Mode at
the playhead. Pressing the key SHALL note-on every expected pitch for that hand
and releasing it SHALL note-off those pitches, through the same note-on/off path
as MIDI, so playing the correct hand key satisfies the gate. When no note is
expected for that hand, the key SHALL do nothing.

For a **percussion** score the assist keys SHALL NOT be offered in this change:
their staff-1/staff-2 keying is meaningless for a single-staff drum part, and
with no percussion input path there is no gate for them to satisfy. When
`add-drum-input-mapping` lands, any percussion assist scheme SHALL key on the
hands/feet voice convention stated in `hand-color-coding`, not on staves.

#### Scenario: Right-hand key plays the expected right-hand notes
- **WHEN** staff-1 notes are expected at the playhead and the right-hand assist
  key is pressed
- **THEN** those pitches are note-on (and note-off on release), satisfying the
  Wait Mode gate for the right hand

#### Scenario: Left-hand key plays the expected left-hand notes
- **WHEN** staff-2 notes are expected and the left-hand assist key is pressed
- **THEN** those pitches are note-on and note-off on release

#### Scenario: Chord of expected notes
- **WHEN** a hand has several notes expected at the same playhead
- **THEN** pressing that hand's key plays all of them together

#### Scenario: Nothing expected for the hand
- **WHEN** no note is expected for a hand at the playhead
- **THEN** pressing that hand's assist key produces no note

#### Scenario: No assist keys for a percussion score
- **WHEN** a percussion score is loaded on desktop
- **THEN** the assist keys produce no notes

### Requirement: Hideable Keyboard In Notation Modes

The user SHALL be able to hide the on-screen keyboard while in the scrolling
staff (Portée) render mode, handing the freed height to the score; the setting
SHALL be reachable from the player settings. In Synthesia the keyboard SHALL
always be shown regardless of the setting, because its cascade aligns to the
keys, and the hide option SHALL NOT be offered in Synthesia. In the engraved
Partition mode the keyboard SHALL NOT be shown at all and the hide option SHALL
NOT be offered there: the notation's own expected-note emphasis already tells
the player what to play, and the freed height is what keeps the current and
next staff lines on screen together. The setting SHALL default to visible and
be session-scoped.

The same policy SHALL govern the **drawn kit**: in a percussion play surface —
which, like Synthesia, aligns its falling notes to its controller — the kit
SHALL always be shown and the hide option SHALL NOT be offered. Since the
notation modes are not offered for a percussion score in the interim (see
`music-drum-kit-view`), the hide setting is simply absent for percussion until
`add-drum-notation-render` lands, at which point the kit SHALL follow the
same per-mode rules as the keyboard.

#### Scenario: Hide the keyboard in the scrolling staff
- **WHEN** the user turns the keyboard off while in Staff (Portée) mode
- **THEN** the on-screen keyboard is not rendered and the score takes the freed
  height

#### Scenario: Synthesia always keeps the keyboard
- **WHEN** the mode is Synthesia and the keyboard has been set hidden
- **THEN** the keyboard is still shown, and the hide option is absent from the
  settings

#### Scenario: Partition never shows the keyboard
- **WHEN** the mode is the engraved Partition, whatever the keyboard setting
- **THEN** the on-screen keyboard is not rendered, the hide option is absent
  from the settings, and the engraving takes the full height

#### Scenario: Default is visible
- **WHEN** a session starts
- **THEN** the on-screen keyboard is visible (in the modes that show it)

#### Scenario: A play surface always keeps its drawn kit
- **WHEN** a percussion score is shown in the cascade
- **THEN** the kit is drawn and no hide option is offered
