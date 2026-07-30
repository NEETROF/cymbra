## MODIFIED Requirements

### Requirement: Note Extraction

For each measure the parser SHALL extract its note events in document order. Each
note event SHALL carry: its pitch (step, octave, and alteration) or a rest flag;
its duration in divisions; its note-type when present (e.g. half, eighth); the
number of augmentation dots; its accidental when present; its voice; and its
staff. A note carrying `<chord/>` SHALL be flagged as a chord member of the
preceding note.

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

## ADDED Requirements

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
