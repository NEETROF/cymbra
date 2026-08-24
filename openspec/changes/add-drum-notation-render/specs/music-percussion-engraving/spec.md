## ADDED Requirements

### Requirement: Percussion is engraved on a single percussion-clef staff without an armature

The notation renderers SHALL engrave a percussion score on a single five-line
staff opened with the **percussion clef glyph**, and SHALL NOT draw a key
signature (armature) — nor the cancelling naturals of a key change — on a
percussion staff, while drawing the time signature exactly as on any other
staff. This rule binds every notation surface: the app's scrolling Staff and
engraved Partition modes, and the console's browser renderer.

An unpitched part has no tonality: a declared `fifths` value is a leftover of
the exporter's defaults, and engraving sharps or flats on a drum staff is
wrong music, not faithfulness. The clef mapping SHALL be explicit — both
renderers currently default an unknown clef sign to treble, so without an
explicit mapping the percussion sign silently draws as a G clef, a defect no
test asserting merely "a clef is drawn" can catch.

#### Scenario: The percussion clef is drawn

- **WHEN** a percussion score whose first measure declares
  `<clef><sign>percussion</sign></clef>` is rendered
- **THEN** each system opens with the percussion clef glyph — not a treble
  clef

#### Scenario: No armature even when the file declares one

- **WHEN** a percussion score declaring `fifths` = 2 is rendered
- **THEN** no sharps are drawn at the head of any system and no key-change
  naturals are ever drawn

#### Scenario: The time signature still draws

- **WHEN** a percussion score in 4/4 is rendered
- **THEN** the time signature is engraved as it would be on a keyboard staff

### Requirement: An unpitched note is placed by its written position

The renderers SHALL place each unpitched note vertically from its written
position — `display-step` and `display-octave` — mapped to staff lines and
spaces exactly as a treble (G, line 2) staff maps pitched spellings, the
MusicXML convention for unpitched display placement (snare C5 on the third
space, kick F4 in the bottom space, closed hi-hat G5 just above the top line),
with ledger lines as for pitched notes. Placement SHALL NOT be derived from
the General MIDI number in any circumstance, and an unpitched note whose
General MIDI number is **unresolved SHALL still be engraved** at its written
position: omission-when-unresolved is a playback rule (a fabricated *sound*
is wrong) and does not transfer to engraving (the written *position* is
present and authoritative — a moderator judging a proposal must see the score
as written, not a version with heads silently missing).

A note written as an empty `<unpitched/>` resolves to the middle line (B4) at
parse time and SHALL be placed there. Foot-struck events are ordinary written
notes here: the kick (General MIDI 35/36) and the pedal hi-hat "chick" (44)
engrave at their written positions like any other note — the cascade's
full-width bar and its bar-encoding question are gameplay concepts that SHALL
NOT appear on a notation surface.

#### Scenario: The standard kit positions engrave as written

- **WHEN** a percussion score writes its snare at C5 and its kick at F4
- **THEN** the snare head sits on the third space and the kick head in the
  bottom space, and neither position is influenced by their General MIDI
  numbers

#### Scenario: An unresolved note is engraved, not dropped

- **WHEN** an unpitched note's instrument reference cannot be resolved to a
  General MIDI number
- **THEN** the note is still engraved at its written position (while the
  playback schedule continues to omit it)

#### Scenario: An empty unpitched placement takes the middle line

- **WHEN** a note is written as `<unpitched/>` with no display position
- **THEN** its head is engraved on the middle line

#### Scenario: The kick is a note on the staff, not a bar

- **WHEN** a percussion score containing kick notes is shown in a notation
  mode
- **THEN** each kick engraves as an ordinary note head at its written
  position — no full-width bar is drawn on any notation surface

### Requirement: Note heads follow one shared kit-piece classification

The renderers SHALL draw cymbal pieces with **x-form** note heads and drum
pieces with the ordinary oval heads, the x form following the note's duration
class like ordinary heads (filled x for quarter and shorter, the open x forms
for half and whole). An **open hi-hat** stroke (General MIDI 46) SHALL
additionally carry the conventional open mark — a small circle above the
head — so the open/closed distinction the file encodes as two General MIDI
numbers stays readable on the staff, as it already is in the cascade. A note
whose General MIDI number is unresolved takes the ordinary oval head.

The classification from General MIDI number to head class SHALL live in the
**shared crate**, derived once beside the resolved number itself and carried
to both painters through the existing geometry (the wasm JSON and the app
bridge). A painter SHALL NOT re-derive head classes from General MIDI ranges
of its own: the app and the console are independent implementations, and two
hand-maintained tables of the same knowledge is exactly how they drift. The
app's kit-view table (`drum_kit.dart`) remains the separate authority for the
**gameplay** question (lanes and drawn pieces — where to aim); the overlap between the
two tables — which pieces are cymbals — SHALL be pinned by a test so they
cannot disagree silently.

#### Scenario: Cymbals take x heads, drums take oval heads

- **WHEN** a percussion score with hi-hat, crash and snare notes is rendered
- **THEN** the hi-hat and crash heads are x-form and the snare heads are the
  ordinary oval, in the app and the console alike

#### Scenario: The open hi-hat is marked

- **WHEN** a score alternates closed (42) and open (46) hi-hat strokes
- **THEN** the open strokes carry the open mark above their x heads and the
  closed strokes do not

#### Scenario: Both painters read one classification

- **WHEN** the app and the console render the same percussion score
- **THEN** every note resolves to the same head class in both, because both
  consume the class the shared crate derived rather than deriving their own

#### Scenario: The gameplay and engraving tables agree where they overlap

- **WHEN** the drift-pinning test compares the kit-view roles with the crate's
  head classes
- **THEN** every General MIDI number the kit view treats as a cymbal
  classifies as an x head, and no drum-role number does

### Requirement: Two voices share the percussion staff legibly

The renderers SHALL lay out a two-voice percussion part on its single staff
with the two voices distinguishable and non-destructive of one another:

- **Stems**: an explicit `<stem>` in the file wins; when a note carries none,
  voice 1 stems up and voice 2 stems down — the drum-notation convention the
  hands/feet split (`hand-color-coding`) is keyed to.
- **Beams**: beam groups SHALL never merge notes of different voices.
- **Rests**: in a measure where both voices are present, rests SHALL be
  vertically displaced by voice — voice 1's rests above the middle line,
  voice 2's below — so a rest does not sit on the midline where the other
  voice's material runs; in a single-voice measure rests keep their ordinary
  midline placement.
- **Shared onsets**: when both voices strike the same instant, both engrave at
  the shared time column with their own stems, and when two heads would land
  on the same staff position one SHALL be offset horizontally so neither is
  hidden.

#### Scenario: Stems follow the file, then the voice

- **WHEN** a two-voice drum measure carries explicit stems on some notes and
  none on others
- **THEN** the explicit stems render as written, and the bare notes stem up in
  voice 1 and down in voice 2

#### Scenario: Rests are displaced by voice

- **WHEN** a measure has hi-hat eighths in voice 1 and a quarter rest in
  voice 2
- **THEN** the rest is drawn below the middle line, clear of the voice-1
  material

#### Scenario: A shared onset keeps both voices legible

- **WHEN** a kick (voice 2) and a hi-hat stroke (voice 1) share an onset
- **THEN** both heads engrave at the shared column, the hi-hat stemmed up and
  the kick stemmed down, and neither obscures the other

#### Scenario: Beams stay within a voice

- **WHEN** both voices carry beamed eighth runs in the same measure
- **THEN** each voice beams its own run and no beam spans the two voices
