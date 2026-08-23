# score-notation — delta for add-repeat-unrolling

## ADDED Requirements

### Requirement: Repeat Structure Extraction

The parser SHALL extract the score's repeat structure into the notation model
while keeping the measure list in written order. It SHALL capture: repeat
barlines from `<barline><repeat>` (direction forward/backward and the `times`
attribute, with the barline style), volta brackets from `<ending>` (the
number list and start/stop/discontinue), measure-repeat references from
`<measure-style><measure-repeat>` (the written measure whose content the `%`
measure replays, resolved transitively for chained `%` measures up to a
bounded depth), and jump markers — segno, coda, to-coda, and the
D.C./D.S./Fine words together with their `<sound>` attributes
(`dacapo`, `dalsegno`, `tocoda`, `fine`, `forward-repeat`). Extraction SHALL
NOT change any existing consumer's view of the written measures.

#### Scenario: Repeat barlines captured

- **WHEN** a measure opens with a forward repeat and a later measure closes
  with a backward repeat (`times="2"`)
- **THEN** the two measures carry the forward and backward repeat marks (with
  the repeat count) and the measure list still contains each written measure
  exactly once

#### Scenario: Volta brackets captured

- **WHEN** the score marks measure 8 with ending numbers "1" (start+stop) and
  measure 9 with ending number "2"
- **THEN** the model records a volta 1 bracket on measure 8 and a volta 2
  bracket on measure 9

#### Scenario: Measure-repeat reference resolved

- **WHEN** measure 4 carries `<measure-repeat type="start">1</measure-repeat>`
  and no notes
- **THEN** the model records that measure 4 replays measure 3's content, and
  measure 4's own note list stays empty (the sign is engraved, not the notes)

#### Scenario: Malformed chains stay bounded

- **WHEN** `%` measures reference each other beyond the resolution depth cap
- **THEN** the unresolvable measure keeps no replay reference (it plays empty,
  the pre-change behavior) and parsing still succeeds

### Requirement: Playback Order Unrolling

The engine SHALL compute, from the extracted repeat structure, the **playback
order**: the sequence of written-measure passes a performer would play. The
resolution SHALL honour forward/backward repeats (including `times`), select
volta brackets by pass number (passes beyond the listed brackets replay the
last bracket), and follow at most one D.C./D.S. jump — with repeats not
re-taken after the jump unless the score's `<sound>` attributes say
otherwise, ending at Fine or the coda when marked. The unroll SHALL be
computed once in the shared engine crate and consumed by every derivation
(the app's, the browser renderer's, and the server audio render) — never
re-implemented per consumer. The unroll SHALL be capped (total played
measures bounded by a fixed multiple of the written count and an absolute
ceiling); on any inconsistency — unmatched backward repeat, jump target
missing, cap exceeded — the playback order SHALL fall back to the written
order one-to-one, which is the pre-change behavior.

#### Scenario: Simple repeat unrolled

- **WHEN** measures 1–4 sit between a forward repeat and a backward repeat
- **THEN** the playback order is 1,2,3,4,1,2,3,4 followed by the rest of the
  piece

#### Scenario: Voltas selected per pass

- **WHEN** a repeated section ends with volta 1 on measure 8 and volta 2 on
  measure 9
- **THEN** the first pass plays …7,8 and jumps back, and the second pass
  plays …7,9 — measure 8 is never played on the second pass

#### Scenario: D.C. al Fine honoured once

- **WHEN** the last measure carries D.C. al Fine and an earlier measure is
  marked Fine
- **THEN** playback returns to the first measure exactly once and ends at the
  Fine measure, without re-taking inner repeats

#### Scenario: Malformed structure degrades to written order

- **WHEN** the score contains a backward repeat with no matching forward
  repeat and no sane resolution
- **THEN** the playback order equals the written order (every measure once, in
  sequence) and no error is surfaced to the player

### Requirement: Played-To-Written Measure Mapping

The derivation SHALL expose, alongside the played-measure timing table, the
written measure index of every played slot, so that any playhead position can
be mapped to the written measure being performed. The Partition cursor, the
measure rewind and practice-range interpretation SHALL use this mapping: the
cursor highlights the written measure of the current played slot (jumping
backward on a repeat), rewind steps through played slots, and taps on the
Partition keep selecting written measures.

#### Scenario: Cursor returns to the repeated section

- **WHEN** playback crosses a backward repeat
- **THEN** the next played slot maps to the repeat's first written measure and
  the Partition cursor jumps back to it

#### Scenario: Rewind steps through played slots

- **WHEN** the player rewinds one measure while inside the second pass of a
  repeat
- **THEN** the playhead lands on the previous played slot (the same written
  section, second pass), not on the first pass

### Requirement: Repeat Notation Engraving

The app's notation renderers SHALL engrave the repeat structure. The
Partition (page) view SHALL draw written order once with repeat barlines
(thick/thin lines plus dots on the repeated side), volta brackets with their
numbers, the `%` sign on measure-repeat measures, and the segno, coda and
D.C./D.S./Fine words or glyphs where the score places them. The scrolling
staff SHALL derive its bar lines from played slots, so a repeated written
measure scrolls past once per pass with its repeat barlines drawn each time,
and only the volta actually played on that pass is shown.

#### Scenario: Partition shows the repeat vocabulary

- **WHEN** a piece with a repeated section, two voltas and a `%` measure is
  opened in Partition mode
- **THEN** the repeat barlines, both volta brackets and the `%` sign are drawn
  at their written positions

#### Scenario: Scrolling staff shows the pass being played

- **WHEN** the scrolling staff crosses a repeated section on its second pass
- **THEN** the section's measures scroll past again and the volta-2 measures
  are shown in place of the volta-1 measures

## MODIFIED Requirements

### Requirement: Derived Playback Timing

The system SHALL derive a playback timing from a parsed score so the time-based
render modes (waterfall and scrolling staff) can present the selected piece and so
the audio synthesizer can play it. The derivation SHALL follow the **playback
order** (the unrolled sequence of written-measure passes), accumulating time per
played slot, so repeated sections sound as many times as the score prescribes
and a measure-repeat (`%`) slot replays the referenced measure's notes at its
played time. For each non-rest note it SHALL compute a MIDI
pitch from the note's step, octave and alteration, and a start time and duration
in milliseconds from the note's running division position and a tempo (taken from
a `metronome` direction when present, otherwise a default). Chord members SHALL
share the onset of the note they attach to; rests SHALL NOT produce a played note.
Tie chains SHALL merge only across slots adjacent in the played order; a tie
written across a jump that is not played contiguously falls back to a playable
note (the dangling-stop rule). Each derived note SHALL also carry its staff,
beam states and the clef in effect,
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

#### Scenario: Repeated section sounds twice

- **WHEN** a two-measure section sits between repeat barlines
- **THEN** the derived timeline contains that section's notes twice, the second
  pass shifted by the section's duration, and the song end reflects the
  unrolled length

#### Scenario: Measure-repeat slot replays the referenced content

- **WHEN** a `%` measure replays the previous written measure
- **THEN** the derived timeline sounds the referenced measure's notes during the
  `%` measure's played slot instead of silence

### Requirement: Per-Measure Playback Timing

The derivation SHALL compute, for each **played slot** of the playback order,
its playback start time in
milliseconds, using the same running division position and tempo as the note
timeline (a slot's start is its accumulated start-divisions times the
milliseconds-per-division). This lets any playback position be mapped to the
played slot it falls in — and, through the played-to-written mapping, to the
written measure being performed — and the fraction elapsed within it. For a
score without repeats the table is identical to the written measure table.

#### Scenario: Measure start times derived

- **WHEN** a score is derived for playback
- **THEN** each played slot has a start time in ms equal to its accumulated
  start-divisions × ms-per-division, and the first slot starts at 0

#### Scenario: Position maps to a measure and fraction

- **WHEN** the playhead is at a time within a played slot's span
- **THEN** that slot (and its written measure) is identified as current and the
  fraction elapsed within it is the position between its start and the next
  slot's start

#### Scenario: Repeated measure has one slot per pass

- **WHEN** a written measure is played twice because of a repeat
- **THEN** the timing table contains two slots for it, each with its own start
  time, both mapping to the same written measure
