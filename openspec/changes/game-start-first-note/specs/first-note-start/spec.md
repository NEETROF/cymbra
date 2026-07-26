## ADDED Requirements

### Requirement: Effective Start Trims Leading Silence

The effective start of a piece SHALL be derived from the onset of the first sounding note of
the **currently selected hand(s)** — the smallest note onset among the notes actually shown
and played for that selection — so that leading rests and empty leading measures before the
first note are skipped. Leading rests SHALL NOT count as sounding notes.

To preserve the falling-note approach animation and a sense of pulse, the playhead SHALL be
placed a short, bounded lead-in **before** that first onset rather than exactly on it. The
lead-in SHALL be on the order of the note fall-in / about one beat (a small fixed budget, not
the full leading silence) and SHALL be clamped so the start position is never before time
zero. When the first onset is within the lead-in budget of time zero (the piece already
starts at or near the beginning), the effective start SHALL be zero and behaviour SHALL be
unchanged.

A piece that contains no sounding notes for the current selection SHALL fall back to an
effective start of zero. Rests or empty measures that occur **after** the first note SHALL
NOT move the start.

Whenever playback begins fresh (a new run / transport start from the top, not a resume from a
mid-piece pause), the playhead SHALL be placed at this effective start rather than at absolute
zero.

#### Scenario: Leading empty measures are skipped
- **WHEN** a piece's first sounding note begins in the 3rd measure and playback starts from
  the top
- **THEN** the playhead is placed a short lead-in before that note (not at time zero), so the
  empty leading measures beyond the lead-in are skipped

#### Scenario: Short lead-in precedes the first note
- **WHEN** the effective start is computed for a piece with leading rests
- **THEN** the start position is the first note's onset minus the bounded lead-in, clamped to
  no earlier than time zero

#### Scenario: Piece already starting near the beginning is unchanged
- **WHEN** a piece's first sounding note begins at (or within the lead-in budget of) time zero
- **THEN** the effective start is zero and playback begins exactly as before

#### Scenario: A selection with no notes falls back to zero
- **WHEN** the current hand selection has no sounding notes
- **THEN** the effective start is zero

#### Scenario: Later rests do not move the start
- **WHEN** a piece begins with a note near time zero but contains rests or an empty measure
  partway through
- **THEN** the effective start stays at the beginning (only leading silence before the first
  note is trimmed)

### Requirement: Effective Start Applies In Every Render Mode And Wait-Mode State

Starting at the effective start SHALL apply in all three render modes — waterfall (Synthesia),
the horizontal scrolling staff, and the engraved vertical Partition — and with Wait Mode either
on or off. The set of notes that are played or judged SHALL NOT change; only the initial
playhead position does. In every mode the first note SHALL visibly approach over the lead-in
rather than appearing already at the hit line.

#### Scenario: Trimmed start in every render mode
- **WHEN** playback starts from the top with leading rests present, in any render mode
  (Synthesia, scrolling staff, or Partition)
- **THEN** the playhead begins at the same effective start (lead-in before the first note) in
  each mode

#### Scenario: Wait Mode runs the lead-in then freezes at the first note
- **WHEN** Wait Mode is on, the piece has leading rests, and playback starts from the top
- **THEN** the playhead begins at the lead-in before the first note, advances that brief
  lead-in so the note approaches, and the gate freezes at the first note's onset — without
  playing through the full leading rests

#### Scenario: Free run starts at the lead-in before the first note
- **WHEN** Wait Mode is off, the piece has leading rests, and playback starts from the top
- **THEN** the playhead begins at the lead-in before the first note

### Requirement: Every Fresh-Start Transport Honours The Effective Start

Every transport action that starts the piece afresh SHALL place the playhead at the effective
start: the initial score load, restart / Retry, a hand-selection change that restarts the run,
and a loop wrap-around from the end of the piece. Resuming from a mid-piece pause SHALL NOT
re-trim (it continues from where it was paused). Because the start is derived from the selected
hand(s), a hand change SHALL recompute the effective start for the new selection.

#### Scenario: Initial load starts trimmed
- **WHEN** a score with leading rests is first loaded into the player
- **THEN** the playhead rests at the effective start (lead-in before the first note) ready to
  play

#### Scenario: Restart / Retry returns to the effective start
- **WHEN** the player restarts the piece or taps Retry
- **THEN** the playhead returns to the effective start, not to time zero

#### Scenario: Changing hands recomputes and restarts at that hand's first note
- **WHEN** the selected hand(s) change and the run restarts from the top
- **THEN** the effective start is recomputed from the new selection's first note and the
  playhead returns there

#### Scenario: Loop wraps to the effective start
- **WHEN** playback loops from the end of the piece back to the top
- **THEN** the playhead wraps to the effective start, skipping the leading rests

#### Scenario: Resuming mid-piece does not re-trim
- **WHEN** the player pauses partway through and then resumes
- **THEN** playback continues from the paused position without jumping to the first note

### Requirement: Countdown And Scored Run Anchor To The Effective Start

The free-run get-ready countdown and the opening of a scored run SHALL be triggered by a fresh
start at the effective start position (not only when the playhead is at absolute zero). A piece
with leading rests SHALL therefore still get its countdown and open a scored run, and the first
note SHALL arrive no earlier than the countdown clears. The playhead SHALL stay frozen at the
effective start while the countdown runs.

#### Scenario: Countdown and run open for a trimmed start
- **WHEN** Wait Mode is off, the piece has leading rests, and the player starts playback from
  the top
- **THEN** the get-ready countdown runs with the playhead frozen at the effective start, a
  scored run opens, and the first note begins after the countdown reaches zero

#### Scenario: Scored run opens even though the playhead is not at zero
- **WHEN** a run starts from the top with the playhead seeded at a non-zero effective start
- **THEN** the scored run still opens (the run-start condition keys off the effective start,
  not absolute zero)

#### Scenario: Wait Mode still skips the countdown
- **WHEN** Wait Mode is on and playback starts from the top of a piece with leading rests
- **THEN** no countdown is shown and the gate freezes at the first note's onset after the
  lead-in
