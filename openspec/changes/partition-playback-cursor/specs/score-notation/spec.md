## ADDED Requirements

### Requirement: Per-Measure Playback Timing

The derivation SHALL compute, for each measure, its playback start time in
milliseconds, using the same running division position and tempo as the note
timeline (a measure's start is its accumulated start-divisions times the
milliseconds-per-division). This lets any playback position be mapped to the
measure it falls in and the fraction elapsed within that measure.

#### Scenario: Measure start times derived
- **WHEN** a score is derived for playback
- **THEN** each measure has a start time in ms equal to its accumulated
  start-divisions × ms-per-division, and the first measure starts at 0

#### Scenario: Position maps to a measure and fraction
- **WHEN** the playhead is at a time within a measure's span
- **THEN** that measure is identified as current and the fraction elapsed within
  it is the position between its start and the next measure's start

### Requirement: Partition Playhead Cursor

In Partition mode the painter SHALL draw a playhead cursor — a vertical line —
at the current playback position, located on the system and measure that contain
the playhead and at the horizontal fraction corresponding to the elapsed time
within that measure. The cursor SHALL move as the playhead advances and SHALL
freeze when Wait Mode freezes the playhead (it reads the same position). The
cursor SHALL span the staves drawn for that system.

#### Scenario: Cursor placed at the playback position
- **WHEN** Partition mode is shown and the playhead is partway through a measure
- **THEN** a vertical cursor is drawn on that measure at the matching horizontal
  fraction, spanning the system's staves

#### Scenario: Cursor advances with playback
- **WHEN** the playhead advances across measures and systems
- **THEN** the cursor moves to the measure/system containing the new position

#### Scenario: Cursor freezes with Wait Mode
- **WHEN** Wait Mode freezes the playhead at an awaited note
- **THEN** the cursor stops at that position until playback advances

### Requirement: Partition Note Highlighting

In Partition mode the notes sounding at the playhead SHALL be highlighted using
the same expected/correct feedback colors as the on-screen keyboard: a note due
at the playhead but not yet played reads as expected, and a note due and being
played reads as correct. Notes not at the playhead SHALL render in the normal
engraving ink.

#### Scenario: Current note highlighted as expected
- **WHEN** a note's time window contains the playhead and its key is not held
- **THEN** that note head is drawn in the expected color

#### Scenario: Current note highlighted as correct
- **WHEN** a note's time window contains the playhead and its key is held
- **THEN** that note head is drawn in the correct color

#### Scenario: Other notes stay normal
- **WHEN** a note is not at the playhead
- **THEN** it is drawn in the normal engraving ink

### Requirement: Partition Auto-Scroll Per Line

In Partition mode the view SHALL scroll automatically to keep the current line
(system) visible, advancing **once per staff line** rather than per measure: the
vertical position depends only on which system the cursor is in, so the view stays
put while the cursor crosses measures of the same line and moves when the playhead
reaches a new line (no per-measure back-and-forth). When the whole score fits in
the viewport, no scrolling SHALL occur. Look-ahead is provided by the next-line
overlay, not by scrolling ahead.

#### Scenario: View follows the cursor per line
- **WHEN** the playhead moves to a measure on a new system
- **THEN** the view scrolls once so that system is visible

#### Scenario: No scroll within a line
- **WHEN** the cursor crosses measures within the same system
- **THEN** the view does not scroll (the line stays put)

#### Scenario: No scroll when it all fits
- **WHEN** every system fits within the viewport
- **THEN** the view does not scroll

### Requirement: Next-Line Preview Overlay

In Partition mode the player SHALL show a small "next line" overlay — the first
measures (up to two) of the upcoming system — pinned to the top-left of the
viewport, so the reader can see what comes next without scrolling the main view
ahead. The overlay SHALL appear only once the playhead is past the middle of the
current line (when the top-left area holds already-played measures) and only when
a next system exists; it SHALL be hidden otherwise (including on the last line).

#### Scenario: Overlay appears near the end of a line
- **WHEN** the playhead passes the middle of the current line and more lines follow
- **THEN** the first measures of the next line are shown in a top-left overlay

#### Scenario: Hidden early in the line
- **WHEN** the playhead is in the first half of the current line
- **THEN** no next-line overlay is shown

#### Scenario: Hidden on the last line
- **WHEN** the cursor is on the last system of the score
- **THEN** no next-line overlay is shown
