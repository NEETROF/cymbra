## ADDED Requirements

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
