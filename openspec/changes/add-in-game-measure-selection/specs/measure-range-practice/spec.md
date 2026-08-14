## ADDED Requirements

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
