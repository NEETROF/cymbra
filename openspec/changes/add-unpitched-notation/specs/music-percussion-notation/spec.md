## ADDED Requirements

### Requirement: Unpitched note events

The parser SHALL represent a percussion note — a `<note>` carrying `<unpitched>`
rather than `<pitch>` — as a note event whose written position (`display-step`,
`display-octave`) is held in a channel **distinct** from the pitched channel. An
unpitched note's written position denotes a staff placement, NOT a sounding pitch,
so it SHALL NOT be exposed as a `Pitch`: any consumer computing a MIDI number from
step/octave/alteration would otherwise derive a meaningless frequency from a
placement. A note event SHALL therefore be exactly one of: pitched, unpitched, or a
rest.

#### Scenario: Unpitched note is parsed with its written position

- **WHEN** a note declares `<unpitched>` with display-step E and display-octave 5
- **THEN** a note event is produced carrying that written position in the unpitched
  channel, with its duration, voice and staff, and with no pitch

#### Scenario: An unpitched note is not a pitched note

- **WHEN** a consumer inspects an unpitched note event's pitched channel
- **THEN** it finds no pitch, so no MIDI number is derived from the written
  position

#### Scenario: Pitched parsing is unaffected

- **WHEN** a score containing only `<pitch>` notes is parsed
- **THEN** every note event carries its pitch exactly as before, and none carries
  an unpitched position

### Requirement: Part-list instrument table

The parser SHALL read the `<part-list>` instrument declarations —
`<score-part>/<score-instrument>` and their `<midi-instrument>` children — and
expose, per instrument id, the General MIDI percussion number declared by
`<midi-unpitched>`. This table is the ONLY authoritative link between a written
percussion note and the sound it denotes, so an unpitched note's `<instrument id>`
SHALL resolve through it rather than through any inference from the note's written
position.

#### Scenario: Instrument id resolves to its General MIDI number

- **WHEN** the part list declares an instrument whose `midi-unpitched` is 38 and a
  note references that instrument id
- **THEN** that note event resolves to General MIDI percussion number 38

#### Scenario: Unresolvable instrument leaves the sound unknown

- **WHEN** an unpitched note references an instrument id absent from the part list,
  or the declaration carries no `midi-unpitched`
- **THEN** the note event is still produced with its written position and duration,
  and its General MIDI number is left unknown rather than guessed from the written
  position

#### Scenario: A score without a part-list instrument table still parses

- **WHEN** a score declares no `<score-instrument>` at all
- **THEN** parsing succeeds and the instrument table is empty

### Requirement: Percussion clef

The parser SHALL accept a clef whose sign is `percussion` and report it as such to
renderers, so a percussion staff is not mistaken for a treble or bass staff. The
clef sign SHALL be represented by a type able to carry a multi-character sign; a
sign the parser does not recognise SHALL leave the clef at the staff's default
rather than failing the parse.

#### Scenario: Percussion clef is reported

- **WHEN** a measure declares `<clef><sign>percussion</sign><line>2</line></clef>`
- **THEN** the score document reports a percussion clef on that staff

#### Scenario: Existing clef signs are unchanged

- **WHEN** a score declares a treble (G/2) or bass (F/4) clef
- **THEN** the score document reports it exactly as before

#### Scenario: Unknown clef sign degrades rather than fails

- **WHEN** a measure declares a clef sign the parser does not recognise
- **THEN** parsing succeeds and that staff keeps its default clef

### Requirement: Instrument classification of a score

The parser SHALL derive, from a parsed document, which instrument family the score
is written for — keyboard or percussion — as a single value for the whole score. A
score is classified as percussion when its notes are unpitched; classification
SHALL be derived from the parse alone and never from a filename, a part name typed
by a contributor, or any other external claim. A score whose classification cannot
be determined SHALL be reported as unknown rather than defaulted to either family.

#### Scenario: A drum score is classified as percussion

- **WHEN** a score whose notes are unpitched is parsed
- **THEN** its derived instrument classification is percussion

#### Scenario: A piano score is classified as keyboard

- **WHEN** a score whose notes are pitched is parsed
- **THEN** its derived instrument classification is keyboard

#### Scenario: Classification never trusts external metadata

- **WHEN** a score's file name or part name says one instrument while its notation
  says another
- **THEN** the classification follows the notation

### Requirement: The validation gate stays closed in this change

The shared validation gate SHALL continue to reject a score containing no **pitched**
notes, exactly as before this change, so a percussion score still cannot enter the
system through the upload or crawler paths. Parsing percussion notation and
admitting it are deliberately separated: opening the gate discloses drum scores to
every caller, which requires the access controls delivered by `add-drums-access`.
Until then this capability is reachable only by a direct call to the parser.

#### Scenario: A percussion score is still refused by the gate

- **WHEN** a score whose only notes are unpitched is validated
- **THEN** it is rejected for containing no playable notes, exactly as before

#### Scenario: The parser still produces the percussion document

- **WHEN** the same score is parsed directly rather than validated
- **THEN** the full percussion document is produced, with its unpitched notes,
  instrument table and classification

### Requirement: Unpitched notes reach the playback schedule

The playback schedule SHALL include unpitched notes, timed by the same rules as
pitched notes, carrying the General MIDI percussion number resolved from the
part-list instrument table instead of a pitch. An unpitched note whose General MIDI
number could not be resolved SHALL be omitted from the schedule rather than emitted
with a fabricated number. This rule SHALL hold identically in the shared crate's
schedule and in the app's mirror implementation, which are deliberate duplicates:
a divergence would make the back-office preview time a score differently from the
app.

#### Scenario: A percussion note is scheduled

- **WHEN** a score with unpitched notes is scheduled
- **THEN** each note appears at its computed start time carrying its General MIDI
  percussion number

#### Scenario: An unresolvable note is omitted, not fabricated

- **WHEN** an unpitched note's General MIDI number could not be resolved
- **THEN** it does not appear in the schedule, and the surrounding notes keep their
  computed times

#### Scenario: The crate and the app schedule identically

- **WHEN** the same percussion score is scheduled by the shared crate and by the
  app's mirror implementation
- **THEN** both produce the same notes with the same start times, durations and
  General MIDI numbers
