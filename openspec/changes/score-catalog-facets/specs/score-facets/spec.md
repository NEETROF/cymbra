## ADDED Requirements

### Requirement: Musical facets derived at ingest

The system SHALL derive a set of musical facets for every score from its single parsed
document (the shared summary derivation), without a second parse and without any
client-supplied claim. The facets SHALL include: the smallest note value present, whether the
score contains chords, tuplets, and dotted rhythms, the pitch ambitus (lowest and highest
sounding pitch), the number of staves, the count of playable notes, the tempo in beats-per-minute (when the
score carries a tempo/metronome marking), and whether dynamics markings are present. Each
facet SHALL be computed identically for the upload record and the crawler catalog row.

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
traits. The catalog facets SHALL be **populated by the crawler at ingest** (it already parses
each score), so the existing corpus is repopulated by re-crawling — no separate backfill pass.
A score whose facet cannot be determined SHALL store a null for that facet rather than a
fabricated value.

#### Scenario: Crawled rows carry facets

- **WHEN** the crawler ingests a score into the catalog
- **THEN** that row carries the facets derived from the parsed score

#### Scenario: Undeterminable facet stored as null

- **WHEN** a facet cannot be derived for a score
- **THEN** that facet is stored as null and the row is still persisted
