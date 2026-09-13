## ADDED Requirements

### Requirement: Measured Input Offset Applied To Audio-Sourced Attacks

Scoring SHALL shift each attack timestamp of an audio-sourced run earlier by
the active route's measured input offset before judgment, so the player is
not penalized for the capture and detection chain's latency; the shift SHALL
come only from the calibration measurement — never a hand-tuned guess — and
MIDI-sourced runs SHALL be entirely unaffected by it.

#### Scenario: Detection latency does not read as late

- **WHEN** a player attacks a pitch exactly on its onset and the capture chain
  delivers the detected event a measured offset later
- **THEN** the attack is judged at its shifted (true) time and is not marked
  late by the chain's latency

#### Scenario: MIDI runs are untouched

- **WHEN** a run's input source is MIDI
- **THEN** no input offset is applied and every judgment is identical to the
  behavior before this capability existed

## MODIFIED Requirements

### Requirement: Sustain Judgment

For each correctly-attacked note, scoring SHALL evaluate how long the pitch was held
relative to the note's intended duration and SHALL derive a sustain ratio bounded to a
sensible range (a note released far too early scores low; a note held for roughly its
intended duration scores high). Releasing after the intended duration SHALL NOT be
penalized beyond the maximum.

For **audio-sourced runs**, where the damper pedal masks releases and note-off
events are best-effort at most, scoring SHALL NOT judge sustain: the sustain
dimension is excluded and its weight redistributed across the remaining
dimensions (the attack-only precedent set by percussion scoring), so an
audio-sourced score is never depressed by undetectable releases.

#### Scenario: Held for its full duration scores high sustain
- **WHEN** a correctly-attacked note is held for approximately its intended duration
- **THEN** its sustain dimension for that note is at or near the maximum

#### Scenario: Released far too early scores low sustain
- **WHEN** a correctly-attacked note is released well before a minimum fraction of its
  intended duration
- **THEN** its sustain dimension for that note is low

#### Scenario: Over-holding is not penalized
- **WHEN** a correctly-attacked note is held past its intended duration
- **THEN** its sustain dimension is not reduced below the value for an exactly-held note

#### Scenario: Audio-sourced run ignores releases

- **WHEN** a run's input source is the microphone and a note's release is never
  observed
- **THEN** the run's score carries no sustain dimension and is not reduced by
  the missing release
