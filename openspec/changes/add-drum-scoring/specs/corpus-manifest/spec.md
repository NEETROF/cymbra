## MODIFIED Requirements

### Requirement: Difficulty is provenance-tracked

The system SHALL record difficulty as a nullable `level` (Beginner /
Intermediate / Advanced) together with a `level_source` (`source` | `heuristic` |
`manual` | null). A source-declared grade SHALL set `level_source = source`;
otherwise a heuristic estimate computed from the parsed score MAY set
`level_source = heuristic`; `level` SHALL be null when neither applies. A
heuristic estimate SHALL NEVER be recorded as a source grade.

The heuristic SHALL be **instrument-aware**. For a **percussion** score it SHALL
estimate from drum-shaped features — stroke density (unpitched notes per
measure), tempo, the fastest subdivision, limb simultaneity (simultaneous
distinct pieces at one onset; two-voice writing), and kit breadth (the count of
distinct pieces) — and SHALL NOT apply the keyboard features (ambitus, melodic
leaps, key accidentals, grand staff), which count only pitched notes and so
degenerate to zero on a drum part, grading every groove Beginner regardless of
content. Both estimates SHALL emit the same three-level vocabulary, so the
difficulty weights of the play rewards and the global boards read one scale for
both instruments.

Percussion rows previously graded by the keyboard-shaped heuristic SHALL be
**re-graded** with the instrument-aware estimate. Only rows with
`level_source = heuristic` are touched — overwriting a heuristic value without
clobbering `source` or `manual` grades is exactly the overwrite the provenance
rule exists to permit.

#### Scenario: Source grade recorded as authoritative

- **WHEN** the source provides an explicit difficulty grade
- **THEN** `level` is set from it and `level_source = source`

#### Scenario: Heuristic estimate flagged as such

- **WHEN** no source grade exists and the heuristic runs
- **THEN** `level` holds the estimate and `level_source = heuristic`, so a later
  curation pass can distinguish and overwrite it without clobbering real grades

#### Scenario: A dense groove is not Beginner by degeneracy

- **WHEN** the heuristic grades a fast, dense, multi-limb percussion score
- **THEN** the estimate comes from the drum-shaped features and is not Beginner
  merely because the score contains no pitched notes

#### Scenario: The re-grade touches only heuristic rows

- **WHEN** the re-grade pass processes existing percussion rows
- **THEN** rows with `level_source = heuristic` receive the instrument-aware
  estimate, and rows graded `source` or `manual` are unchanged
