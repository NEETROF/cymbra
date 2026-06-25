## ADDED Requirements

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
