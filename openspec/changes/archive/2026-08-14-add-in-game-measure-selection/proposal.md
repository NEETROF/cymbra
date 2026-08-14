# In-game measure navigation & selection

## Why

Choosing a partial play session (start/end measure) currently lives in the pre-play
setup modal — the player must leave the flow of the game, open a modal, and drive
steppers to drill the passage they just fumbled. The one in-game path (tapping the
engraved Partition) only exists in Partition render mode, applies immediately on the
second tap (easy to trigger by accident), and is unavailable on phones where the
Partition segment is hidden. Practicing a passage should be reachable from the game
screen itself, in every render mode, in one gesture.

## What Changes

- **Transport measure rewind** — a new button in the play/wait transport bar (kept
  alongside the existing restart-from-top button): each tap moves the playhead back
  one measure; repeated taps stack, clamped at the effective start of the run
  (practice-range start, else the top of the piece). Works while playing or paused.
  Rewinding discards any in-flight scored run (a scored run only arms from the top,
  so leaderboard integrity is preserved by construction — same rule as selective
  runs).
- **Dedicated measure-selection mode** — long-pressing the rewind button opens a
  full-screen selection view: the engraved score scrolling vertically, **no keyboard
  displayed**, with its own title bar showing the draft range plus confirm / cancel /
  whole-piece actions. Tapping a first and a last measure drafts the range
  (order-normalized, re-tap restarts, draft highlighted on the score). **Nothing
  touches the live session until confirm**: confirm applies the range through the
  existing selective-run path (unscored practice, playhead to range start, per-score
  persistence); cancel returns to the game unchanged; "whole piece" clears the range.
- **Guided-coaching step for the new control** — the player's guided tour (piano
  sound → MIDI device → hands) gains a final step pointing at the measure-rewind
  button, teaching both gestures (tap = back one bar, long-press = pick a passage).
  Replayable from the help/tips screen like the rest of the tour, no input barrier.
- **REMOVED: the pre-play setup modal's practice step** — the full-run vs section
  run-type toggle and the second-step range picker disappear from the setup modal;
  the in-game mode becomes the only deliberate range-selection surface. The modal
  becomes range-neutral (it never sets or clears the active range). The per-score
  saved-range pre-fill (D7) relocates into the new selection mode. The end-of-run
  summary's "practice this section" picker dialog is kept (it reuses the same
  shared widgets).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `measure-range-practice`: ADDED requirements — in-game transport measure rewind,
  and the dedicated full-screen selection mode as the deliberate range-selection
  surface. Note: this capability's base spec is itself the in-flight change
  `add-measure-range-practice`; this change stacks on it and both deltas merge into
  `openspec/specs/measure-range-practice/` at archive time.
- `pre-play-setup`: REMOVED requirement — "Full-run vs selective-practice choice in
  setup" (added by the same in-flight parent change). **Archive-order constraint**:
  `add-measure-range-practice` must archive before this change so the requirement
  exists in the main spec when the removal folds in.
- `feature-discovery`: MODIFIED requirement — "Guided in-context coaching for the
  player's controls" adds the measure-rewind control (tap + long-press) to the
  guided tour's enumerated steps.

## Impact

**Products**: Cymbra Music only (Flutter app). No backend, no back-office, no ID
changes.

**Consumed (unchanged)**:
- Selective-run semantics and setters — `setPracticeRange` / `clearPracticeRange`
  (`apps/music/lib/state/player_notifier.dart`), unscored-by-construction arming
  (`_maybeStartRun` refuses selective runs and non-top starts), per-score
  persistence (`practice_settings_store.dart`).
- Partition engraving + measure hit-testing — `PartitionPainter.hitRects`,
  `measureAtPosition`, practice-range tint
  (`apps/music/lib/painters/partition_painter.dart`).

**New/modified code**:
- `apps/music/lib/screens/player_screen.dart` — `_TransportBar` gains the rewind
  button (tap + long-press), route/overlay to the new selection view.
- New widget/screen for the selection mode (vertical Partition, title bar, draft
  range state, per-score saved-range pre-fill).
- `player_notifier.dart` — a `rewindOneMeasure()` transport action (playhead move,
  gate/held reset, scored-run cancel).
- `apps/music/lib/screens/pre_play_setup_modal.dart` — the practice section, the
  practice step, and the modal's range apply/pre-fill logic are removed (the modal
  no longer touches the practice range). `PracticeRangeControls` /
  `PracticeScoreStrip` stay — the end-of-run summary dialog
  (`practice_range_picker.dart`) still uses them.
- Coaching: `coaching_notifier.dart` (`PlayerCoachStep` gains a step),
  `coach_mark.dart` (`CoachAnchor` for the rewind button), `coach_copy.dart` +
  l10n for the step's copy; the help screen's replay picks it up with no further
  change.
- l10n strings for the new button and selection-mode title bar; prune modal-only
  practice keys that become unused.
