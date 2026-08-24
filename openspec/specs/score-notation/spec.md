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

The parser SHALL read the part list and, for a keyboard part, the number of staves
declared by `attributes/staves` (e.g. 2 for a grand staff). Every note,
direction, and clef that carries a `staff`/`number` SHALL be associated with the
correct staff; when no staff is indicated, the element SHALL default to staff 1.

The parser SHALL additionally read the part list's **instrument declarations** —
`<score-part>/<score-instrument>` and their `<midi-instrument>` children — and
retain, per instrument id, the General MIDI percussion number derived from
`<midi-unpitched>`. That element is **one-based** (MusicXML numbers MIDI notes
1–128), so the General MIDI number is the element value minus one; the full
resolution rule lives in the `music-percussion-notation` capability, which this
table exists to serve. A part list carrying no instrument declaration SHALL parse
exactly as before.

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
  carries a `midi-unpitched` of 39
- **THEN** the document retains General MIDI percussion number 38 — the one-based
  element value minus one — against the instrument's id

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

An `<unpitched/>` carrying no display position is **valid** MusicXML denoting the
middle staff line: the note event SHALL be produced with that default placement
rather than dropped — a minimally-authored single-line part writes every note
this way, and dropping them would silently erase the whole part.

A degenerate `<note>` — one carrying none of `<pitch>`, `<unpitched>` or `<rest>`
— fits none of the three forms and SHALL be skipped: no note event is produced,
its declared duration still advances the running position so the surrounding
notes keep their written times, and parsing succeeds. This follows the parser's
degrade-don't-fail posture — rejecting a whole file over one malformed note
would be a worse outcome than dropping the note.

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

#### Scenario: Empty unpitched element defaults to the middle line
- **WHEN** a note carries an empty `<unpitched/>` with no display position
- **THEN** a note event is produced with the middle-line placement, the MusicXML
  default

#### Scenario: Degenerate note is skipped, parsing succeeds
- **WHEN** a note carries none of pitch, unpitched or rest
- **THEN** no note event is produced for it, its declared duration still advances
  the running position, and parsing succeeds

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
narrower than a fixed minimum floor. Every time column SHALL additionally
receive at least a fixed minimum column advance regardless of how short its
duration is, so dense passages (16ths and shorter) cannot collapse into
unreadably tight spacing. Spacing SHALL be computed over the union of
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

#### Scenario: Short columns respect the minimum column advance
- **WHEN** a measure contains a run of 16th (or shorter) notes
- **THEN** each of its time columns is allotted at least the minimum column
  advance, so the measure's `min_width` grows linearly with the number of such
  columns instead of compressing them

#### Scenario: Longer columns are unaffected by the column floor
- **WHEN** a time column's non-linear spacing already exceeds the minimum column
  advance (e.g. a quarter note or longer)
- **THEN** its width is the non-linear value, unchanged by the floor

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
begin when the next measure would overflow *or* a fixed maximum of **three**
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
- **WHEN** more than three measures would fit width-wise on one line
- **THEN** the engine still wraps to a new system after the third measure,
  preserving order

#### Scenario: Grand staff kept together
- **WHEN** the part has two staves
- **THEN** each system carries both staves so treble and bass render as one
  grand staff

#### Scenario: Oversized measure on its own system
- **WHEN** a single measure's `min_width` exceeds the available width
- **THEN** that measure occupies a system by itself

#### Scenario: All measures fit on one system
- **WHEN** the cumulative `min_width` of up to three measures is within the
  available width and no more measures exist
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
(system) and the upcoming line visible, advancing **once per staff line** rather
than per measure: the vertical position depends only on which system the cursor
is in, so the view stays put while the cursor crosses measures of the same line
and moves when the playhead reaches a new line (no per-measure back-and-forth).
The scroll target SHALL anchor the current system in the upper part of the
viewport (near the top, not centred) so that the next system is visible below it
whenever one exists and the viewport can show more than one system — look-ahead
is provided by the scroll position itself, in place and at full size. When the
whole score fits in the viewport, no scrolling SHALL occur.

#### Scenario: View follows the cursor per line
- **WHEN** the playhead moves to a measure on a new system
- **THEN** the view scrolls once so that system sits in the upper part of the
  viewport

#### Scenario: Next line visible below the current line
- **WHEN** the playhead is on a system that is not the last and the viewport is
  tall enough for more than one system
- **THEN** the following system is visible below the current one, at full size,
  in its place in the score

#### Scenario: No scroll within a line
- **WHEN** the cursor crosses measures within the same system
- **THEN** the view does not scroll (the line stays put)

#### Scenario: No scroll when it all fits
- **WHEN** every system fits within the viewport
- **THEN** the view does not scroll

### Requirement: Key-Signature Pitch Inference for Unmarked Scores

The parser SHALL apply the key signature (`<fifths>`) to every pitched note of a
document that contains no explicit alteration on any note (no `<alter>` element
and no `<accidental>` element anywhere in the score), setting each note's
alteration to the value the key signature dictates for its step (sharps order
`F C G D A E B` at +1; flats order `B E A D G C F` at -1; count clamped to 7;
unaffected steps 0). The key signature in force SHALL be the one declared for that
note's measure, so mid-piece key changes are honored.

This inference is deliberately narrow: it fires **only** for scores that carry no
alteration data at all — the case where a minimal or non-conforming exporter
relied on the drawn armure alone. As soon as a score contains a single explicit
`<alter>` or `<accidental>`, inference is disabled for the entire document and
every note's alteration is taken verbatim from its `<alter>` (0 when absent).

Rationale: conforming exporters (MuseScore/Finale/Sibelius) always emit `<alter>`
on every altered note and encode running measure accidentals by writing an
accidental once and leaving later same-pitch notes bare. Re-deriving those bare
notes from the key signature would mis-sound legitimately natural notes, so
inference must never run on a score that carries any alteration data.

#### Scenario: Key signature applied to a score with no alteration data

- **WHEN** a score's only pitch information is `<step>`/`<octave>` (no `<alter>`
  and no `<accidental>` anywhere) under a key signature of three flats
  (`fifths` = -3)
- **THEN** each B, E, and A note reports alteration -1 (B♭, E♭, A♭) and every
  other step reports 0, so the derived MIDI numbers sound the key signature

#### Scenario: A single explicit alteration disables inference document-wide

- **WHEN** a score under three flats has one note with an explicit `<alter>` (or
  `<accidental>`) and other notes with only `<step>`/`<octave>`
- **THEN** the explicitly-marked note keeps its `<alter>` value and every other
  note reports alteration 0 (natural) — the key signature is NOT inferred for the
  unmarked notes, matching how conforming exporters encode pitch

#### Scenario: Mid-piece key change is honored in an unmarked score

- **WHEN** an unmarked score (no `<alter>`/`<accidental>` anywhere) declares
  `fifths` = -3 in its first measure and `fifths` = 0 in a later measure
- **THEN** B notes in the first measure report alteration -1 (B♭) while B notes
  in the later measure report 0 (B natural)

#### Scenario: Conforming score is unchanged

- **WHEN** a score emits `<alter>` on its altered notes (as MuseScore does),
  including bare notes that the exporter intends as natural within a measure's
  accidental context
- **THEN** every note's alteration equals its explicit `<alter>` (0 when absent),
  identical to the behavior before this change — no key-signature inference runs

### Requirement: Played-System Dimming

In Partition mode the renderer SHALL dim (reduce the opacity of) every system
that lies entirely before the system containing the playhead while a playhead is
active, so the current and upcoming lines stand out.
The current system and all following systems SHALL render at full opacity. When
no playhead is active (stopped, or the score has no timing) all systems SHALL
render at full opacity.

#### Scenario: Fully played lines are dimmed
- **WHEN** the playhead is on system N during playback
- **THEN** systems 0..N-1 render dimmed while systems N and beyond render at
  full opacity

#### Scenario: No dimming without a playhead
- **WHEN** no playback position exists (score stopped or untimed)
- **THEN** every system renders at full opacity

### Requirement: Current-Measure Highlight

In Partition mode the measure containing the playhead SHALL be emphasised with a
subtle background wash spanning the system's staves for that measure's width, in
addition to the playhead cursor line. The wash SHALL move with the playhead from
measure to measure, SHALL NOT obscure or recolor the engraved glyphs, and SHALL
NOT be shown when no playhead is active.

#### Scenario: Active measure carries a wash
- **WHEN** the playhead is inside a measure during playback
- **THEN** that measure's background is washed with a subtle accent colour while
  neighbouring measures keep the plain background

#### Scenario: Wash follows the playhead
- **WHEN** the playhead crosses into the next measure
- **THEN** the wash moves to the new measure

#### Scenario: No wash when stopped
- **WHEN** no playback position exists
- **THEN** no measure wash is drawn

### Requirement: Notation Size Setting

The player SHALL offer a score size setting with three levels (small, medium,
large; medium is the default) that scales the engraved notation in **both**
notation views. In the Partition (vertical) view the setting SHALL scale the
staff space — and with it every glyph, spacing and system dimension — and
systems SHALL re-wrap so the enlarged line still fits the viewport width. In the
Portée (horizontal scrolling staff) view the setting SHALL scale the staff and
glyphs and adjust the visible time window proportionally, so note size and
horizontal spacing grow together. The setting SHALL apply immediately when
changed.

#### Scenario: Larger size re-wraps the Partition
- **WHEN** the user switches the score size from medium to large in Partition
  mode
- **THEN** the notation renders with a larger staff space and systems re-wrap to
  fit the viewport width (fewer measures per line when needed)

#### Scenario: Larger size scales the Portée view
- **WHEN** the user switches the score size from medium to large in the
  horizontal staff view
- **THEN** the staff, glyphs and note spacing render proportionally larger (a
  shorter time window spans the same width)

#### Scenario: Default size unchanged
- **WHEN** the user has never touched the score size setting
- **THEN** both views render exactly as at the medium (1.0) scale

### Requirement: Notation Paper Theme

The player SHALL offer a notation theme setting with two values — the app's
dark surface (default) and a paper-like light theme — applied to both notation
views (Partition and Portée). The paper theme SHALL render the engraving
dark-on-light (near-black ink on an ivory background) with hand, correct and
accent colours darkened to keep at least a 4.5:1 contrast against the paper
background, so the per-hand colour coding survives the light background. The
setting SHALL apply immediately when changed and SHALL NOT affect the
Synthesia view or the rest of the app's chrome.

#### Scenario: Paper renders dark-on-light with adapted colours
- **WHEN** the user selects the paper theme and opens a notation view
- **THEN** the background is light, glyphs are near-black, and note heads use
  the darkened per-hand palette

#### Scenario: Dark stays the default
- **WHEN** the user has never touched the notation theme
- **THEN** both notation views render on the dark surface exactly as before

#### Scenario: Chrome is unaffected
- **WHEN** the paper theme is active
- **THEN** the top bar, transport and keyboard keep the app's dark theme

