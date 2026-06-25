# keyboard-display Specification

## Purpose

The keyboard-display capability governs the on-screen piano keyboard and its
aligned Synthesia-style waterfall in Cymbra: locking the app to landscape,
choosing the visible key range (auto-fit or fixed presets), keeping the keyboard
and waterfall rendered against a shared range, and showing three-state per-key
feedback (expected / correct / pressed) driven by the same gate used by Wait
Mode.
## Requirements
### Requirement: Landscape-Locked Orientation

The app SHALL run in landscape orientation only; portrait SHALL be unavailable on
mobile devices. The lock SHALL be enforced both by the Flutter runtime and by the
native iOS and Android configuration. On platforms without device orientation
(desktop/web) the lock SHALL be a no-op with no regression.

#### Scenario: Portrait is disabled on mobile
- **WHEN** the app runs on a phone or tablet and the device is rotated to portrait
- **THEN** the UI remains in landscape

#### Scenario: Desktop unaffected
- **WHEN** the app runs on macOS/Linux/Windows/web
- **THEN** orientation locking is a no-op and the app renders normally

### Requirement: Keyboard Range Modes

The on-screen keyboard SHALL support a set of range modes: an **auto** mode that
fits the loaded piece, and fixed presets of 25, 37, 49, 61, 76, and 88 keys. The
current mode SHALL be held in immutable state, default to the full **88-key**
piano, and be changeable at runtime through a notifier method.

#### Scenario: Default mode is the full 88-key piano
- **WHEN** the app starts
- **THEN** the keyboard range mode is the 88-key preset (A0..C8)

#### Scenario: Mode changed at runtime
- **WHEN** the user selects a different range mode
- **THEN** the state updates to that mode and the keyboard re-renders at the new
  range

### Requirement: Auto-Fit To Piece Range

In auto mode the keyboard range SHALL be computed from the loaded score's notes:
the low bound snaps down to a C and the high bound up to an octave top, the span
is at least two octaves, and the range is clamped within the 88-key bounds
(A0=21 .. C8=108). The computed range SHALL always include every note in the
score so no note is dropped from the keyboard or the aligned waterfall. With no
notes loaded the range SHALL fall back to a sensible default (C4..C6).

#### Scenario: Range covers the piece
- **WHEN** a score's lowest note is D3 and highest is F5
- **THEN** the auto range's low is ≤ D3 and high is ≥ F5

#### Scenario: Minimum span enforced
- **WHEN** a score uses only two adjacent pitches
- **THEN** the auto range still spans at least two octaves

#### Scenario: Clamped to piano bounds
- **WHEN** a score includes the lowest/highest piano keys
- **THEN** the auto range never extends below A0 or above C8

#### Scenario: Empty score fallback
- **WHEN** no notes are loaded
- **THEN** the range falls back to C4..C6

### Requirement: Preset Covers The Piece

When a fixed preset is selected, the displayed range SHALL be positioned to cover
the score's notes; if the piece is wider than the preset, the range SHALL widen
to include all notes rather than clip them. The range SHALL remain clamped within
the 88-key bounds.

#### Scenario: Preset window shifts to the music
- **WHEN** a 49-key preset is selected and the piece sits in a higher register
- **THEN** the displayed window shifts to include the piece's notes

#### Scenario: Piece wider than preset
- **WHEN** the piece spans more keys than the selected preset
- **THEN** the displayed range widens to include every note

### Requirement: Shared Range For Keyboard And Waterfall

The keyboard painter and the Synthesia waterfall SHALL render against the same
range so note columns stay aligned to their keys. Changing the range mode SHALL
update both together.

#### Scenario: Waterfall stays aligned to keys
- **WHEN** the range mode changes
- **THEN** the falling-note columns and the keyboard keys use the same horizontal
  mapping

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

### Requirement: On-Screen Key Input

The on-screen keyboard SHALL be playable by mouse and touch at all times —
**while playback is running and while it is stopped** — in every render mode. A
pointer-down on a key SHALL produce a note-on for that key's pitch and the
matching pointer-up (or pointer-cancel) SHALL produce a note-off. On-screen play
SHALL be handled identically to a physical MIDI key — internally it reuses the
player's note-on/note-off entry points — so it drives the pressed-key feedback and
the Wait Mode gate the same way, including while Wait Mode is blocking.

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

### Requirement: Pointer Pitch Hit-Testing

Mapping a pointer position to a pitch SHALL use the shared keyboard layout so a
tap selects the key actually drawn under the pointer. Black keys SHALL take
priority over white keys within their overlapping region (the upper part of the
keyboard where a black key is painted over the gap between whites); a pointer
outside the displayed key range SHALL map to no pitch and produce no note.

#### Scenario: White key hit
- **WHEN** the pointer is over the lower part of a white key, away from any black
  key
- **THEN** the hit-test returns that white key's pitch

#### Scenario: Black key priority in overlap
- **WHEN** the pointer is in the upper region where a black key overlaps the
  boundary between two white keys
- **THEN** the hit-test returns the black key's pitch, not the white key's

#### Scenario: Out of range is ignored
- **WHEN** the pointer is beyond the lowest or highest displayed key
- **THEN** the hit-test returns no pitch and no note-on is emitted

### Requirement: Multi-Touch Polyphony

The on-screen keyboard SHALL support multiple simultaneous pointers so that
chords can be played where the platform reports multi-touch. Each pointer SHALL
track its own pressed pitch independently, so releasing one finger SHALL note-off
only that finger's pitch and leave the others sounding.

#### Scenario: Two keys held at once
- **WHEN** two pointers press two different keys simultaneously
- **THEN** both pitches are note-on and held together

#### Scenario: Independent release
- **WHEN** two keys are held by two pointers and one pointer lifts
- **THEN** only that pointer's pitch is note-off and the other remains held

### Requirement: Assisted Correct-Hand Keys

The desktop keyboard SHALL provide two assist keys that play the notes expected
at the current playhead for one hand: the **left-hand** key plays all expected
staff-2 notes and the **right-hand** key plays all expected staff-1 notes. The
expected set is the same gate used by Wait Mode at the playhead. Pressing the key
SHALL note-on every expected pitch for that hand and releasing it SHALL note-off
those pitches, through the same note-on/off path as MIDI, so playing the correct
hand key satisfies the gate. When no note is expected for that hand, the key
SHALL do nothing.

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

### Requirement: Assisted Near-Miss Keys

The desktop keyboard SHALL provide two near-miss keys that play a deliberately
wrong note close to the expected one, one per hand. Pressing a near-miss key
SHALL pick a pitch near an expected note for that hand — within a few semitones,
never equal to an expected pitch and kept within the displayed keyboard range —
note-on that pitch on key-down, and note-off the same pitch on key-up. Because the
pitch differs from the expected note, a near-miss SHALL NOT satisfy the Wait Mode
gate. When no note is expected for that hand, the near-miss key SHALL do nothing.

#### Scenario: Near-miss plays a nearby wrong note
- **WHEN** a note is expected for a hand and that hand's near-miss key is pressed
- **THEN** a pitch within a few semitones of the expected note — and not equal to
  any expected pitch — is note-on, and note-off on release

#### Scenario: Near-miss does not satisfy the gate
- **WHEN** Wait Mode is waiting and a near-miss key is pressed
- **THEN** the gate stays blocked (the wrong pitch does not match the expected
  note)

#### Scenario: Near-miss stays in range
- **WHEN** the expected note is at the edge of the displayed keyboard range
- **THEN** the chosen near-miss pitch is still within the displayed range

### Requirement: Assist Keys Replace The Pitch Fallback

The desktop keyboard SHALL use the four assist keys (left-correct, right-correct,
left-near-miss, right-near-miss) as its input scheme, replacing the former fixed
per-pitch key-to-note mapping. The on-screen keyboard (mouse/touch) SHALL remain
the way to play exact, arbitrary notes and SHALL be unaffected.

#### Scenario: Old per-pitch keys no longer fire notes
- **WHEN** a key from the former pitch mapping (that is not an assist key) is
  pressed
- **THEN** no note is produced by the desktop keyboard

#### Scenario: On-screen keyboard still plays exact notes
- **WHEN** the user presses a key on the on-screen keyboard
- **THEN** that exact pitch plays as before

