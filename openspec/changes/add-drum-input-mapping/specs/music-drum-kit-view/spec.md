## MODIFIED Requirements

### Requirement: Wait Mode is not offered for a percussion score, for now

Until `add-drum-scoring` lands, a percussion score SHALL be playable in the
timed modes only, and Wait Mode SHALL NOT be offered for it. The restriction
SHALL apply to percussion scores only.

The recorded reason changes with `add-drum-input-mapping` and is restated
precisely, because the original one — no percussion input path exists, so the
gate would block forever — is no longer true: strokes now reach the player,
and a blocked gate could technically be satisfied. What remains missing is
honest judgment. The gate's exact-pitch test is wrong for percussion: one
lane deliberately collapses several General MIDI numbers (acoustic 38 /
electric 40 snare, closed 42 / open 46 hi-hat), the on-screen pad emits one
canonical member of that set, and a stroke a drummer correctly aims would
still be refused whenever the file's number differs from the struck one — the
gate would hold on spelling, not on playing. Deciding which numbers satisfy
which onsets is the matcher's equivalence table, owned by `add-drum-scoring`;
Wait Mode for percussion arrives with it.

#### Scenario: Wait Mode absent for a percussion score

- **WHEN** a percussion score is loaded
- **THEN** Wait Mode is not offered, and playback runs in the timed modes only

#### Scenario: Keyboard scores keep Wait Mode

- **WHEN** a keyboard score is loaded
- **THEN** Wait Mode is offered exactly as before
