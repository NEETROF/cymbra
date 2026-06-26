# score-notation Specification

## Purpose
TBD - created by archiving change musicxml-parsing-and-geometry. Update Purpose after archive.
## Requirements
### Requirement: MusicXML Asset Loading

The system SHALL load an uncompressed MusicXML document (`.musicxml` or `.xml`)
from the Flutter application assets as raw bytes and hand those bytes to the Rust
engine for parsing. Asset access SHALL go through an injectable source seam so
that state and widgets are testable without the native library or a real asset
bundle.

#### Scenario: Asset loaded and forwarded to the engine
- **WHEN** the app requests a bundled MusicXML asset by path
- **THEN** the raw bytes are read and passed to the Rust parser, which returns a
  structured score document

#### Scenario: Source seam overridden in tests
- **WHEN** a test provides a fake score-asset source returning in-memory bytes
- **THEN** the notation state is produced without touching the asset bundle or
  native library

### Requirement: Streaming MusicXML Parsing

The Rust engine SHALL parse MusicXML using a streaming, event-based (SAX-style)
reader rather than building a full in-memory DOM, so that large multi-megabyte
scores parse with bounded memory. The parser SHALL accept the document as bytes
or a UTF-8 string.

#### Scenario: Large document parses without full-DOM allocation
- **WHEN** a large MusicXML document is parsed
- **THEN** the parser consumes it as a stream of events and produces the score
  document without materializing the entire XML tree in memory

#### Scenario: Bytes and string inputs are equivalent
- **WHEN** the same document is provided once as bytes and once as a UTF-8 string
- **THEN** the resulting score document is identical

### Requirement: Score Metadata Extraction

The parser SHALL extract score metadata when present: the work title and the
composer. Missing metadata fields SHALL be represented as absent (empty/none)
rather than causing a parse failure.

#### Scenario: Title and composer present
- **WHEN** the document declares a work title and a `composer` creator
- **THEN** the score document reports that title and composer

#### Scenario: Metadata absent
- **WHEN** the document omits title or composer
- **THEN** the corresponding fields are empty/none and parsing still succeeds

### Requirement: Part And Multi-Staff Structure

The parser SHALL read the part list and, for a piano part, the number of staves
declared by `attributes/staves` (e.g. 2 for a grand staff). Every note,
direction, and clef that carries a `staff`/`number` SHALL be associated with the
correct staff; when no staff is indicated, the element SHALL default to staff 1.

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

### Requirement: Starting Attributes Extraction

The parser SHALL extract the starting musical attributes of a part: the
`divisions` (ticks per quarter note), one clef per staff (sign and line,
identified by the clef `number`), the key signature (`fifths`, with mode when
present), and the time signature (beats and beat-type). When a measure restates
attributes, the most recent value SHALL apply to subsequent notes. The document
SHALL keep the *initial* clef per staff, and SHALL additionally record any clef
changes per measure so a renderer can switch clefs mid-piece.

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

### Requirement: Measure Time Navigation

The parser SHALL maintain a running time position within each measure and SHALL
honour `backup` and `forward` elements, which move that position backward or
forward by their `duration`. This lets a single part interleave multiple voices
and staves (e.g. write the treble staff for a measure, `backup` to the bar
start, then write the bass staff) and still place every note event at the
correct time position.

#### Scenario: Backup rewinds the running position
- **WHEN** the treble staff notes fill a 3/4 measure and a `backup` of one full
  measure follows
- **THEN** the running time position returns to the measure start so the bass
  staff notes are placed from the beginning of the bar

#### Scenario: Forward advances over an implicit gap
- **WHEN** a `forward` element of one beat appears
- **THEN** the running time position advances by one beat before the next note

#### Scenario: Chord members do not advance time
- **WHEN** a note carries the `<chord/>` element
- **THEN** the running time position is unchanged for that note (it sounds with
  the preceding note)

### Requirement: Note Extraction

For each measure the parser SHALL extract its note events in document order. Each
note event SHALL carry: its pitch (step, octave, and alteration) or a rest flag;
its duration in divisions; its note-type when present (e.g. half, eighth); the
number of augmentation dots; its accidental when present; its voice; and its
staff. A note carrying `<chord/>` SHALL be flagged as a chord member of the
preceding note.

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

### Requirement: Tie Extraction

The parser SHALL detect tied notes via the note `tie` element (`start`/`stop`)
and mark each note event with whether it begins and/or ends a tie, so the
renderer can draw the tie and the playback layer can treat tied notes as one
sustained sound.

#### Scenario: Tie start and stop
- **WHEN** one note carries `tie type=start` and the following note of the same
  pitch carries `tie type=stop`
- **THEN** the first note event is flagged tie-start and the second tie-stop

### Requirement: Slur Extraction

The parser SHALL detect phrasing slurs via the note `slur` element
(`start`/`stop`) and mark each note event with whether it begins and/or ends a
slur, so the renderer can draw the phrase arc. A slur is distinct from a tie: it
spans notes of differing pitch.

#### Scenario: Slur start and stop
- **WHEN** one note carries `slur type=start` and a later note in the phrase
  carries `slur type=stop`
- **THEN** the first note event is flagged slur-start and the later one slur-stop

### Requirement: Tuplet (Time-Modification) Extraction

The parser SHALL read `time-modification` (`actual-notes`, `normal-notes`) so
that tuplets such as triplets (3 in the time of 2) are represented, preserving
the played `duration` while recording the tuplet ratio for display and timing.

#### Scenario: Triplet ratio captured
- **WHEN** notes carry `time-modification` with actual-notes 3 and normal-notes 2
- **THEN** their note events record a 3:2 tuplet ratio

### Requirement: Stem And Beam Extraction

The parser SHALL read each note's `stem` direction (up/down) when present and its
`beam` states (begin/continue/end) so the renderer can draw stems and beam
groups.

#### Scenario: Stem direction captured
- **WHEN** a note declares `stem` up and another declares `stem` down
- **THEN** their note events record stem directions up and down respectively

#### Scenario: Beam group captured
- **WHEN** consecutive eighth notes declare `beam` begin, continue, and end
- **THEN** their note events record the corresponding beam states

### Requirement: Lyric Extraction

The parser SHALL read note `lyric` content (`syllabic` and `text`) and attach it
to the note event, so vocal/lyric text can be rendered under the staff.

#### Scenario: Syllable attached to a note
- **WHEN** a note carries a lyric with syllabic=begin and text "Dans"
- **THEN** the note event carries that lyric syllable

### Requirement: Direction Extraction

The parser SHALL extract measure `direction` elements and associate each with its
staff and time position: tempo/expression `words` (e.g. "Andantino", "dolce"),
`dynamics` (e.g. pp), `wedge` hairpins (crescendo/diminuendo, with start/stop),
and `metronome` markings. Unknown direction types SHALL be ignored rather than
causing a parse failure.

#### Scenario: Words direction captured
- **WHEN** a measure contains a direction with words "Andantino"
- **THEN** a direction is recorded with that text at its time position

#### Scenario: Dynamics captured
- **WHEN** a measure contains a `dynamics` direction of pp
- **THEN** a dynamics direction of pp is recorded

#### Scenario: Crescendo wedge captured
- **WHEN** a measure contains a `wedge type=crescendo` followed later by a
  `wedge type=stop`
- **THEN** a crescendo hairpin is recorded spanning from start to stop

#### Scenario: Unknown direction ignored
- **WHEN** a measure contains a direction type the parser does not model
- **THEN** it is ignored and parsing continues

### Requirement: Non-linear Measure Geometry

The geometry engine SHALL compute, for every measure, a minimum width
(`min_width`) derived from the measure's note density using a non-linear spacing
function: shorter durations SHALL receive proportionally more space than a
purely linear mapping of duration to width, and a measure SHALL never be
narrower than a fixed minimum floor. Spacing SHALL be computed over the union of
time positions across all staves and voices of the measure, so the two staves of
a grand staff stay horizontally aligned; chord members and notes that share a
time position SHALL NOT add horizontal space.

#### Scenario: Denser measures are wider
- **WHEN** measure A contains eight eighth notes and measure B contains two half
  notes, with equal total duration
- **THEN** `min_width(A)` is greater than `min_width(B)`

#### Scenario: Spacing is sub-linear in duration
- **WHEN** comparing a quarter note to a note of twice its duration
- **THEN** the width allotted to the longer note is more than the quarter's but
  less than twice the quarter's (sub-linear growth)

#### Scenario: Both staves share one horizontal spacing
- **WHEN** staff 1 and staff 2 have notes at the same time positions in a measure
- **THEN** those positions contribute a single shared column of spacing (not
  double-counted)

#### Scenario: Minimum width floor respected
- **WHEN** a measure contains a single whole-measure rest
- **THEN** its `min_width` is at least the fixed minimum floor

### Requirement: System Layout

The geometry engine SHALL group measures into systems (justified staff lines) for
a given available width: measures SHALL be appended to the current system while
their cumulative `min_width` fits the available width, and a new system SHALL
begin when the next measure would overflow *or* a fixed maximum number of
measures per system is reached (so dense scores stay legible on a wide viewport).
A single measure whose `min_width` exceeds the available width SHALL occupy its
own system. Each system SHALL carry the staves of the part (e.g. treble + bass
for piano) so a grand staff is laid out together. The returned layout SHALL
preserve measure order.

#### Scenario: Measures wrap into multiple systems
- **WHEN** the cumulative `min_width` of measures exceeds the available width
- **THEN** the engine starts a new system at the first measure that would
  overflow, preserving order

#### Scenario: Cap on measures per system
- **WHEN** more measures than the per-system maximum would fit width-wise on one
  line
- **THEN** the engine still wraps to a new system at the cap, preserving order

#### Scenario: Grand staff kept together
- **WHEN** the part has two staves
- **THEN** each system carries both staves so treble and bass render as one
  grand staff

#### Scenario: Oversized measure on its own system
- **WHEN** a single measure's `min_width` exceeds the available width
- **THEN** that measure occupies a system by itself

#### Scenario: All measures fit on one system
- **WHEN** the cumulative `min_width` of all measures is within the available
  width
- **THEN** the engine returns a single system containing every measure in order

### Requirement: Malformed Input Handling

The system SHALL fail gracefully on malformed or non-MusicXML input: the parser
SHALL return a recoverable error rather than panicking, and the Flutter layer
SHALL surface an error/empty state without crashing.

#### Scenario: Malformed XML rejected safely
- **WHEN** the engine is given bytes that are not well-formed XML
- **THEN** it returns an error result and does not panic

#### Scenario: Non-MusicXML document rejected safely
- **WHEN** the engine is given well-formed XML that is not a MusicXML score
- **THEN** it returns an error or an empty score document without crashing

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

The system SHALL derive a playback timing from a parsed score so the time-based
render modes (waterfall and scrolling staff) can present the selected piece and so
the audio synthesizer can play it. For each non-rest note it SHALL compute a MIDI
pitch from the note's step, octave and alteration, and a start time and duration
in milliseconds from the note's running division position and a tempo (taken from
a `metronome` direction when present, otherwise a default). Chord members SHALL
share the onset of the note they attach to; rests SHALL NOT produce a played note.
Each derived note SHALL also carry its staff, beam states and the clef in effect,
so the scrolling Staff mode can lay out a grand staff with beamed groups and
position notes by the clef in force (honouring mid-piece clef changes). The
derivation itself produces no sound — it is a timing/pitch model; audible playback
is rendered by the audio-output synthesizer that consumes this timing (see the
`audio-output` capability).

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

### Requirement: Per-Measure Playback Timing

The derivation SHALL compute, for each measure, its playback start time in
milliseconds, using the same running division position and tempo as the note
timeline (a measure's start is its accumulated start-divisions times the
milliseconds-per-division). This lets any playback position be mapped to the
measure it falls in and the fraction elapsed within that measure.

#### Scenario: Measure start times derived
- **WHEN** a score is derived for playback
- **THEN** each measure has a start time in ms equal to its accumulated
  start-divisions × ms-per-division, and the first measure starts at 0

#### Scenario: Position maps to a measure and fraction
- **WHEN** the playhead is at a time within a measure's span
- **THEN** that measure is identified as current and the fraction elapsed within
  it is the position between its start and the next measure's start

### Requirement: Partition Playhead Cursor

In Partition mode the painter SHALL draw a playhead cursor — a vertical line —
at the current playback position, located on the system and measure that contain
the playhead and at the horizontal fraction corresponding to the elapsed time
within that measure. The cursor SHALL move as the playhead advances and SHALL
freeze when Wait Mode freezes the playhead (it reads the same position). The
cursor SHALL span the staves drawn for that system.

#### Scenario: Cursor placed at the playback position
- **WHEN** Partition mode is shown and the playhead is partway through a measure
- **THEN** a vertical cursor is drawn on that measure at the matching horizontal
  fraction, spanning the system's staves

#### Scenario: Cursor advances with playback
- **WHEN** the playhead advances across measures and systems
- **THEN** the cursor moves to the measure/system containing the new position

#### Scenario: Cursor freezes with Wait Mode
- **WHEN** Wait Mode freezes the playhead at an awaited note
- **THEN** the cursor stops at that position until playback advances

### Requirement: Partition Note Highlighting

In Partition mode the note(s) at the playhead SHALL be emphasised relative to the
rest of the score so the reader sees what to play now: a note whose time window
contains the playhead and whose key is held SHALL read as "correct" (green),
otherwise it SHALL be drawn brighter than its normal colour. Notes away from the
playhead SHALL keep their normal rendering. (The base colour of each note head is
set per hand by the hand-colour-coding capability.)

#### Scenario: Current note emphasised as correct when held
- **WHEN** a note's time window contains the playhead and its key is held
- **THEN** that note head is drawn in the correct (green) colour

#### Scenario: Current note emphasised when not held
- **WHEN** a note's time window contains the playhead and its key is not held
- **THEN** that note head is drawn brighter than its normal colour

#### Scenario: Other notes are not emphasised
- **WHEN** a note is not at the playhead
- **THEN** it is drawn in its normal (per-hand) colour

### Requirement: Partition Auto-Scroll Per Line

In Partition mode the view SHALL scroll automatically to keep the current line
(system) visible, advancing **once per staff line** rather than per measure: the
vertical position depends only on which system the cursor is in, so the view stays
put while the cursor crosses measures of the same line and moves when the playhead
reaches a new line (no per-measure back-and-forth). When the whole score fits in
the viewport, no scrolling SHALL occur. Look-ahead is provided by the next-line
overlay, not by scrolling ahead.

#### Scenario: View follows the cursor per line
- **WHEN** the playhead moves to a measure on a new system
- **THEN** the view scrolls once so that system is visible

#### Scenario: No scroll within a line
- **WHEN** the cursor crosses measures within the same system
- **THEN** the view does not scroll (the line stays put)

#### Scenario: No scroll when it all fits
- **WHEN** every system fits within the viewport
- **THEN** the view does not scroll

### Requirement: Next-Line Preview Overlay

In Partition mode the player SHALL show a small "next line" overlay — the first
measure of the upcoming system, engraved at the same scale as the main view —
pinned to the top-left of the viewport, so the reader can see what comes next.
The overlay SHALL appear only when it adds information: the playhead is past the
middle of the current line, a next system exists, AND that next system is not
already visible in the viewport (so it never covers the score when the next line
is already shown below the current one). It SHALL be hidden otherwise — early in
the line, on the last line, or whenever the next line is already on screen.

#### Scenario: Overlay appears when the next line is off-screen
- **WHEN** the playhead passes the middle of the current line, more lines follow,
  and the next line is not visible in the viewport
- **THEN** the first measure of the next line is shown in a top-left overlay

#### Scenario: Hidden when the next line is already visible
- **WHEN** the viewport already shows the next line below the current one
- **THEN** no next-line overlay is shown (the score is not covered)

#### Scenario: Hidden early in the line
- **WHEN** the playhead is in the first half of the current line
- **THEN** no next-line overlay is shown

#### Scenario: Hidden on the last line
- **WHEN** the cursor is on the last system of the score
- **THEN** no next-line overlay is shown

