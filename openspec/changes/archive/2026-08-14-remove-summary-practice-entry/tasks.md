# Tasks — remove-summary-practice-entry

## 1. Summary modal

- [x] 1.1 Drop `SummaryAction.practice` and the `summary-practice` button from
      `session_summary_modal.dart`; "retry" returns to a full-width secondary button
      now that it is alone on its row
- [x] 1.2 Drop the `SummaryAction.practice` branch (and the picker import) from
      `player_screen.dart`'s `_onScoredRunFinished`

## 2. Dead code

- [x] 2.1 Delete `widgets/practice_range_picker.dart`,
      `widgets/practice_range_controls.dart` and `widgets/practice_score_strip.dart`
      — the picker was their only remaining caller
- [x] 2.2 Delete `test/widgets/practice_range_picker_test.dart`; keep
      `practice_range_ui_test.dart` and `measure_select_screen_test.dart` (in-game flow)

## 3. Localization

- [x] 3.1 Remove `practicePickerTitle`, `practicePickerStart`, `practiceFromBar`,
      `practiceUnscoredNote`, `summaryPractice` from `app_{en,fr,es,it}.arb`
      (keep `practiceToBar` — still used by the in-game selection flow); regenerate

## 4. Tests & gates

- [x] 4.1 Update `session_summary_ui_test.dart`: the practice action is asserted
      **absent**, and the short-viewport test no longer expects its label
- [x] 4.2 `flutter analyze`, `dart run custom_lint`, `dart format`, full
      `flutter test --exclude-tags golden` green
