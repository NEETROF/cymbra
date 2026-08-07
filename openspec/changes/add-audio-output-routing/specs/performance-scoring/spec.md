## ADDED Requirements

### Requirement: Output Offset Applied To The Timing Reference

Scoring SHALL shift the time reference it judges attacks against by the
configured output offset, so a player following delayed audio is not penalized
for the transport's delay. The same offset SHALL shift the visual playhead, so
that what is highlighted and what is judged stay derived from a single value and
cannot drift apart. An offset of zero SHALL leave every timing judgment identical
to the behaviour without the setting.

#### Scenario: Zero offset changes nothing

- **WHEN** the output offset is zero
- **THEN** every timing verdict is identical to the verdict produced without the
  offset setting

#### Scenario: Delayed audio does not penalize the player

- **WHEN** a non-zero output offset is configured and the player attacks a pitch in
  time with the delayed sound they hear
- **THEN** the attack is judged against the shifted reference rather than being
  judged late

#### Scenario: Playhead and scoring shift together

- **WHEN** a non-zero output offset is configured
- **THEN** the visual playhead and the scoring reference are both shifted by that
  same offset

#### Scenario: Wait Mode inherits the shift

- **WHEN** a non-zero output offset is configured and Wait Mode is on
- **THEN** the gate opens against the same shifted reference, and reaction time is
  measured from that gate
