# score-facets Specification

## Purpose
TBD - created by archiving change score-catalog-facets. Update Purpose after archive.
## Requirements
### Requirement: Musical facets derived at ingest

The system SHALL derive a set of musical facets for every score from its single parsed
document (the shared summary derivation), without a second parse and without any
client-supplied claim. The facets SHALL include: the smallest note value present, whether the
score contains chords, tuplets, and dotted rhythms, the pitch ambitus (lowest and highest
sounding pitch), the number of staves, the count of playable notes, the tempo in beats-per-minute (when the
score carries a tempo/metronome marking), whether dynamics markings are present, and the
score's **instrument** (`keyboard`, `percussion`, or `unknown`). Each
facet SHALL be computed identically for the upload record and the crawler catalog row.

The instrument facet **replaces** the former grand-staff `is_piano` facet. That facet was a
proxy rather than a detection — it reported whether the part declared two or more staves, so a
single-staff keyboard piece read as not-piano while an organ, a two-staff arrangement or a drum
kit notated on two staves read as piano. The instrument is now derived from the notation
itself, so the proxy is retired rather than kept alongside.

The count of playable notes SHALL include **unpitched** notes, so a percussion score reports a
non-zero count. The pitch ambitus SHALL be left unknown for a percussion score, whose notes
carry a written staff position rather than a sounding pitch.

(Deferred to a follow-up because they require extending the app-bridged parse model: the mode
major/minor facet and the ornaments/articulations/pedal expressivity flags. The key signature
`key_fifths` remains available in the meantime.)

#### Scenario: Smallest note value is captured

- **WHEN** a score is ingested whose fastest notated value is a sixteenth
- **THEN** its smallest-note-value facet records the sixteenth granularity

#### Scenario: Rhythmic and textural flags are captured

- **WHEN** a score contains chords, tuplets, or dotted rhythms
- **THEN** the corresponding facet flag is set for each trait present

#### Scenario: Ambitus is captured from pitches

- **WHEN** a score is ingested
- **THEN** its lowest and highest sounding pitches are recorded so a hand-span (ambitus)
  range can be computed

#### Scenario: Facets are derived, never supplied

- **WHEN** a score is ingested
- **THEN** every facet is computed from the parsed document, not from any external metadata claim

#### Scenario: An absent signal is left unknown, not false-positive

- **WHEN** a score has no sounding notes (e.g. a rest-only measure)
- **THEN** the smallest-note-value and ambitus facets are left unknown rather than guessed

#### Scenario: Tempo captured from a marking

- **WHEN** a score carries a tempo/metronome marking
- **THEN** its tempo facet records the beats-per-minute of that marking

#### Scenario: Missing tempo is left unknown, never fabricated

- **WHEN** a score carries no tempo/metronome marking
- **THEN** its tempo facet is left unknown (null) rather than set to a playback default

#### Scenario: Instrument is recorded

- **WHEN** a score whose notes are unpitched is ingested
- **THEN** its instrument facet records percussion, and a score of pitched notes records
  keyboard

#### Scenario: A two-staff drum score is not recorded as keyboard

- **WHEN** a percussion score notated on two staves is ingested
- **THEN** its instrument facet records percussion, not keyboard

#### Scenario: Playable notes include unpitched notes

- **WHEN** a percussion score is ingested
- **THEN** its playable-note count is non-zero

#### Scenario: A percussion score has no ambitus

- **WHEN** a percussion score is ingested
- **THEN** its lowest and highest pitch facets are left unknown rather than derived from
  written staff positions

### Requirement: Smallest note value falls back to duration when untyped

When a note carries no notated type, the smallest-note-value derivation SHALL fall back to the
note's duration relative to the score's divisions to classify its value, so a score exported
without `<type>` elements still receives a smallest-note-value facet. Rests and zero-duration
(grace) notes SHALL be ignored.

#### Scenario: Untyped notes still yield a value

- **WHEN** a score's notes omit the notated type but carry durations
- **THEN** the smallest-note-value facet is derived from the durations

#### Scenario: Rests do not affect the value

- **WHEN** a score interleaves rests with notes
- **THEN** only sounding notes contribute to the smallest-note-value facet

### Requirement: Facets persisted on catalog and user scores

The derived facets SHALL be persisted as nullable fields on the public catalog scores (and the
same columns SHALL exist on user-uploaded scores for parity), so search can filter by these
traits. The general catalog facets SHALL be **populated by the crawler at ingest** (it already
parses each score), so for them the existing corpus is repopulated by re-crawling — no separate
backfill pass. The **instrument** facet is the one exception: it is the predicate the drum
audience enforcement reads, so it gets a one-time application-level re-derivation pass over the
stored bytes (below) rather than waiting on a re-crawl that user uploads would never receive.
A score whose facet cannot be determined SHALL store a null for that facet rather than a
fabricated value — **except the instrument**, which SHALL store the literal `unknown` and never
NULL, so the enforcement gate and the instrument filter each test exactly one undeterminable
value instead of two spellings of it (the column is `NOT NULL DEFAULT 'unknown'` with a CHECK
on the three values).

The stored **instrument** column replaces the stored boolean grand-staff column on both tables.
The grand-staff flag cannot be translated into an instrument: `true` only ever meant "two or
more staves" and `false` "fewer than two", which includes single-staff keyboard pieces *and*
percussion parts alike. Existing rows SHALL therefore be **re-derived from their stored bytes**
rather than mapped from the flag. Because the bytes live in the object store, not in the row,
this re-derivation is an application-level pass (see `music-drums-visibility` and the change's
tasks), not a SQL backfill. A row whose bytes cannot be read or parsed SHALL be recorded
as `unknown` rather than assigned a family.

This matters beyond tidiness: the corpus already contains percussion scores, ingested despite
the playable-notes gate. A migration that mapped the flag would record every one of them as
`unknown`, and any consumer treating `unknown` as "certainly not percussion" would then be
wrong about real rows.

#### Scenario: Instrument is re-derived, not translated

- **WHEN** the migration processes an existing row
- **THEN** its instrument comes from parsing its stored bytes, not from the former grand-staff
  flag

#### Scenario: An existing percussion row is found

- **WHEN** the migration processes a row whose stored score is a drum part
- **THEN** its instrument is recorded as `percussion`

#### Scenario: An unreadable row stays unknown

- **WHEN** the migration cannot read or parse a row's stored bytes
- **THEN** its instrument is recorded as `unknown` rather than guessed

#### Scenario: Crawled rows carry facets

- **WHEN** the crawler ingests a score into the catalog
- **THEN** that row carries the facets derived from the parsed score

#### Scenario: Undeterminable facet stored as null

- **WHEN** a facet other than the instrument cannot be derived for a score
- **THEN** that facet is stored as null and the row is still persisted

#### Scenario: The instrument is never NULL

- **WHEN** any row is written or backfilled and the instrument cannot be determined
- **THEN** the instrument column holds the literal `unknown`, never NULL

