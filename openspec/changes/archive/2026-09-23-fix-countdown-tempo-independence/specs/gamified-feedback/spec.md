## MODIFIED Requirements

### Requirement: Pre-Start Countdown

The system SHALL show a race-game style get-ready countdown (3 → 2 → 1 → GO) centred
over the player before the playhead advances when the player starts **free-run** playback
(Wait Mode off) from the beginning of a piece, so the player has time to ready their hands
before the notes start moving. The countdown SHALL last the same **wall-clock** duration
regardless of the transport speed — it is the player's reaction time, not musical time, so
it SHALL NOT be scaled by the tempo setting in either direction. The playhead SHALL stay
frozen at the start while the countdown runs, and playback SHALL begin (the first note
arriving no earlier than GO clearing) when the countdown reaches zero. Each step SHALL
animate in (fade and scale) and the GO step SHALL disappear before the first note. Presses
made during the countdown SHALL be treated as warm-ups and SHALL NOT be scored. Resuming
playback from a mid-piece pause SHALL NOT show the countdown. In Wait Mode the cascade
already freezes at the first onset (unlimited ready time), so no countdown SHALL be shown.

#### Scenario: Countdown precedes a free-run fresh start

- **WHEN** the player starts free-run playback from the beginning
- **THEN** a 3…2…1…GO countdown is shown, the playhead stays at the start until it ends, and
  playback begins when it reaches zero

#### Scenario: Countdown duration is independent of the transport speed

- **WHEN** the player starts free-run playback from the beginning at a transport speed
  other than 1x — at either the slowest or the fastest setting
- **THEN** the countdown takes the same wall-clock time to reach zero as it does at 1x,
  and this holds for each lap of a measure-range practice loop that re-arms it

#### Scenario: Wait Mode skips the countdown

- **WHEN** the player starts playback from the beginning with Wait Mode on
- **THEN** no countdown is shown (the cascade waits at the first onset instead)

#### Scenario: Warm-up presses during the countdown are not scored

- **WHEN** the player presses a key while the countdown is running
- **THEN** the press is not recorded as a judgment by the scorer

#### Scenario: Resuming mid-piece skips the countdown

- **WHEN** the player resumes playback after pausing mid-piece
- **THEN** no countdown is shown and playback continues immediately
