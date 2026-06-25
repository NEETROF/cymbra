## Why

The desktop computer-keyboard input maps 15 QWERTY keys to fixed pitches
(C4..D5) — cramped, only one octave, and it forces the user to know the exact
notes. For practising along with a piece, what's actually wanted is to advance by
playing "the right hand" or "the left hand" without hunting pitches, and to be
able to deliberately fire a near-miss to exercise the wrong-note path. A handful
of assist keys is far easier than a tiny piano on the keyboard.

## What Changes

- **Replace** the 15-key pitch mapping with **four assist keys** that act on the
  notes expected at the current playhead, split by hand (staff 1 = right, staff
  2 = left):
  - `a` → play **all expected left-hand notes** (correct) — satisfies the gate.
  - `z` → play **all expected right-hand notes** (correct) — satisfies the gate.
  - `q` → play a **random pitch near** the expected left-hand note (a near-miss
    that does **not** satisfy the gate).
  - `s` → play a **random pitch near** the expected right-hand note (near-miss).
- Each assist key emits note-on on key-down and note-off on key-up, through the
  same player note-on/off path as MIDI and the on-screen keyboard.
- When no note is expected for that hand, the key does nothing.

## Capabilities

### New Capabilities
<!-- ADDS requirements to the existing keyboard-display capability spec. -->
- `keyboard-display`: assisted practice keys on the desktop keyboard — correct
  per-hand keys that play the expected notes, and near-miss keys that play a
  random nearby wrong note, replacing the per-pitch fallback mapping.

## Impact

- **Input**: `screens/player_screen.dart` — remove `_keyToPitch`; map
  `LogicalKeyboardKey.keyA/keyZ/keyQ/keyS` to the four assist actions; track the
  pitches each key fired so key-up note-offs exactly those (random varies per
  press).
- **State**: `state/player_data.dart` — add a helper returning the expected notes
  at a time **filtered by hand** (using `TimedNote.staff`); a pure near-miss
  picker (expected pitch → a nearby different pitch, kept within the keyboard
  range) with injectable randomness for deterministic tests.
- **Tests**: unit tests for the per-hand expected lookup and the near-miss picker
  (near but never equal, in range); update the existing computer-keyboard widget
  test to the new assist keys (correct key satisfies Wait Mode; near-miss does
  not).
- No Rust/engine or public-API change. Additive to `keyboard-display` (no
  requirement modified). The on-screen keyboard (mouse/touch) is unaffected and
  still plays exact notes.
