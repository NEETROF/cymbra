## MODIFIED Requirements

### Requirement: Musical facets derived at ingest

The system SHALL derive a set of musical facets for every score from its single parsed
document (the shared summary derivation), without a second parse and without any
client-supplied claim. The facets SHALL include: the smallest note value present, whether the
score contains chords, tuplets, and dotted rhythms, the pitch ambitus (lowest and highest
sounding pitch), the number of staves, the count of playable notes, the tempo in beats-per-minute (when the
score carries a tempo/metronome marking), whether dynamics markings are present, and the
score's **instrument classification** (keyboard or percussion, per the
`music-percussion-notation` capability). Each
facet SHALL be computed identically for the upload record and the crawler catalog row.

The pitch ambitus SHALL be left unknown for a percussion score, whose notes carry a written
staff position rather than a sounding pitch. (The existing staff-count facet `is_piano` is
replaced by this instrument classification in `add-drums-access`, which owns the column
migration its consumers read; until then it keeps its current meaning.)

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

#### Scenario: Instrument classification is recorded

- **WHEN** a score whose notes are unpitched is ingested
- **THEN** its instrument facet records percussion, and a score of pitched notes records
  keyboard

#### Scenario: A percussion score has no ambitus

- **WHEN** a percussion score is ingested
- **THEN** its lowest and highest pitch facets are left unknown rather than derived from
  written staff positions
