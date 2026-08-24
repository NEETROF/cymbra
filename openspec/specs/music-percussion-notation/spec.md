# music-percussion-notation Specification

## Purpose
TBD - created by archiving change add-unpitched-notation. Update Purpose after archive.
## Requirements
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
expose, per instrument id, the General MIDI percussion number derived from
`<midi-unpitched>`. The element is **one-based**: MusicXML numbers MIDI notes
1–128 where MIDI itself counts 0–127, so the General MIDI number is the element
value **minus one**. A conformant exporter (MuseScore among them) writes `39` for
the GM-38 acoustic snare and `43` for the GM-42 closed hi-hat; taking the element
value verbatim would shift every kit piece one instrument off (a real file's
hi-hat would read as a high floor tom) while self-consistent fixtures kept the
tests green. This table is the ONLY authoritative link between a written
percussion note and the sound it denotes, so an unpitched note's `<instrument id>`
SHALL resolve through it rather than through any inference from the note's written
position.

An unpitched note carrying no `<instrument>` element SHALL resolve to the part's
**sole** declared instrument when the part list declares exactly one
`score-instrument` with a `midi-unpitched` — the MusicXML default-instrument
convention, routine in Finale/Sibelius/Dorico exports of single-line percussion (a
tambourine part, a one-instrument drum line), where notes never reference the one
instrument they can only mean. When several instruments are declared and a note
references none, the ambiguity SHALL NOT be guessed away: the note's General MIDI
number stays unknown.

#### Scenario: Instrument id resolves to its General MIDI number

- **WHEN** the part list declares an instrument whose `midi-unpitched` is 39 and a
  note references that instrument id
- **THEN** that note event resolves to General MIDI percussion number 38 — the
  one-based element value minus one

#### Scenario: A note without an instrument reference falls back to the sole instrument

- **WHEN** a part list declares exactly one `score-instrument` whose
  `midi-unpitched` is 39, and an unpitched note carries no `<instrument>` element
- **THEN** that note event resolves to General MIDI percussion number 38, the
  part's default instrument

#### Scenario: Several instruments and no reference stays unknown

- **WHEN** a part list declares several instruments and an unpitched note carries
  no `<instrument>` element
- **THEN** that note event's General MIDI number is left unknown rather than picked
  arbitrarily among the declarations

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
is written for — keyboard, percussion, or unknown — as a single value for the whole
score. A score SHALL classify as **percussion** when every non-rest note of the
parsed part is unpitched, as **keyboard** when every non-rest note is pitched, and
as **unknown** otherwise — mixed pitched-and-unpitched content, or a score with no
notes at all. The all-or-nothing rule is deliberate: the product premise is that a
score is piano XOR drums, and a file violating that premise is a fact to report,
not to round off — `add-drums-access` keys server-side access control and catalog
filtering on this value, so a guessed family would become a guessed disclosure
decision. Classification reads the **first part only**, matching the parser's
existing single-part document model: whichever part a multi-part file puts first
is the part being classified. Classification SHALL be derived from the parse alone
and never from a filename, a part name typed by a contributor, or any other
external claim.

#### Scenario: A drum score is classified as percussion

- **WHEN** a score whose non-rest notes are all unpitched is parsed
- **THEN** its derived instrument classification is percussion

#### Scenario: A piano score is classified as keyboard

- **WHEN** a score whose non-rest notes are all pitched is parsed
- **THEN** its derived instrument classification is keyboard

#### Scenario: A mixed score is classified as unknown

- **WHEN** the parsed part contains both pitched and unpitched non-rest notes
- **THEN** its derived instrument classification is unknown — neither family is
  guessed for a score that violates the piano-XOR-drums premise

#### Scenario: A score with no notes is classified as unknown

- **WHEN** the parsed part contains only rests, or no notes at all
- **THEN** its derived instrument classification is unknown

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

The playback schedule SHALL include unpitched notes when, and only when, the score
classifies as **percussion**, timed by the same rules as pitched notes and carrying
the General MIDI percussion number resolved from the part-list instrument table
instead of a pitch. The condition is what keeps this change inert: the validation
gate counts only pitched notes, so a score mixing pitched and unpitched notes is
admissible **today** and can already sit in the corpus — its unpitched notes
currently parse to nothing and play as silence. Emitting them unconditionally
would change that score's playback (General MIDI numbers rendered through the
piano synth, Wait Mode gating on notes that were silent yesterday) with no access
control in front of it. Gated on the classification — which a mixed score fails,
reading as unknown — emission is reachable only by scores the still-closed gate
refuses, so every score admissible today plays exactly as before.

An unpitched note whose General MIDI number could not be resolved SHALL be omitted
from the schedule rather than emitted with a fabricated number. Tied unpitched
notes SHALL merge into ONE prolonged note keyed by (voice, resolved General MIDI
number) — a drum tie means "let ring", not "strike again" — and a tie chain whose
links' General MIDI number is unresolved SHALL be omitted entirely: half a merged
chain is a fabricated rhythm.

This rule SHALL hold identically in the shared crate's schedule and in the app's
mirror implementation, which are deliberate duplicates: a divergence would make
the back-office preview time a score differently from the app. The crate schedule
performs no tie merging for **pitched** notes today while the app mirror does;
that pre-existing divergence is out of scope here and does not undermine parity,
which quantifies over percussion-classified scores — where both sides apply this
same unpitched tie rule.

#### Scenario: A percussion note is scheduled

- **WHEN** a score classified as percussion is scheduled
- **THEN** each unpitched note appears at its computed start time carrying its
  General MIDI percussion number

#### Scenario: A mixed score keeps today's behaviour

- **WHEN** a score containing both pitched and unpitched notes — admissible
  through today's validation gate — is scheduled
- **THEN** its pitched notes are scheduled exactly as before and its unpitched
  notes are skipped, exactly as today

#### Scenario: An unresolvable note is omitted, not fabricated

- **WHEN** an unpitched note's General MIDI number could not be resolved
- **THEN** it does not appear in the schedule, and the surrounding notes keep their
  computed times

#### Scenario: Tied unpitched notes merge into one prolonged note

- **WHEN** a percussion score ties a cymbal note across a barline
- **THEN** the schedule contains one note whose duration spans the whole chain,
  keyed by its voice and resolved General MIDI number — in the crate schedule and
  the app mirror alike

#### Scenario: The crate and the app schedule identically

- **WHEN** the same percussion-classified score — tied cymbals included — is
  scheduled by the shared crate and by the app's mirror implementation
- **THEN** both produce the same notes with the same start times, durations and
  General MIDI numbers

