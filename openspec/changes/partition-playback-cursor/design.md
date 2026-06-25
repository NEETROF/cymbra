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

### Decision: Per-line auto-scroll + a next-line overlay (not scroll-ahead)
`_PartitionView` owns a `ScrollController`. The scroll target depends **only on
the cursor's system index** (centred via the painter's `systemTopY`/`systemStride`),
so it advances once per staff line and never oscillates per measure; it re-issues
the animation only when the line changes. Look-ahead is **not** done by scrolling
ahead — instead a small **next-line overlay** (first ≤2 measures of the next
system, engraved by a second `PartitionPainter` scaled down) is pinned top-left,
shown only once the cursor passes the middle of the current line (so it covers
already-played measures) and only when a next system exists. **Why:** an earlier
"pin current line to top to pre-reveal the next line below" approach failed on
short viewports — a grand-staff system nearly fills the viewport, leaving no room
below for the next line. The overlay shows the upcoming measures regardless of
viewport height. **Trade-off:** a second painter instance and a corner that
overlaps the (already-played) start of the line.

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
