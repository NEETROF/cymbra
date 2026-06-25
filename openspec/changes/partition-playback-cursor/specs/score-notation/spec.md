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

### Requirement: Partition Auto-Scroll With Look-Ahead

In Partition mode the view SHALL scroll automatically to keep the playhead cursor
visible, and SHALL pre-reveal the upcoming measure/system — bringing the next
system into view before the current measure ends — so the reader sees what is
coming rather than the view jumping when the cursor reaches the bottom. When the
whole score fits in the viewport, no scrolling SHALL occur.

#### Scenario: View follows the cursor
- **WHEN** the cursor advances toward the bottom of the visible area
- **THEN** the view scrolls so the cursor stays visible

#### Scenario: Upcoming system pre-revealed
- **WHEN** the playhead nears the end of the current measure and the next system
  is still below the fold
- **THEN** the view scrolls ahead so the upcoming system becomes visible before
  the current measure ends

#### Scenario: No scroll when it all fits
- **WHEN** every system fits within the viewport
- **THEN** the view does not scroll
