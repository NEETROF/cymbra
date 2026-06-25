## ADDED Requirements

### Requirement: Onset-Gated Playback

Wait Mode SHALL freeze time-based playback at each note onset and SHALL release
the freeze as soon as the required key(s) for that onset have been pressed. The
gate operates on note onsets (notes whose start coincides with the current gate
position), not on the sustained note window: once the required keys are pressed
the playhead advances to the next onset even if those keys are immediately
released. When Wait Mode is off, playback SHALL advance without gating.

#### Scenario: Press releases the gate
- **WHEN** Wait Mode is on, the playhead reaches a note's onset, and the player
  presses that note's key
- **THEN** the gate releases and the playhead advances to the next onset

#### Scenario: Release after press does not re-block
- **WHEN** the player has pressed the required key for the current onset and then
  lifts it before the next tick
- **THEN** the gate stays released and playback continues (sustain is not required)

#### Scenario: Wrong or missing key keeps blocking
- **WHEN** the playhead is at an onset and the required key has not been pressed
- **THEN** the playhead stays frozen at that onset

#### Scenario: Wait Mode off does not gate
- **WHEN** Wait Mode is off
- **THEN** the playhead advances through onsets without waiting for key presses

### Requirement: Validation At The Right Moment

A key press SHALL count toward the current onset gate only while the playhead has
reached that onset; a press that occurs before the playhead arrives at the onset
SHALL NOT pre-satisfy it. This keeps Wait Mode a timing exercise — the player must
press the note when the cascade reaches it, not arbitrarily early.

#### Scenario: Early press does not pre-satisfy
- **WHEN** the player presses the upcoming note's key while the playhead is still
  before that note's onset
- **THEN** the gate is not satisfied; when the playhead reaches the onset it still
  waits for a press at that moment

#### Scenario: Press at the onset satisfies
- **WHEN** the playhead has reached the onset and the player presses the required
  key
- **THEN** the press counts and the gate releases

### Requirement: Chord Onset Requires All Pitches

When multiple notes share an onset, the gate SHALL require every one of those
pitches to be pressed before releasing, accepting the presses as they accumulate
while the gate is active (the pitches need not be pressed simultaneously). The
gate SHALL NOT release until the full set for that onset has been pressed.

#### Scenario: All chord notes pressed
- **WHEN** an onset has three pitches and the player presses all three (in any
  order) while the gate is active
- **THEN** the gate releases after the third press

#### Scenario: Partial chord keeps blocking
- **WHEN** an onset has three pitches and the player has pressed only two of them
- **THEN** the gate stays frozen until the remaining pitch is pressed

### Requirement: No Sustain Or Synchronization Scoring

Wait Mode SHALL NOT require a note to be held for its notated duration and SHALL
NOT penalize early release or imprecise timing in this capability. Measuring
synchronization quality (how close the press is to the beat, sustain accuracy) is
explicitly out of scope and reserved for a future scoring/gamification capability.

#### Scenario: Holding past the press is not required
- **WHEN** the player presses the required key exactly at the onset and releases
  immediately
- **THEN** the onset is considered satisfied with no timing penalty applied
