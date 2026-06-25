## Why

Partition mode is a static engraving: `PartitionPainter` is "geometry only — no
playback or interaction", so during playback nothing moves, no note lights up,
and the view does not follow along. The other modes (Synthesia, Staff) show the
playhead and highlight the notes due now; Partition should too, so a learner can
read the score while it plays.

## What Changes

- Derive a **per-measure playback time** (each measure's start in ms) alongside
  the existing note timeline, so any playback position maps to a measure and a
  fraction within it.
- Add a **playhead cursor** in Partition mode: a vertical line that marks the
  current playback position, moving across measures and systems as `elapsedMs`
  advances (and freezing with Wait Mode, since it reads the same playhead).
- **Highlight the notes at the playhead** using the same expected/correct colors
  as the keyboard feedback, so the current notes stand out on the score.
- **Auto-scroll with look-ahead**: keep the cursor in view and **pre-reveal the
  upcoming measure/system** (still hidden below) before the current measure ends,
  so reading stays smooth instead of jumping at the last moment.

## Capabilities

### Modified Capabilities
<!-- None: nothing in score-notation forbids a playhead, so this is additive. -->

### New Capabilities
<!-- These ADD requirements to the existing score-notation capability spec. -->
- `score-notation`: per-measure timing derivation plus a Partition-mode playhead
  cursor, current-note highlighting, and look-ahead auto-scroll.

## Impact

- **Derivation**: `state/notation_playback.dart` — `DerivedPlayback` gains
  `measureStartMs` (per-measure start in ms); computed from the existing
  `measureStartDiv * msPerDivision`.
- **State**: `state/player_data.dart` / `player_notifier.dart` — carry
  `measureStartMs` so the player exposes the current measure + fraction at
  `elapsedMs`.
- **Painter**: `painters/partition_painter.dart` — accept `elapsedMs` +
  `measureStartMs` (+ song end), draw the cursor on the active system and accent
  the active notes; recompute on playhead change (`shouldRepaint`).
- **View**: `screens/player_screen.dart` `_PartitionView` — drive a
  `ScrollController` to follow the cursor's system with a look-ahead lead.
- **Tests**: unit tests for `measureStartMs` and the elapsed→measure/fraction
  mapping; widget test that the Partition painter repaints and the scroll offset
  advances with the playhead.
- No Rust/engine or public-API change. Additive to `score-notation` (no requirement
  modified) → no overlap with the other in-flight changes.
