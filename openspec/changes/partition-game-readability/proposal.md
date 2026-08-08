# Partition game-mode readability rework

## Why

Playing in the engraved Partition (vertical) mode is hard to read today: the
"NEXT" preview overlay covers part of the score, the floating sync gauge plus its
reserved 104 px right gutter crowd the end of every line, lines pack up to four
measures so dense scores render notes almost side by side, and there is no way
to make the notation bigger. Players report the game mode is cluttered and dense
scores are illegible.

## What Changes

- **Look-ahead auto-scroll replaces the NEXT overlay** (Partition): the current
  system is anchored near the top of the viewport (~30%) instead of centred, so
  the next line is always visible below it, full size, in place. The top-left
  "NEXT" preview overlay is removed entirely. Already-played systems are dimmed
  so the eye anchors on the current line.
- **Scoring HUD moves to the top bar** (all render modes): the floating 88×120
  sync gauge is replaced by a compact horizontal score chip (tier colour + live
  sync % + combo) in the player top bar. The 104 px right gutter reserved for
  the gauge in Partition is removed, reclaiming full engraving width. Tier-up
  celebrations (shake + burst) carry over to the chip. Per-note hit sparks in
  Synthesia/Staff are unchanged.
- **Denser scores get wider measures** (Rust `musicxml-core`): the non-linear
  spacing function gains a minimum per-column advance so short-duration columns
  (16ths and shorter) can no longer collapse to ~13 px; dense measures widen
  naturally and the greedy packer puts fewer of them per line. The fixed cap
  drops from 4 to 3 measures per system.
- **New "score size" setting (S/M/L)** in the player preferences and the
  pre-play/settings modal, applied to **both** notation views:
  - Partition (vertical): scales the staff space and lays systems out against a
    proportionally reduced width, so notes grow and lines re-wrap.
  - Portée/Staff (horizontal): scales the notation via `noteScale` and narrows
    the look-ahead time window proportionally, so notes grow and spacing keeps
    pace.
- **Current-measure highlight** (Partition): a subtle background wash behind the
  measure containing the playhead, in addition to the cursor line.
- **Playback progress bar**: a thin full-width bar directly above the on-screen
  keyboard showing the piece's duration and the playhead's position, in every
  render mode (at the bottom of the render area when the keyboard is hidden).

## Capabilities

### New Capabilities

- `playback-progress`: a thin progress bar above the keyboard reflecting the
  piece's duration and the current playback position.

### Modified Capabilities

- `score-notation`:
  - *Non-linear Measure Geometry*: add a minimum per-column spacing floor.
  - *System Layout*: measures-per-system cap becomes 3.
  - *Partition Auto-Scroll Per Line*: scroll target anchors the current line
    near the viewport top so the next line is visible below (look-ahead by
    scrolling, not by overlay).
  - *Next-Line Preview Overlay*: **removed** (superseded by look-ahead scroll).
  - New requirements: played-system dimming, current-measure highlight, and
    notation size scaling across the Partition and Staff views.
- `gamified-feedback`:
  - *Sync Gauge Display*: the gauge becomes a compact top-bar score chip shown
    in every render mode; no floating box over the play surface and no reserved
    gutter. Tier celebrations target the chip.
- `pre-play-setup`:
  - *Setup modal contents*: adds the score size (S/M/L) control.
  - *Setup choices persist across scores and restarts*: the score size persists
    with the other play settings.

## Impact

- Rust: `crates/musicxml-core` (`space`/`min_width`, `MAX_MEASURES_PER_SYSTEM`,
  tests). No public FFI signature change → no `flutter_rust_bridge` regen. The
  back-office wasm consumes the same layout (stale-wasm gotcha: regenerate with
  `yarn gen:wasm` when testing BO locally).
- Flutter app: `player_screen.dart` (`_PartitionView`, `_TopBar`,
  `_NextLineOverlay` removal, gutter removal), `scoring_overlay.dart`,
  `scoring_gauge.dart` (chip variant), `partition_painter.dart` (staff-space
  parameter, dimming, measure wash), `staff_painter.dart` (already has
  `noteScale`/`lookAheadMs` seams), `player_preferences.dart` (+`scoreSize`),
  `pre_play_setup_modal.dart` (+control), l10n strings, existing widget tests
  for the overlay/gauge/prefs.
- No backend, no proto, no DB.
