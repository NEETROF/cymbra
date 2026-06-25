## MODIFIED Requirements

### Requirement: Partition Rendering State

The Flutter layer SHALL store the parsed-and-laid-out notation in immutable
state and expose it to a Partition-mode `CustomPainter`, which SHALL draw the
computed systems and, for a piano part, the staves of each system with their
measures and notes. When the selected hand is **both** the painter SHALL draw
both staves (treble + bass) of each system; when a single hand is selected the
painter SHALL **collapse the unselected hand's staff entirely** — its staff
lines, clef, key/time signature and notes SHALL NOT be drawn — so only the
selected hand's staff is laid out and rendered. The Partition view SHALL be
presented as a render mode of the existing player screen, alongside the
time-based modes. Updating the loaded document SHALL update the state so the
painter re-renders the new measures.

The Partition painter SHALL engrave with a SMuFL music font (Bravura): note
heads, clefs, flags, accidentals, rests and dynamics SHALL be font glyphs, while
stems, beams, staff and ledger lines are stroked. It SHALL draw, per system, the
clef in effect (including mid-piece clef changes), the key signature (armature)
and the time signature, and per note its beams/flags, dots and accidental. It
SHALL draw tuplet numbers over their groups, ties between tied same-pitch notes,
and phrasing slurs arcing over their phrase. Glyphs belonging to a collapsed
(unselected) staff SHALL NOT be drawn.

#### Scenario: Painter renders both staves
- **WHEN** notation state holds a laid-out two-staff score document and the
  selection is **both**
- **THEN** the Partition painter reads its systems and renders treble and bass
  staves together

#### Scenario: Unselected staff collapsed
- **WHEN** the selection is **right** on a two-staff score
- **THEN** the painter draws only the treble (staff 1) staff — lines, clef,
  signatures and notes — and the bass (staff 2) staff is not drawn

#### Scenario: Partition is a mode of the player screen
- **WHEN** the player screen displays a selected score and the user chooses the
  Partition render mode
- **THEN** the engraved notation is shown within the player, while the on-screen
  keyboard and transport remain present

#### Scenario: Engraved with SMuFL glyphs and signatures
- **WHEN** a score with a key signature and time signature is rendered
- **THEN** each system shows the clef, key signature and time signature, and
  notes are drawn as SMuFL note-head/flag/accidental/rest glyphs

#### Scenario: Tuplets, ties and slurs are drawn
- **WHEN** the score contains a triplet, a tie and a phrasing slur
- **THEN** the painter draws the tuplet number over its group, a tie arc between
  the tied notes, and a slur arc over the phrase

#### Scenario: Clef change is shown mid-system
- **WHEN** a staff changes clef partway through a system
- **THEN** the painter draws the new clef at that measure and positions the
  following notes from the changed clef

#### Scenario: New document re-renders
- **WHEN** a new MusicXML document is loaded into the notation state
- **THEN** the state updates and the painter renders the new measures

### Requirement: Derived Playback Timing

The system SHALL derive a visual playback timing from a parsed score so the
existing time-based render modes (waterfall and scrolling staff) can present the
selected piece. For each non-rest note it SHALL compute a MIDI pitch from the
note's step, octave and alteration, and a start time and duration in
milliseconds from the note's running division position and a tempo (taken from a
`metronome` direction when present, otherwise a default). Chord members SHALL
share the onset of the note they attach to; rests SHALL NOT produce a played
note. Each derived note SHALL also carry its staff, beam states and the clef in
effect, so the scrolling Staff mode can lay out a grand staff with beamed groups
and position notes by the clef in force (honouring mid-piece clef changes), and
so consumers can filter notes by hand using the staff. When a single hand is
selected the scrolling Staff mode SHALL draw only the selected hand's staff and
notes, collapsing the unselected staff. This derivation is visual only — no audio
synthesis or MIDI output is produced.

#### Scenario: Pitch derived from step, octave and alteration
- **WHEN** a note declares step C, octave 4, alteration 0
- **THEN** the derived note has MIDI pitch 60 (middle C); an alteration of +1
  yields 61

#### Scenario: Staff mode positions notes by the clef in effect
- **WHEN** a staff's clef changes from treble to bass mid-piece
- **THEN** the scrolling Staff mode positions that staff's notes from the clef in
  force at each note and shows the clef in effect at the playhead

#### Scenario: Staff mode collapses an unselected hand
- **WHEN** the selection is **left** on a two-staff piece in Staff mode
- **THEN** only the staff-2 (bass) staff and its notes are drawn and the staff-1
  staff is collapsed

#### Scenario: Timing scales with divisions and tempo
- **WHEN** two quarter notes follow one another at a known divisions value and
  tempo
- **THEN** the second note's start time equals the first note's start plus one
  quarter-note duration in milliseconds

#### Scenario: Rests are not played
- **WHEN** the measure contains a rest between two notes
- **THEN** the derived timeline contains the two notes and no entry for the rest
