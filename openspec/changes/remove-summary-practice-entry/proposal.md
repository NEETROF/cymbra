# Drop the summary modal's practice-a-section entry

## Why

`add-in-game-measure-selection` moved range selection into the game screen itself
(long-press the transport rewind → full-screen measure-selection mode) and, at the
time, deliberately kept the end-of-run summary's "practice a section" dialog because
it reused the same shared widgets. With the in-game mode shipped and validated, that
second entry point is redundant: it is a *third* dialog on top of an already-modal
end-of-run flow, it drives a stepper/strip UI the player no longer meets anywhere
else, and it duplicates the saved-range pre-fill that now lives in the selection mode.
The player who wants to drill a passage closes the summary and long-presses rewind.

## What Changes

- **REMOVED: the summary modal's "practice a section" action** — the button and the
  `SummaryAction.practice` case disappear. The summary's explicit choices become
  replay-mistakes / retry / quit (close cross); "retry" goes back to a full-width
  secondary button now that it is alone on its row.
- **REMOVED: the measure-range picker dialog** (`practice_range_picker.dart`) and the
  widgets that existed only to serve it — `PracticeRangeControls` (the from/to
  steppers) and `PracticeScoreStrip` (the embedded engraved strip). Their l10n keys
  (`practicePickerTitle`, `practicePickerStart`, `practiceFromBar`,
  `practiceUnscoredNote`, `summaryPractice`) are dropped in all locales.
  `practiceToBar` stays — the in-game flow still uses it.
- **Unchanged:** every selective-run semantic. The in-game selection mode remains the
  single deliberate range-selection surface, with the same unscored-practice rules,
  per-score persistence, and saved-range pre-fill.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `session-summary`: MODIFIED requirement — the end-of-song summary modal offers
  replay / retry / quit and no longer opens a measure-range picker.

### Removed Capabilities

None.

## Impact

- `apps/music/lib/widgets/session_summary_modal.dart` — enum + actions row
- `apps/music/lib/screens/player_screen.dart` — the `SummaryAction.practice` branch
- Deleted: `practice_range_picker.dart`, `practice_range_controls.dart`,
  `practice_score_strip.dart`, `test/widgets/practice_range_picker_test.dart`
- `apps/music/lib/l10n/app_{en,fr,es,it}.arb`
- No backend, no persistence, no scoring change.
