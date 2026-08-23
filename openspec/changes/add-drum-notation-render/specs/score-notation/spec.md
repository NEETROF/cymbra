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

For a **percussion** score (change: `add-drum-notation-render`) the painter
SHALL engrave the single percussion staff per `music-percussion-engraving` —
percussion clef, no armature, written-position placement, the shared head
classification, two-voice layout. The staff-collapse rule above is **scoped to
keyboard scores**: a drum part has exactly one staff, so when a single class is
selected (hands or feet, per `hand-selection`) the painter SHALL keep drawing
the staff — lines, clef, time signature — and SHALL filter the **events** by
the hands/feet voice convention instead; collapsing "the unselected hand's
staff" would erase the only staff there is. This scoping resolves what would
otherwise be a contradiction between the keyboard collapse rule and the
percussion filter.

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

#### Scenario: A percussion score engraves on its single staff
- **WHEN** a percussion score is shown in Partition mode
- **THEN** one percussion-clef staff is engraved per
  `music-percussion-engraving`, with no armature and with the percussion head
  classes

#### Scenario: Filtering the feet never collapses the percussion staff
- **WHEN** a percussion score is shown in Partition mode and **hands** is
  selected
- **THEN** the staff, its clef and its time signature stay drawn, the
  foot-struck events are hidden by the voice convention, and no staff is
  collapsed

### Requirement: Derived Playback Timing

The system SHALL derive a playback timing from a parsed score so the time-based
render modes (waterfall and scrolling staff) can present the selected piece and so
the audio synthesizer can play it. For each non-rest note it SHALL compute a start
time and duration in milliseconds from the note's running division position and a
tempo (taken from a `metronome` direction when present, otherwise a default), and a
MIDI number obtained as follows: for a **pitched** note, computed from the note's
step, octave and alteration; for an **unpitched** note — emitted only when the
score classifies as percussion (see the `music-percussion-notation` capability),
so a mixed score admissible through today's validation gate keeps its exact
current playback — the General MIDI percussion number resolved from the part
list's instrument declarations. An unpitched note whose number cannot be resolved
SHALL be omitted rather than emitted with a fabricated number; tied unpitched
notes merge into one prolonged note keyed by voice and resolved General MIDI
number, and a chain whose number is unresolved is omitted entirely. Chord members
SHALL share the onset of the note they attach to;
rests SHALL NOT produce a played note. Each derived note SHALL also carry its
staff, beam states and the clef in effect, so the scrolling Staff mode can lay out
a grand staff with beamed groups and position notes by the clef in force (honouring
mid-piece clef changes). A derived **unpitched** note SHALL additionally carry its
**written staff placement** (the diatonic index of its display step and octave)
and its voice, and the scrolling Staff mode SHALL position it from that placement
on the percussion staff per `music-percussion-engraving` — never from the General
MIDI number it carries in the MIDI slot, which is a sound identity, not a
position (change: `add-drum-notation-render`; the derived rests likewise carry
their voice, so the two-voice rest placement can apply). The derivation itself
produces no sound — it is a timing/pitch model; audible playback is rendered by
the audio-output synthesizer that consumes this timing (see the `audio-output`
capability).

#### Scenario: Pitch derived from step, octave and alteration
- **WHEN** a note declares step C, octave 4, alteration 0
- **THEN** the derived note has MIDI pitch 60 (middle C); an alteration of +1
  yields 61

#### Scenario: Staff mode positions notes by the clef in effect
- **WHEN** a staff's clef changes from treble to bass mid-piece
- **THEN** the scrolling Staff mode positions that staff's notes from the clef in
  force at each note and shows the clef in effect at the playhead

#### Scenario: Timing scales with divisions and tempo
- **WHEN** two quarter notes follow one another at a known divisions value and
  tempo
- **THEN** the second note's start time equals the first note's start plus one
  quarter-note duration in milliseconds

#### Scenario: Rests are not played
- **WHEN** the measure contains a rest between two notes
- **THEN** the derived timeline contains the two notes and no entry for the rest

#### Scenario: Derived timing feeds audio playback
- **WHEN** the audio-output capability plays the piece
- **THEN** it sounds each derived note at its computed start and releases it after
  its computed duration

#### Scenario: Unpitched note takes its General MIDI percussion number
- **WHEN** an unpitched note in a percussion-classified score resolves to a
  part-list instrument whose `midi-unpitched` is 39
- **THEN** the derived note carries General MIDI percussion number 38 — the
  one-based element value minus one — timed like any other note

#### Scenario: A mixed score schedules only its pitched notes
- **WHEN** a score containing both pitched and unpitched notes is scheduled
- **THEN** only the pitched notes produce derived notes, exactly as before this
  change

#### Scenario: Unresolvable unpitched note is omitted
- **WHEN** an unpitched note's General MIDI number cannot be resolved
- **THEN** it produces no derived note, and the surrounding notes keep their
  computed times

#### Scenario: Staff mode places an unpitched note as written
- **WHEN** a percussion score's snare (written C5, General MIDI 38) is shown in
  the scrolling Staff mode
- **THEN** it is positioned on the third space from its written placement, and
  the General MIDI number in the MIDI slot plays no part in the position
