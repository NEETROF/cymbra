# measure-range-practice — delta for add-repeat-unrolling

## ADDED Requirements

### Requirement: Selective runs are linear over written measures

The practice measure range SHALL keep operating on **written** measures on a
piece that carries repeats: the picker and Partition taps select written
measures, and a selective run SHALL play the selected written measures
linearly, exactly once per loop iteration, without unrolling any repeat,
volta selection or jump inside the range. A `%` measure inside the range
still replays its referenced measure's content (it has content of its own to
practice). This keeps the loop semantics unambiguous — the player chose bars
to drill, not a performance route — and matches the pre-change behavior for
scores without repeats.

#### Scenario: Range inside a repeated section plays once per loop

- **WHEN** the player selects measures 3–4 of a section the score repeats
  twice and starts a practice loop
- **THEN** each loop iteration plays measures 3–4 exactly once, then loops

#### Scenario: Range spanning both voltas plays them linearly

- **WHEN** the selected range covers the volta 1 and volta 2 measures
- **THEN** the run plays both endings in written order once per iteration
  (no pass-based selection)

#### Scenario: Measure-repeat inside the range is audible

- **WHEN** the selected range contains a `%` measure
- **THEN** that measure sounds its referenced content during the run
