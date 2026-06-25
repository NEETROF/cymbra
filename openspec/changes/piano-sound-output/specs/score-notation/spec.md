## MODIFIED Requirements

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
