# measure-range-practice Specification

## Purpose
TBD - created by archiving change add-measure-range-practice. Update Purpose after archive.
## Requirements
### Requirement: Active measure range on a run

A run SHALL carry an optional **active measure range** `[startMeasure, endMeasure]` over the
piece's measures. When unset, the range SHALL be the **whole piece** and the run SHALL be a
**full run** behaving exactly as a run does today. When the range is narrower than the whole
piece, the run SHALL be a **selective run**. The range SHALL be constrained to valid measure
indices (`0 ≤ startMeasure ≤ endMeasure ≤ lastMeasure`); a single-measure range
(`startMeasure == endMeasure`) SHALL be allowed. Setting the range SHALL move the effective
start/end of playback to that range's bounds (via the measure→time mapping), so playback both
begins and ends within the selected measures.

#### Scenario: Default range is the whole piece
- **WHEN** no measure range has been chosen for a run
- **THEN** the run covers the whole piece and behaves as a full run (same start, end, and
  scoring as today)

#### Scenario: A narrower range makes a selective run
- **WHEN** the user sets a range narrower than the whole piece
- **THEN** playback starts at the range's first measure and ends at the range's last measure

#### Scenario: Single-measure range is allowed
- **WHEN** the user sets `startMeasure == endMeasure`
- **THEN** that one measure is the active range

#### Scenario: Range is clamped to valid measures
- **WHEN** a requested range falls outside `[0, lastMeasure]` or has `start > end`
- **THEN** it is clamped/normalized to a valid in-bounds range

### Requirement: Selective runs are unscored practice

A selective run SHALL NOT begin a scored run: it SHALL NOT arm performance scoring, SHALL NOT
produce a sync% grade, combo, or `SessionResult`, SHALL NOT upload a scored session, and SHALL
NOT show the end-of-run scoring summary. It SHALL still play the selected measures with the
normal audio, hand-selection, metronome, and Wait-Mode gating behavior. Only a **full run**
SHALL be scored.

#### Scenario: Selective run does not arm scoring
- **WHEN** the user starts a selective run
- **THEN** no scored run begins, no sync%/combo is tallied, and no scoring summary is shown

#### Scenario: Selective run does not upload a scored session
- **WHEN** a selective run ends
- **THEN** no `SessionResult` is captured or uploaded to the scoring/leaderboard path

#### Scenario: Full run is still scored
- **WHEN** the user starts a full run (range is the whole piece) from the beginning
- **THEN** a scored run begins and ends in a summary exactly as before

#### Scenario: Wait Mode still gates in a selective run
- **WHEN** Wait Mode is on during a selective run
- **THEN** playback is onset-gated within the range as usual, but the run remains unscored

### Requirement: Looping the range

A selective run SHALL be able to **loop** its active range. When looping, reaching the range's
end SHALL wrap playback back to the range's start (silencing any held notes at the seam) and
continue. The user SHALL be able to set a **loop count** — a finite number of repetitions
`N ≥ 1` or **infinite**. With a finite count, playback SHALL stop (and silence) after the
Nth completed pass; with infinite, it SHALL continue until the user stops it. When looping is
off, a selective run SHALL stop at the range end.

#### Scenario: Wrap at range end when looping
- **WHEN** a looping selective run reaches the range's last measure
- **THEN** playback wraps to the range's first measure and continues, with held notes silenced
  at the wrap

#### Scenario: Finite loop count stops after N passes
- **WHEN** the loop count is a finite `N` and the Nth pass completes
- **THEN** playback stops at the range end and held notes are silenced

#### Scenario: Infinite loop continues until stopped
- **WHEN** the loop count is infinite
- **THEN** playback keeps wrapping until the user stops or exits

#### Scenario: No loop stops at range end
- **WHEN** looping is off and a selective run reaches the range end
- **THEN** playback stops at the range end (no wrap)

### Requirement: Practice tempo ramp

A looping selective run SHALL support an optional **tempo ramp**: at each wrap the playback
speed SHALL increase by a chosen step, **clamped** to the player's maximum speed and never
below the run's starting speed. The ramp SHALL apply only to selective runs and SHALL be off by
default.

#### Scenario: Speed increases each lap when ramp is on
- **WHEN** the tempo ramp is enabled with a positive step and a lap completes
- **THEN** the next lap plays at the previous speed plus the step

#### Scenario: Ramp is clamped to the maximum speed
- **WHEN** ramping would exceed the player's maximum speed
- **THEN** the speed is clamped to the maximum and does not increase further

#### Scenario: Ramp does not affect full runs
- **WHEN** a full run plays
- **THEN** no tempo ramp is applied regardless of the ramp setting

### Requirement: Range selection via steppers and tapping the score

The user SHALL be able to set the active range **two ways**, both driving the same range:
- **Measure steppers** (a from-measure and a to-measure control) available in **every** render
  mode; and
- **Tapping the engraved Partition view** — a first tap selects the range's start measure, a
  second tap selects its end measure (order-normalized), and the current selection is visually
  indicated on the score. Tapping the score SHALL only be required in the Partition render mode;
  the steppers SHALL cover the other render modes.

#### Scenario: Steppers set the range in any render mode
- **WHEN** the user adjusts the from/to measure steppers in any render mode
- **THEN** the active range updates to the chosen measures

#### Scenario: Tapping two measures sets the range on the score
- **WHEN** the render mode is Partition and the user taps a start measure then an end measure
- **THEN** the active range spans those two measures (normalized so start ≤ end) and is shown
  highlighted on the score

#### Scenario: Re-tapping resets the selection
- **WHEN** a range is already selected on the score and the user taps again
- **THEN** a new selection begins from the tapped measure

### Requirement: Per-score persistence of practice settings

The active range and loop settings (loop on/off, loop count, tempo ramp) SHALL be **persisted
per musical score** locally and SHALL be pre-filled when that score is reopened. On load, a
saved range SHALL be **clamped** to the score's current measure count, falling back to the whole
piece if it is no longer valid.

#### Scenario: Saved settings pre-fill on reopen
- **WHEN** the user set a practice range and loop settings for a score, left, and reopens it
- **THEN** the saved range and loop settings are pre-filled

#### Scenario: Stale saved range is clamped on load
- **WHEN** a saved range no longer fits the score's current measure count
- **THEN** it is clamped to valid measures, or reset to the whole piece if it cannot be salvaged

### Requirement: A practice session counts as activity, not score

A completed practice (selective) session SHALL be recorded as a **scoreless activity event**
so it contributes to the profile's daily activity as a **practice count**, while carrying **no**
sync%/scoring data. The event SHALL be recorded **once per practice session** (not once per
lap), so looping does not inflate the count.

#### Scenario: Practice contributes to daily activity
- **WHEN** a practice session completes
- **THEN** a scoreless activity event is recorded for that day and increments the day's practice
  count

#### Scenario: Looping does not inflate the practice count
- **WHEN** a practice session loops many times before the user stops
- **THEN** it is counted as a single practice for that day

#### Scenario: Practice carries no scoring data
- **WHEN** a practice activity event is recorded
- **THEN** it contains no sync%/`SessionResult` and never enters the scored-session/leaderboard
  path

### Requirement: In-game transport measure rewind

The game screen's transport bar SHALL offer a **measure-rewind control**, kept
alongside the existing restart-from-top control. A tap SHALL move the playhead to the
start of the measure containing it when the playhead is meaningfully past that start,
and otherwise to the start of the previous measure, so repeated taps stack one measure
at a time. The rewind SHALL be clamped at the run's **effective start** (the active
range's first measure on a selective run, else the top of the piece). Each rewind
SHALL silence held voices, reset Wait-Mode gate state, and discard any in-flight
scored run; a run that was rewound SHALL NOT produce a score, summary, or upload (a
scored run only re-arms from the top, as today). The playing/paused state SHALL be
preserved across a rewind. The control SHALL be disabled when the piece has no
measure table.

#### Scenario: Tap mid-measure returns to the top of that measure
- **WHEN** the playhead is meaningfully inside measure `m` and the user taps rewind
- **THEN** the playhead moves to the start of measure `m` and playback state
  (playing/paused) is unchanged

#### Scenario: Taps stack back one measure at a time
- **WHEN** the playhead sits at the start of measure `m` and the user taps rewind
- **THEN** the playhead moves to the start of measure `m − 1`, and further taps keep
  stepping one measure back

#### Scenario: Rewind clamps at the effective start
- **WHEN** the playhead is at the run's effective start (range start on a selective
  run, else the top of the piece) and the user taps rewind
- **THEN** the playhead stays at the effective start

#### Scenario: A rewound run is never scored
- **WHEN** a scored full run is in flight and the user taps rewind
- **THEN** the in-flight scored run is discarded, playback continues unscored, and no
  sync%/summary/upload is produced for that run

#### Scenario: Wait-Mode gate state resets on rewind
- **WHEN** Wait Mode is blocked on an onset and the user taps rewind
- **THEN** held-note and gate latches are cleared and the cascade resumes from the
  rewound position's onsets

#### Scenario: Disabled without a measure table
- **WHEN** the loaded piece carries no measure table (e.g. the demo score)
- **THEN** the rewind control is disabled and a tap changes nothing

### Requirement: Dedicated in-game measure-selection mode

A **long-press** on the measure-rewind control SHALL open a dedicated full-screen
measure-selection mode: the engraved score scrolling **vertically**, with **no
keyboard displayed**, under a **title bar** of its own. Entering the mode SHALL pause
playback and SHALL otherwise leave the play session untouched. The mode SHALL be
available in every render mode of the game screen. Inside the mode, a first tap
SHALL draft the range's start measure and a second tap its end (order-normalized);
tapping again SHALL begin a new draft; the draft SHALL be highlighted on the score
and displayed in the title bar. The draft SHALL NOT affect the play session until
confirmed. The mode SHALL be unavailable when the piece has no engraved notation or
no measure table. When the run is already selective, the draft SHALL pre-fill from
the active range; otherwise it SHALL pre-fill from the score's saved practice
settings when present (this mode is where per-score saved settings surface, now
that the pre-play setup modal no longer carries a range picker).

#### Scenario: Long-press opens the selection mode paused
- **WHEN** the user long-presses the measure-rewind control while a piece with
  notation is loaded
- **THEN** playback pauses and a full-screen vertical score view opens with a title
  bar and no keyboard, the session otherwise unchanged

#### Scenario: Two taps draft a highlighted range
- **WHEN** the user taps a first measure then a second measure in the selection mode
- **THEN** the draft range spans those measures (normalized so start ≤ end), is
  highlighted on the score and shown in the title bar, and the play session is not
  yet affected

#### Scenario: Re-tapping restarts the draft
- **WHEN** a complete draft exists and the user taps another measure
- **THEN** a new draft begins from the tapped measure

#### Scenario: Confirm applies the range as a selective run
- **WHEN** a complete draft exists and the user confirms
- **THEN** the mode closes and the drafted range becomes the active measure range,
  with all existing selective-run semantics (unscored practice, playhead at the range
  start, per-score persistence)

#### Scenario: Cancel leaves the session untouched
- **WHEN** the user cancels or navigates back from the selection mode
- **THEN** the mode closes and the play session is exactly as on entry (still paused,
  same playhead, same range)

#### Scenario: Whole-piece action clears the range
- **WHEN** the user chooses the whole-piece action in the selection mode
- **THEN** the mode closes, the active range clears back to a full run, and the
  playhead returns to the piece's effective start

#### Scenario: Saved settings pre-fill the draft
- **WHEN** the run is a full run, the score has per-score saved practice settings,
  and the user opens the selection mode
- **THEN** the draft pre-fills from the saved range (clamped to the current measure
  count), and nothing applies until confirmed

#### Scenario: Unavailable without notation or measure table
- **WHEN** the loaded piece has no engraved notation or no measure table
- **THEN** the long-press does not open the selection mode

