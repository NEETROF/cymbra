## MODIFIED Requirements

### Requirement: Part And Multi-Staff Structure

The parser SHALL read the part list and, for a keyboard part, the number of staves
declared by `attributes/staves` (e.g. 2 for a grand staff). Every note,
direction, and clef that carries a `staff`/`number` SHALL be associated with the
correct staff; when no staff is indicated, the element SHALL default to staff 1.

The parser SHALL additionally read the part list's **instrument declarations** —
`<score-part>/<score-instrument>` and their `<midi-instrument>` children — and
retain, per instrument id, the General MIDI percussion number declared by
`<midi-unpitched>`, so an unpitched note can resolve the sound it denotes (see the
`music-percussion-notation` capability). A part list carrying no instrument
declaration SHALL parse exactly as before.

#### Scenario: Piano part has two staves
- **WHEN** the part declares `staves` of 2
- **THEN** the score document reports two staves for that part

#### Scenario: Notes routed to their staff
- **WHEN** a note declares `staff` 1 and another declares `staff` 2 in the same
  measure
- **THEN** each note event is associated with its declared staff

#### Scenario: Staff defaults when absent
- **WHEN** a note omits the `staff` element in a single-staff part
- **THEN** the note event is associated with staff 1

#### Scenario: Instrument declarations are retained
- **WHEN** the part list declares a `score-instrument` whose `midi-instrument`
  carries a `midi-unpitched` number
- **THEN** the document retains that number against the instrument's id

#### Scenario: A part list without instruments is unchanged
- **WHEN** the part list declares no `score-instrument`
- **THEN** parsing succeeds and the document carries no instrument declarations

### Requirement: Starting Attributes Extraction

The parser SHALL extract the starting musical attributes of a part: the
`divisions` (ticks per quarter note), one clef per staff (sign and line,
identified by the clef `number`), the key signature (`fifths`, with mode when
present), and the time signature (beats and beat-type). When a measure restates
attributes, the most recent value SHALL apply to subsequent notes. The document
SHALL keep the *initial* clef per staff, and SHALL additionally record any clef
changes per measure so a renderer can switch clefs mid-piece.

A clef sign SHALL be represented by a type able to carry a **multi-character**
sign, so that `percussion` is admitted alongside the single-letter signs. A sign
the parser does not recognise SHALL leave that staff at its default clef rather
than failing the parse.

#### Scenario: Per-staff clefs on a grand staff
- **WHEN** the first measure declares clef number 1 as treble (G/2) and clef
  number 2 as bass (F/4)
- **THEN** the score document reports a treble clef for staff 1 and a bass clef
  for staff 2

#### Scenario: Mid-piece clef change recorded per measure
- **WHEN** a staff is in treble clef in the first measure and a later measure
  declares a bass clef for that staff
- **THEN** the document keeps the initial treble clef for the staff and records
  the bass-clef change on that later measure

#### Scenario: Initial key and time signature
- **WHEN** the first measure declares key `fifths` of -3 and time 3/4
- **THEN** the score document reports key fifths=-3 and time signature=3/4

#### Scenario: Divisions drive duration interpretation
- **WHEN** `divisions` is 4 and a note has `duration` 4
- **THEN** that note is interpreted as one quarter note (one division-beat)

#### Scenario: Percussion clef is admitted
- **WHEN** a measure declares a clef whose sign is `percussion`
- **THEN** the score document reports a percussion clef on that staff

#### Scenario: Unrecognised clef sign degrades
- **WHEN** a measure declares a clef sign the parser does not recognise
- **THEN** parsing succeeds and that staff keeps its default clef

### Requirement: Note Extraction

For each measure the parser SHALL extract its note events in document order. Each
note event SHALL carry exactly one of: its pitch (step, octave, and alteration),
its **unpitched written position** (display-step and display-octave, for a
percussion note), or a rest flag. Each note event SHALL further carry: its
duration in divisions; its note-type when present (e.g. half, eighth); the
number of augmentation dots; its accidental when present; its voice; and its
staff. A note carrying `<chord/>` SHALL be flagged as a chord member of the
preceding note. A note referencing an `<instrument>` SHALL retain that instrument
id, so it can be resolved against the part list's instrument declarations.

An unpitched note's written position SHALL NOT be exposed as a pitch, because it
denotes a staff placement rather than a sounding pitch; deriving a MIDI number from
it would fabricate a frequency the score never states.

A note's alteration SHALL be taken from its explicit `<alter>` element when
present (defaulting to 0 when absent), EXCEPT that for a document with no explicit
alteration anywhere the key signature is applied per the **Key-Signature Pitch
Inference for Unmarked Scores** requirement. Downstream pitch computation (MIDI
number derivation for playback, ambitus scanning, and staff rendering) relies on
this alteration field alone and SHALL NOT re-derive alteration from the key
signature.

#### Scenario: Pitched note extracted
- **WHEN** a note declares step C, octave 5, duration one quarter, voice 1, staff 1
- **THEN** a note event is produced with that pitch, duration, voice, and staff,
  and is not flagged as a chord member

#### Scenario: Altered pitch and accidental
- **WHEN** a note declares `alter` -1 and an `accidental` of flat
- **THEN** the note event reports alteration -1 and accidental=flat

#### Scenario: Dotted note
- **WHEN** a note carries one `dot` element
- **THEN** the note event reports a dot count of 1

#### Scenario: Rest extracted
- **WHEN** a note carries the `<rest/>` element
- **THEN** a note event is produced flagged as a rest with its duration, voice,
  and staff

#### Scenario: Unpitched note extracted
- **WHEN** a note carries `<unpitched>` with display-step E and display-octave 5,
  a duration, a voice and a staff
- **THEN** a note event is produced carrying that written position, duration,
  voice and staff, with no pitch and not flagged as a rest

#### Scenario: Instrument reference retained
- **WHEN** a note carries an `<instrument>` element
- **THEN** the note event retains that instrument id

### Requirement: Derived Playback Timing

The system SHALL derive a playback timing from a parsed score so the time-based
render modes (waterfall and scrolling staff) can present the selected piece and so
the audio synthesizer can play it. For each non-rest note it SHALL compute a start
time and duration in milliseconds from the note's running division position and a
tempo (taken from a `metronome` direction when present, otherwise a default), and a
MIDI number obtained as follows: for a **pitched** note, computed from the note's
step, octave and alteration; for an **unpitched** note, the General MIDI percussion
number resolved from the part list's instrument declarations. An unpitched note
whose number cannot be resolved SHALL be omitted rather than emitted with a
fabricated number. Chord members SHALL share the onset of the note they attach to;
rests SHALL NOT produce a played note. Each derived note SHALL also carry its
staff, beam states and the clef in effect, so the scrolling Staff mode can lay out
a grand staff with beamed groups and position notes by the clef in force (honouring
mid-piece clef changes). The derivation itself produces no sound — it is a
timing/pitch model; audible playback is rendered by the audio-output synthesizer
that consumes this timing (see the `audio-output` capability).

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
- **WHEN** an unpitched note resolves to a part-list instrument whose
  `midi-unpitched` is 38
- **THEN** the derived note carries MIDI number 38, timed like any other note

#### Scenario: Unresolvable unpitched note is omitted
- **WHEN** an unpitched note's General MIDI number cannot be resolved
- **THEN** it produces no derived note, and the surrounding notes keep their
  computed times
