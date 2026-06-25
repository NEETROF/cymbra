## Why

When learning a piano piece, players practise one hand at a time. Today every
render mode (Synthesia, Staff, Partition) always shows — and Wait Mode always
awaits — both hands at once, so there is no way to isolate the right or left
hand. Letting the user pick which hand plays, and removing the other hand from
view and from the gate, turns Cymbra into a hands-separate practice tool.

## What Changes

- Add a **hand selection** to player state with three values — **Left**,
  **Right**, **Both** (default Both) — held in immutable state and changed via a
  notifier method, session-only (no persistence; resets to Both on launch).
- Add a **hand selector** to the player settings (reachable from a settings
  control in the top bar), available in all three render modes.
- In **all** modes, notes belonging to the unselected hand are **not displayed
  and not awaited**: they are excluded from each painter's note set and from the
  required-notes gate that drives Wait Mode and the keyboard's expected/correct
  feedback.
- In **Staff** and **Partition** (grand-staff) modes, an unselected hand's
  **entire staff is collapsed** — its staff lines, clef, key/time signature and
  notes are removed — so only the selected hand's staff remains.
- In **Synthesia** mode, only the selected hand's falling-note columns are drawn.
- Staff ↔ hand mapping follows the MusicXML convention already used in the
  engine: **staff 1 = right hand**, **staff 2 = left hand**.

## Capabilities

### New Capabilities
- `hand-selection`: choosing which hand plays (Left/Right/Both) and filtering
  every render mode and the playback gate so unselected-hand notes are neither
  shown nor awaited; collapsing an unselected hand's staff in grand-staff modes.

### Modified Capabilities
- `keyboard-display`: the required-notes gate behind Three-State Key Feedback (and
  Wait Mode) is restricted to the selected hand(s), so a hidden hand's keys are
  never shown as expected.
- `score-notation`: Partition rendering and the derived Staff-mode layout respect
  hand visibility — an unselected hand's staff is collapsed rather than always
  drawing both staves of the grand staff.

## Impact

- **State**: `apps/music/lib/state/player_data.dart` (new `Hand` enum + field,
  note-filter and required-notes helpers), `player_notifier.dart` (setter).
- **UI**: `apps/music/lib/screens/player_screen.dart` (hand selector in the
  settings drawer opened from the top bar).
- **Painters**: `synthesia_painter.dart`, `staff_painter.dart`,
  `partition_painter.dart` consume hand-filtered notes; staff painters collapse a
  hidden staff.
- **Tests**: unit tests for the filter/gate helpers; widget tests for the
  selector and per-mode visibility; golden refresh for collapsed-staff layouts.
- No Rust/engine or public-API changes — filtering is entirely in the Flutter
  layer using the `staff` field already present on notes. No persistence layer.
