## MODIFIED Requirements

### Requirement: Validation At The Right Moment

A pitch SHALL count toward the current onset gate when it is down at the moment
the playhead has reached that onset — whether the player presses it while the
gate is active OR was already holding it continuously when the gate became
active. A press that occurred **and was released** before the playhead reached
the onset SHALL NOT pre-satisfy it.

Each press SHALL count toward at most one onset: once a held pitch has satisfied
an onset, that same hold SHALL NOT satisfy a later onset of the same pitch — a
repeated pitch requires a fresh attack (release and re-press, or a new press).
This keeps Wait Mode a timing exercise while tolerating a sustained/tied note
carried into the onset where it first sounds; it does not let a single held key
auto-advance through repeated notes.

#### Scenario: Early press-and-release does not pre-satisfy
- **WHEN** the player presses the upcoming note's key and releases it before the
  playhead reaches that note's onset
- **THEN** the gate is not satisfied; when the playhead reaches the onset it still
  waits for the pitch to be down at that moment

#### Scenario: Pitch held through the onset satisfies
- **WHEN** the player presses a note's key before its onset, that pitch has not
  already satisfied an earlier onset, and it is still held when the playhead
  reaches the onset
- **THEN** the held pitch counts as satisfied at the onset with no re-press
  required, and the gate releases (subject to any other pitches the onset needs)

#### Scenario: Repeated pitch held across onsets requires a fresh attack
- **WHEN** the same pitch is required at two consecutive onsets, the player's hold
  satisfied the first onset, and the player keeps holding it into the second onset
  without re-pressing
- **THEN** the second onset is NOT satisfied by the sustained hold; it stays frozen
  until the player releases and re-presses (or otherwise re-attacks) that pitch

#### Scenario: Press at the onset satisfies
- **WHEN** the playhead has reached the onset and the player presses the required
  key
- **THEN** the press counts and the gate releases

### Requirement: Chord Onset Requires All Pitches

When multiple notes share an onset, the gate SHALL require every one of those
pitches to be down before releasing. Each pitch MAY be satisfied either by being
already held continuously when the onset becomes active or by being pressed while
the gate is active; the pitches need not be pressed simultaneously. The gate
SHALL NOT release until the full set for that onset is satisfied.

#### Scenario: All chord notes pressed
- **WHEN** an onset has three pitches and the player presses all three (in any
  order) while the gate is active
- **THEN** the gate releases after the third press

#### Scenario: Mix of held and freshly pressed pitches
- **WHEN** an onset has three pitches, the player is already holding one of them
  from before the onset, and then presses the other two while the gate is active
- **THEN** the held pitch and the two presses together satisfy the onset and the
  gate releases

#### Scenario: Partial chord keeps blocking
- **WHEN** an onset has three pitches and only two of them are down (held or
  pressed)
- **THEN** the gate stays frozen until the remaining pitch is down
