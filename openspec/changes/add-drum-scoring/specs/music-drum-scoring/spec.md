## ADDED Requirements

### Requirement: Stroke identity is matched at the kit piece's grain

The percussion matcher SHALL decide whether an incoming stroke satisfies a
written note by **kit piece**, never by raw General MIDI number equality: an
incoming number satisfies a written note exactly when both resolve to the same
piece of the named-piece table that `music-drum-kit-view` collapses lanes with —
hi-hat {42, 46}, snare {37, 38, 40}, ride {51, 53, 59}, kick {35, 36}, and each
tom, each accent cymbal and each terminal-bucket number a piece of its own. The
written note's number is the parser's resolution
(`NoteEvent.unpitched.gm_number`; MusicXML's `<midi-unpitched>` is one-based, so
the General MIDI number is the element value − 1).

The governing principle: **the matcher never demands a distinction the cascade
does not draw.** Numbers the display collapses into one lane with one look —
which zone of the snare, which flavour of kick, which ride surface — are
hardware-mapping artefacts the player can neither see nor control, so they
match; pieces with lanes of their own are separate aim points, so hitting the
wrong one is exactly the error scoring exists to catch.

Equivalence SHALL be keyed to the **static table**, not to the score-derived
lane layout: the layout intersects each group with the numbers present in the
score, so a number the score never uses has no lane — but it still denotes the
same physical piece and SHALL still match. A stroke of a piece that no open
onset requires SHALL be an extra/wrong note, exactly as an unexpected pitch is
for a keyboard score.

#### Scenario: An electric snare satisfies a written acoustic snare

- **WHEN** the score requires General MIDI 38 at an onset and the player's kit
  sends 40
- **THEN** the stroke binds to that onset and is judged on its timing

#### Scenario: A number absent from the score still matches its piece

- **WHEN** a score contains only 38-numbered snare notes and the player's kit
  sends 37 (side stick) at a snare onset
- **THEN** the stroke binds — equivalence reads the piece table, not the lanes
  derived from the score

#### Scenario: A china does not satisfy a written crash

- **WHEN** the score requires a crash (49) and the player strikes the china (52)
- **THEN** the stroke does not bind; the onset stays unsatisfied and the stroke
  is recorded as an extra note

#### Scenario: One tom does not satisfy another

- **WHEN** the score requires the high tom and the player strikes the floor tom
- **THEN** the stroke does not bind and is recorded as an extra note

#### Scenario: Both kick numbers drive the same pedal

- **WHEN** the score requires General MIDI 35 and the player's kit sends 36
- **THEN** the stroke binds to the kick onset

#### Scenario: A stroke nobody asked for is an extra note

- **WHEN** the player strikes a piece that no open onset requires within the
  timing windows
- **THEN** the stroke is recorded as an extra/wrong note against the run and
  credits no onset

### Requirement: The open and closed hi-hat shade the verdict and never gate

The matcher SHALL treat the closed (42) and open (46) hi-hat as one piece for
**binding and gate release**, and SHALL cap the timing verdict of a
wrong-articulation stroke **below `perfect`**: the stroke binds to the onset,
releases the Wait gate, and is judged at most `good` on its timing scale. A
correct-articulation stroke SHALL be judged by timing alone, with no bonus.

Open versus closed is the one distinction the cascade draws **inside** a lane
(the hollow variant), so erasing it entirely would void a drawn promise — but
producing it requires a hi-hat controller many electronic kits lack, and a
practice gate must never block on a stroke the player's hardware cannot emit.
Shading resolves both: the run always completes, the difference always costs.

#### Scenario: A closed stroke releases a gate waiting on an open one

- **WHEN** Wait Mode is blocked on an open hi-hat (46) and the player, on a kit
  with no hi-hat controller, strikes a closed hi-hat (42)
- **THEN** the gate releases and the stroke binds to the onset

#### Scenario: The wrong articulation is never perfect

- **WHEN** a stroke binds to a hi-hat onset with the wrong articulation, at a
  timing that would otherwise be `perfect`
- **THEN** its verdict is capped below `perfect`

#### Scenario: The right articulation is judged on timing alone

- **WHEN** a stroke binds to a hi-hat onset with the written articulation
- **THEN** its verdict comes from the ordinary timing scale, uncapped

### Requirement: Percussion reuses the keyboard timing windows and ignores velocity

The percussion scorer SHALL judge attacks with the **same timing windows and
verdict scale** as keyboard scoring — the signed offset against the scheduled
onset in free run, the reaction time from the gate opening in Wait Mode — and
SHALL ignore stroke velocity, as keyboard scoring already ignores key velocity.
The windows remain tunable constants, not spec-locked values; any
drum-specific retuning is a constants change informed by on-device play, never
a silent divergence per call site.

#### Scenario: The same windows judge both instruments

- **WHEN** a keyboard note and a percussion stroke are each attacked at the same
  signed offset from their scheduled onsets in free run
- **THEN** both receive the same timing verdict

#### Scenario: Velocity does not change a verdict

- **WHEN** two otherwise-identical strokes arrive with very different velocities
- **THEN** their verdicts are identical

### Requirement: A percussion run has no sustain dimension

The percussion synchronization percentage SHALL blend **timing and correctness
only**, renormalizing the two weights so their relative ratio matches the
keyboard blend's, and the sustain dimension SHALL be **absent** for a percussion
run — absent like a mode sub-score with no onsets, never zero and never a
constant full credit. The keyboard blend SHALL be unchanged. A stroke's release
SHALL never be judged: a drum note's duration is an artefact of the hardware,
not of the player.

#### Scenario: An instant release costs nothing

- **WHEN** a percussion stroke binds correctly and its note-off follows
  immediately
- **THEN** no sustain penalty exists anywhere in the run's result

#### Scenario: The percussion blend has two dimensions

- **WHEN** a percussion run's synchronization percentage is computed
- **THEN** it is a weighted blend of timing and correctness only, with the
  weights renormalized from the keyboard ratio

#### Scenario: A do-nothing percussion run scores zero, not the sustain weight

- **WHEN** a percussion run ends with every onset missed and no stroke played
- **THEN** its synchronization percentage is 0 — no absent dimension leaks
  credit into the blend

#### Scenario: Keyboard runs are unaffected

- **WHEN** a keyboard run is scored
- **THEN** the three-dimension blend and its weights are exactly as before

### Requirement: One stroke identity serves every consumer

The system SHALL route every stroke-identity decision — the Wait Mode gate's
required set, the scorer's binding decision, the extra-stroke detection and the
pad feedback — through **one shared stroke-identity function**, sourced from the
kit-piece table — never a second, locally re-derived equivalence. A stroke that releases the gate SHALL be a
stroke the scorer binds, and a stroke the scorer rejects SHALL never release
the gate.

#### Scenario: The gate and the scorer agree

- **WHEN** any stroke arrives while Wait Mode is blocked on an onset
- **THEN** the gate releases exactly when the scorer binds that stroke to the
  onset, and stays blocked exactly when the scorer would record it as an extra
  note

#### Scenario: Pad feedback agrees with the judgment

- **WHEN** a stroke is judged
- **THEN** the pad feedback reflects the same piece resolution the scorer used
