## MODIFIED Requirements

### Requirement: Output Offset Applied To The Timing Reference

Scoring SHALL judge a free-run attack against the score position the player is actually
**hearing** — the playhead shifted back by the configured output offset — so a player
following delayed audio is not penalized for the transport's delay. The same shift SHALL
be applied to the visual playhead, so that what is highlighted and what a free-run attack
is judged against stay derived from a single value and cannot drift apart.

In **Wait Mode** the system SHALL judge against the **unshifted** playhead, because the
score clock there serves only to identify which onset the frozen playhead is gating,
matched to within a millisecond; nothing has sounded during a freeze, and the verdict
itself is a reaction time taken from the wall clock. A shifted clock in that mode matches
no onset, and a press that satisfies an open gate SHALL NOT be silently discarded.

The clock the scorer measures against SHALL NOT depend on the transport speed. It is
compared against positions stamped earlier in the same run — pending onsets, the bind
window, the start of a held note — so a value the player can change mid-run would step the
clock and resolve notes they still own. (The offset is a wall-clock latency applied to a
score clock, which is a known unit error at speeds other than 1x; correcting it SHALL be
done by making the heard position a *delayed* playhead, not by rescaling the offset by the
current speed.)

An offset of zero SHALL leave every timing judgment and every drawn position identical to
the behaviour without the setting, at every transport speed and in both modes.

#### Scenario: Zero offset changes nothing

- **WHEN** the output offset is zero
- **THEN** every timing verdict is identical to the verdict produced without the
  offset setting, at any transport speed and in either mode

#### Scenario: Delayed audio does not penalize the player

- **WHEN** a non-zero output offset is configured and the player attacks a pitch in
  time with the delayed sound they hear
- **THEN** the attack is judged against the shifted reference rather than being
  judged late

#### Scenario: Playhead and scoring shift together

- **WHEN** a non-zero output offset is configured
- **THEN** the visual playhead and the free-run scoring reference are both shifted by that
  same offset

#### Scenario: Wait Mode judges on the unshifted playhead

- **WHEN** a non-zero output offset is configured and Wait Mode is on
- **THEN** an awaited press is credited: the gate is identified against the unshifted
  playhead it is frozen at, and reaction time is measured from when that gate opened

#### Scenario: An awaited press is never silently dropped

- **WHEN** a press satisfies an open Wait Mode gate under any output offset
- **THEN** it is recorded as a judgment — it is never left absent from the run's results,
  which would let the note be reported missed at the end of a run the player played

#### Scenario: Changing the tempo mid-run resolves nothing

- **WHEN** a scored run is in progress under a non-zero output offset and the player
  changes the transport speed
- **THEN** no pending onset is resolved by the change itself, and no held note's sustain is
  truncated by it — the scoring clock does not move when the tempo does
