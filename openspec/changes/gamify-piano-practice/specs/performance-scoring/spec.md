## ADDED Requirements

### Requirement: Scored Run Activation

Performance scoring SHALL be active in a "playing" render mode (Synthesia or the
horizontal scrolling staff) with Wait Mode **either on or off**. Scoring SHALL be
inactive only in the engraved Partition view, which has no live play surface. A scored
run SHALL begin when playback starts from the beginning of the piece and SHALL end when
the playhead reaches the end of the piece.

A scored run SHALL judge only the notes of the currently selected hand(s) and SHALL record
which hand(s) were played, so the result reflects a single hand selection. Changing the
selected hand(s) SHALL restart the piece from the beginning with a fresh scored run for the
new selection, so the score stays coherent over the whole piece and still ends in a summary
(rather than leaving the piece playing unscored).

#### Scenario: Scoring runs in Synthesia with Wait Mode off
- **WHEN** Wait Mode is off, the render mode is Synthesia or scrolling staff, and the
  player starts playback from the beginning
- **THEN** a scored run begins and note judgments accumulate as the playhead advances

#### Scenario: Scoring runs with Wait Mode on
- **WHEN** Wait Mode is on, the render mode is Synthesia or scrolling staff, and the
  player starts playback from the beginning
- **THEN** a scored run begins and note judgments accumulate at each gated onset, using the
  Wait-Mode timing model, without altering the Wait-Mode gating behavior

#### Scenario: Partition view is not scored
- **WHEN** the render mode is the engraved Partition view
- **THEN** no scored run is active regardless of the Wait Mode setting

#### Scenario: Scoring is scoped to the selected hand(s)
- **WHEN** the player has selected a single hand and starts a scored run
- **THEN** only that hand's notes are judged and the result records the hand selection

#### Scenario: Changing hands restarts scoring from the top
- **WHEN** the selected hand(s) change while a scored run is active
- **THEN** the playhead returns to the start and a fresh scored run begins for the new
  selection, keeping the gauge/effects active and still ending in a summary

### Requirement: Per-Note Timing Judgment

For each expected note onset in the visible hand(s), scoring SHALL classify the
player's attack of that pitch into ordered timing verdicts (e.g. perfect, good, early,
late, missed) using tolerance windows expressed in milliseconds, such that the tightest
window scores highest. The measured quantity SHALL depend on the mode:

- **Wait Mode off (free tempo):** the signed offset between the actual press time and the
  note's scheduled onset time — an attack before the onset is `early`, after it is `late`.
- **Wait Mode on (gated):** the **reaction time** from the moment the playhead reaches
  the onset and its gate opens to the correct attack — a fast reaction scores highest and a
  slow one scores a lower verdict; absolute-tempo offset is not used because time is frozen
  at the gate.

Only the first qualifying attack of the required pitch SHALL bind to a given onset; later
presses of the same pitch SHALL NOT re-satisfy an onset already judged.

#### Scenario: On-time attack scores perfect (Wait Mode off)
- **WHEN** Wait Mode is off and the player presses the required pitch within the tightest
  timing window of the note's scheduled onset
- **THEN** that onset is judged `perfect`

#### Scenario: Slightly-off attack scores a lower verdict (Wait Mode off)
- **WHEN** Wait Mode is off and the player presses the required pitch inside the wider
  window but outside the tightest one, before the note's onset window closes
- **THEN** that onset is judged `good`, `early`, or `late` according to the signed offset

#### Scenario: Fast reaction scores perfect (Wait Mode on)
- **WHEN** Wait Mode is on and the player attacks the correct pitch very soon after the
  gate opens on that onset
- **THEN** that onset is judged `perfect` on the reaction-time scale

#### Scenario: Slow reaction scores a lower verdict (Wait Mode on)
- **WHEN** Wait Mode is on and the player takes longer to attack the correct pitch after
  the gate opens
- **THEN** that onset is judged a lower verdict (`good`/`late`) according to the reaction time

#### Scenario: Never-pressed onset is missed
- **WHEN** (Wait Mode off) the note's onset window closes and the required pitch was never
  pressed within any timing window
- **THEN** that onset is judged `missed`

#### Scenario: A single hold binds to one onset only
- **WHEN** a pitch has already bound to and satisfied one onset
- **THEN** the same continuous hold does not satisfy a later onset of the same pitch

### Requirement: Correctness And Extra-Note Judgment

Scoring SHALL treat a press of a pitch that no open onset requires (within the timing
windows) as an extra/wrong note and SHALL record it against the run without crediting
any onset. A wrong note SHALL reduce the run's correctness dimension but SHALL NOT
freeze or alter playback.

#### Scenario: Wrong pitch is recorded as an extra note
- **WHEN** the player presses a pitch that no currently-open onset expects
- **THEN** the press is recorded as an extra note and lowers the correctness dimension,
  and playback continues unaffected

#### Scenario: Correct pitch at the right onset is not an extra note
- **WHEN** the player presses a pitch that an open onset requires within its timing window
- **THEN** the press credits that onset and is not counted as an extra note

### Requirement: Sustain Judgment

For each correctly-attacked note, scoring SHALL evaluate how long the pitch was held
relative to the note's intended duration and SHALL derive a sustain ratio bounded to a
sensible range (a note released far too early scores low; a note held for roughly its
intended duration scores high). Releasing after the intended duration SHALL NOT be
penalized beyond the maximum.

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

### Requirement: Rolling Synchronization Percentage

Scoring SHALL expose a live synchronization percentage in the range 0–100 derived by
combining the timing, correctness, and sustain dimensions accumulated so far in the run.
The percentage SHALL update as onsets are judged so the on-screen gauge can reflect the
current run quality, and it SHALL be defined (a sensible starting value) before any note
has been judged.

#### Scenario: Percentage rises with good play
- **WHEN** the player accumulates a sequence of `perfect`/`good` on-time, well-sustained
  notes
- **THEN** the live synchronization percentage trends upward toward 100

#### Scenario: Percentage falls with poor play
- **WHEN** the player accumulates missed onsets, wrong notes, or poorly-sustained notes
- **THEN** the live synchronization percentage trends downward

#### Scenario: Defined before the first judgment
- **WHEN** a scored run has just started and no onset has been judged yet
- **THEN** the synchronization percentage has a defined starting value and the gauge renders

### Requirement: Per-Onset Mode Stamping And Run Classification

The system SHALL stamp each judged onset with the Wait-Mode state that was active **at the
moment that onset was judged**, because Wait Mode may be toggled on or off at any moment
during a run, and SHALL carry the metric appropriate to that mode (tempo offset when Wait
Mode was off, reaction time when Wait Mode was on). Toggling Wait Mode mid-run SHALL NOT reset,
discard, or invalidate the run. At the end of the run the system SHALL derive a run-level
classification from the per-onset stamps: `free` when every judged onset was Wait-Mode-off,
`wait` when every judged onset was Wait-Mode-on, otherwise `mixed`, together with the count
of onsets judged in each mode (so the dominant mode is derivable).

Because Wait-Mode (reaction) and free-run (tempo) timing measure different things, the
system SHALL compute a separate synchronization sub-score for each mode over only that
mode's onsets, so a run can feed the reaction leaderboard, the tempo leaderboard, or both.
A sub-score for a mode with no onsets SHALL be absent (not zero).

#### Scenario: Onset stamped with the mode active at judgment
- **WHEN** an onset is judged while Wait Mode is on, and a later onset is judged after Wait
  Mode has been turned off
- **THEN** the first onset is stamped `wait` with a reaction metric and the second is stamped
  `free` with a tempo-offset metric

#### Scenario: Toggling mid-run does not invalidate the run
- **WHEN** the player toggles Wait Mode one or more times during a scored run
- **THEN** the run continues, all onsets keep accumulating, and the run still finalizes into a
  session result

#### Scenario: Run classified free / wait / mixed
- **WHEN** a run finalizes with onsets judged only under Wait-Mode-off, only under
  Wait-Mode-on, or under both
- **THEN** its classification is `free`, `wait`, or `mixed` respectively, with per-mode onset
  counts recorded

#### Scenario: Mixed run yields two sub-scores
- **WHEN** a `mixed` run finalizes
- **THEN** it carries both a tempo sub-score (over its free onsets) and a reaction sub-score
  (over its wait onsets), each usable by its respective leaderboard

#### Scenario: Pure run yields one sub-score
- **WHEN** a `free` (or `wait`) run finalizes
- **THEN** it carries the tempo (or reaction) sub-score and the other sub-score is absent

### Requirement: Immutable Session Result Record

At the end of a scored run the system SHALL produce an immutable session-result record
containing the overall synchronization percentage, the per-mode synchronization sub-scores
and per-mode onset counts, the run classification (`free`/`wait`/`mixed`), the per-dimension
aggregates (timing, correctness, sustain), the mean signed free-run timing offset and the
mean Wait-Mode reaction time (each absent when that mode had no hits), the count of onsets by
verdict, the best combo/streak, the piece identity and selected hand(s), and the per-note
judgment list (each carrying its mode stamp and its signed offset or reaction time) needed to
drive a replay. The record SHALL be serializable so a
later change can upload it to the server and route it to the correct leaderboard(s).

#### Scenario: Result produced at song end
- **WHEN** a scored run reaches the end of the piece
- **THEN** an immutable session-result record with the aggregates, per-mode sub-scores, run
  classification, per-note judgments, and piece identity is produced

#### Scenario: Result is serializable
- **WHEN** a session-result record exists
- **THEN** it can be serialized to and restored from a plain data representation without
  loss of the fields the summary, the per-mode leaderboards, and future server sync require
