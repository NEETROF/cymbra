## ADDED Requirements

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
gate used by Wait Mode at the current playback position.

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
