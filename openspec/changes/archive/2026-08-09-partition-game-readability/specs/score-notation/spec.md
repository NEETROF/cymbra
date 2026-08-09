# score-notation delta — partition-game-readability

## MODIFIED Requirements

### Requirement: Non-linear Measure Geometry

The geometry engine SHALL compute, for every measure, a minimum width
(`min_width`) derived from the measure's note density using a non-linear spacing
function: shorter durations SHALL receive proportionally more space than a
purely linear mapping of duration to width, and a measure SHALL never be
narrower than a fixed minimum floor. Every time column SHALL additionally
receive at least a fixed minimum column advance regardless of how short its
duration is, so dense passages (16ths and shorter) cannot collapse into
unreadably tight spacing. Spacing SHALL be computed over the union of
time positions across all staves and voices of the measure, so the two staves of
a grand staff stay horizontally aligned; chord members and notes that share a
time position SHALL NOT add horizontal space.

#### Scenario: Denser measures are wider
- **WHEN** measure A contains eight eighth notes and measure B contains two half
  notes, with equal total duration
- **THEN** `min_width(A)` is greater than `min_width(B)`

#### Scenario: Spacing is sub-linear in duration
- **WHEN** comparing a quarter note to a note of twice its duration
- **THEN** the width allotted to the longer note is more than the quarter's but
  less than twice the quarter's (sub-linear growth)

#### Scenario: Short columns respect the minimum column advance
- **WHEN** a measure contains a run of 16th (or shorter) notes
- **THEN** each of its time columns is allotted at least the minimum column
  advance, so the measure's `min_width` grows linearly with the number of such
  columns instead of compressing them

#### Scenario: Longer columns are unaffected by the column floor
- **WHEN** a time column's non-linear spacing already exceeds the minimum column
  advance (e.g. a quarter note or longer)
- **THEN** its width is the non-linear value, unchanged by the floor

#### Scenario: Both staves share one horizontal spacing
- **WHEN** staff 1 and staff 2 have notes at the same time positions in a measure
- **THEN** those positions contribute a single shared column of spacing (not
  double-counted)

#### Scenario: Minimum width floor respected
- **WHEN** a measure contains a single whole-measure rest
- **THEN** its `min_width` is at least the fixed minimum floor

### Requirement: System Layout

The geometry engine SHALL group measures into systems (justified staff lines) for
a given available width: measures SHALL be appended to the current system while
their cumulative `min_width` fits the available width, and a new system SHALL
begin when the next measure would overflow *or* a fixed maximum of **three**
measures per system is reached (so dense scores stay legible on a wide viewport).
A single measure whose `min_width` exceeds the available width SHALL occupy its
own system. Each system SHALL carry the staves of the part (e.g. treble + bass
for piano) so a grand staff is laid out together. The returned layout SHALL
preserve measure order.

#### Scenario: Measures wrap into multiple systems
- **WHEN** the cumulative `min_width` of measures exceeds the available width
- **THEN** the engine starts a new system at the first measure that would
  overflow, preserving order

#### Scenario: Cap on measures per system
- **WHEN** more than three measures would fit width-wise on one line
- **THEN** the engine still wraps to a new system after the third measure,
  preserving order

#### Scenario: Grand staff kept together
- **WHEN** the part has two staves
- **THEN** each system carries both staves so treble and bass render as one
  grand staff

#### Scenario: Oversized measure on its own system
- **WHEN** a single measure's `min_width` exceeds the available width
- **THEN** that measure occupies a system by itself

#### Scenario: All measures fit on one system
- **WHEN** the cumulative `min_width` of up to three measures is within the
  available width and no more measures exist
- **THEN** the engine returns a single system containing every measure in order

### Requirement: Partition Auto-Scroll Per Line

In Partition mode the view SHALL scroll automatically to keep the current line
(system) and the upcoming line visible, advancing **once per staff line** rather
than per measure: the vertical position depends only on which system the cursor
is in, so the view stays put while the cursor crosses measures of the same line
and moves when the playhead reaches a new line (no per-measure back-and-forth).
The scroll target SHALL anchor the current system in the upper part of the
viewport (near the top, not centred) so that the next system is visible below it
whenever one exists and the viewport can show more than one system — look-ahead
is provided by the scroll position itself, in place and at full size. When the
whole score fits in the viewport, no scrolling SHALL occur.

#### Scenario: View follows the cursor per line
- **WHEN** the playhead moves to a measure on a new system
- **THEN** the view scrolls once so that system sits in the upper part of the
  viewport

#### Scenario: Next line visible below the current line
- **WHEN** the playhead is on a system that is not the last and the viewport is
  tall enough for more than one system
- **THEN** the following system is visible below the current one, at full size,
  in its place in the score

#### Scenario: No scroll within a line
- **WHEN** the cursor crosses measures within the same system
- **THEN** the view does not scroll (the line stays put)

#### Scenario: No scroll when it all fits
- **WHEN** every system fits within the viewport
- **THEN** the view does not scroll

## REMOVED Requirements

### Requirement: Next-Line Preview Overlay

**Reason**: Superseded by look-ahead auto-scroll — the current line is anchored
near the top of the viewport so the next line is always visible below it, full
size and in place; a preview box floating over the score is redundant and
occludes the notation.
**Migration**: Delete the overlay widget and its visibility heuristics; the
Partition Auto-Scroll Per Line requirement now guarantees next-line visibility.

## ADDED Requirements

### Requirement: Played-System Dimming

In Partition mode the renderer SHALL dim (reduce the opacity of) every system
that lies entirely before the system containing the playhead while a playhead is
active, so the current and upcoming lines stand out.
The current system and all following systems SHALL render at full opacity. When
no playhead is active (stopped, or the score has no timing) all systems SHALL
render at full opacity.

#### Scenario: Fully played lines are dimmed
- **WHEN** the playhead is on system N during playback
- **THEN** systems 0..N-1 render dimmed while systems N and beyond render at
  full opacity

#### Scenario: No dimming without a playhead
- **WHEN** no playback position exists (score stopped or untimed)
- **THEN** every system renders at full opacity

### Requirement: Current-Measure Highlight

In Partition mode the measure containing the playhead SHALL be emphasised with a
subtle background wash spanning the system's staves for that measure's width, in
addition to the playhead cursor line. The wash SHALL move with the playhead from
measure to measure, SHALL NOT obscure or recolor the engraved glyphs, and SHALL
NOT be shown when no playhead is active.

#### Scenario: Active measure carries a wash
- **WHEN** the playhead is inside a measure during playback
- **THEN** that measure's background is washed with a subtle accent colour while
  neighbouring measures keep the plain background

#### Scenario: Wash follows the playhead
- **WHEN** the playhead crosses into the next measure
- **THEN** the wash moves to the new measure

#### Scenario: No wash when stopped
- **WHEN** no playback position exists
- **THEN** no measure wash is drawn

### Requirement: Notation Size Setting

The player SHALL offer a score size setting with three levels (small, medium,
large; medium is the default) that scales the engraved notation in **both**
notation views. In the Partition (vertical) view the setting SHALL scale the
staff space — and with it every glyph, spacing and system dimension — and
systems SHALL re-wrap so the enlarged line still fits the viewport width. In the
Portée (horizontal scrolling staff) view the setting SHALL scale the staff and
glyphs and adjust the visible time window proportionally, so note size and
horizontal spacing grow together. The setting SHALL apply immediately when
changed.

#### Scenario: Larger size re-wraps the Partition
- **WHEN** the user switches the score size from medium to large in Partition
  mode
- **THEN** the notation renders with a larger staff space and systems re-wrap to
  fit the viewport width (fewer measures per line when needed)

#### Scenario: Larger size scales the Portée view
- **WHEN** the user switches the score size from medium to large in the
  horizontal staff view
- **THEN** the staff, glyphs and note spacing render proportionally larger (a
  shorter time window spans the same width)

#### Scenario: Default size unchanged
- **WHEN** the user has never touched the score size setting
- **THEN** both views render exactly as at the medium (1.0) scale

### Requirement: Notation Paper Theme

The player SHALL offer a notation theme setting with two values — the app's
dark surface (default) and a paper-like light theme — applied to both notation
views (Partition and Portée). The paper theme SHALL render the engraving
dark-on-light (near-black ink on an ivory background) with hand, correct and
accent colours darkened to keep at least a 4.5:1 contrast against the paper
background, so the per-hand colour coding survives the light background. The
setting SHALL apply immediately when changed and SHALL NOT affect the
Synthesia view or the rest of the app's chrome.

#### Scenario: Paper renders dark-on-light with adapted colours
- **WHEN** the user selects the paper theme and opens a notation view
- **THEN** the background is light, glyphs are near-black, and note heads use
  the darkened per-hand palette

#### Scenario: Dark stays the default
- **WHEN** the user has never touched the notation theme
- **THEN** both notation views render on the dark surface exactly as before

#### Scenario: Chrome is unaffected
- **WHEN** the paper theme is active
- **THEN** the top bar, transport and keyboard keep the app's dark theme
