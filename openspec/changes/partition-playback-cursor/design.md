## Context

Partition mode renders a parsed `ScoreDocument` laid out into `System`s by the
geometry engine; `PartitionPainter` (`painters/partition_painter.dart`) draws it
and is explicitly "geometry only". The time-based modes share the player's
`elapsedMs` playhead and `requiredNotesAt(elapsedMs)` gate. The derivation
(`notationToTimedNotes`) already walks measures accumulating `measureStartDiv`
and a constant `msPerDivision`, so per-measure ms start times are one line away.

The painter's per-measure x-mapping already exists: within a measure it maps a
division position to x via `xForPosition(position)`, justifying measures across
the width. What's missing is (a) a time→measure/fraction mapping, (b) a cursor +
note accenting in the painter, and (c) a `ScrollController` that follows it.

## Goals / Non-Goals

**Goals:**
- Emit per-measure start times (ms) from the derivation.
- Draw a moving playhead cursor in Partition mode at the right measure/fraction.
- Highlight the notes at the playhead with the keyboard's expected/correct colors.
- Auto-scroll to follow the cursor, pre-revealing the next system (look-ahead).

**Non-Goals:**
- No change to engraving geometry or the layout algorithm.
- No per-tempo-change timing (the derivation uses a single bpm today; keep that).
- No interaction (click-to-seek) — display only.
- No audio (owned by `piano-sound-output`).

## Decisions

### Decision: Carry timing in division space, ms only at the measure boundary
Add `List<int> measureStartMs` to `DerivedPlayback` (and thread it through
`PlayerData`). Map `elapsedMs → (measureIndex, fraction)` with a small pure
helper: find the measure whose `[startMs, nextStartMs)` contains `elapsedMs`,
`fraction = (elapsedMs - startMs) / (nextStartMs - startMs)`. For the **cursor x**
use `fraction` directly against the measure's drawn width. For **note
highlighting** convert to `cursorDiv = fraction × divPerMeasure` and accent notes
in that measure whose `[positionDivisions, +durationDivisions)` contains
`cursorDiv`. **Why:** the painter already thinks in divisions per measure; this
avoids computing per-note ms and reuses existing mapping. **Trade-off:** assumes
uniform tempo within a measure (true today) and treats irregular/pickup measures
by fraction — acceptable for v1.

### Decision: Pass playhead inputs into the painter; keep state minimal
`PartitionPainter` gains `elapsedMs`, `measureStartMs`, `songEndMs`,
`activeNotes`, and `requiredPitches` (or compute required from the same window).
`shouldRepaint` adds `elapsedMs`/`activeNotes` so the cursor animates. The active
measure/fraction is computed in the painter (or a shared pure helper used by both
painter and tests). **Why:** mirrors how Synthesia/Staff painters already take
`elapsedMs`/`activeNotes`; no new state object.

### Decision: Note color reuses the keyboard's three-state logic
A note at the playhead is "expected" unless its pitch is in `activeNotes`, then
"correct" — same precedence as `PianoKeyboardPainter._stateOf`. Reuse the same
`CymbraColors` (secondaryContainer / tertiary) for visual consistency. **Why:**
one mental model across keyboard and score.

### Decision: Auto-scroll via a ScrollController with a look-ahead lead
`_PartitionView` owns a `ScrollController`. On playhead change, compute the
cursor's absolute y (system index × system pitch, known from the painter's
`_systemHeight + _systemGap`). Target offset = cursorSystemTop − leadMargin,
where `leadMargin` is sized so the **next** system is already on-screen before the
current measure ends (e.g. lead ≈ one system height, or scroll when the cursor
passes a threshold fraction of the viewport). Animate with a short duration.
**Why:** simplest way to pre-reveal upcoming music. **Trade-off:** needs the
system-height metric shared from the painter (expose a const/helper).

### Decision: Compute timing in a host-testable helper
Put `measureStartMs` derivation and the `elapsedMs → (measure, fraction)` lookup
in pure functions (in `notation_playback.dart` / `player_data.dart`) so they unit
-test without widgets, satisfying the coverage rule. The painter and scroll math
are exercised by a widget test.

## Risks / Trade-offs

- **Irregular measures / pickup bars** → fraction-based placement may drift from
  true note x; mitigate by mapping fraction→x within the measure's own width
  (not a global), so the cursor stays inside the right measure.
- **Scroll jank** → animate with a short curve and avoid scheduling a scroll
  every tick (only when target moves beyond a small epsilon).
- **No score loaded / demo score** → Partition shows the empty state; cursor code
  must no-op when `measureStartMs` is empty.
- **Coverage** → timing helpers are pure and unit-tested; painter/scroll covered
  by a widget test asserting repaint and a non-zero scroll offset after advancing.

## Migration Plan

Additive and in-memory: a new derived field, painter inputs, and a scroll
controller. Default with no score loaded behaves as today. Rollback removes the
cursor inputs and the controller. No data migration; no public API change.

## Open Questions

- **Look-ahead amount**: lead by a full system vs a fraction of the viewport —
  tune during implementation against a real multi-system score.
- **Cursor style**: thin accent line vs a translucent band over the current beat
  — start with a thin line; revisit if it reads poorly over dense notation.
