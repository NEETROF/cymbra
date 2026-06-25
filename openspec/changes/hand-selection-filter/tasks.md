## 1. State: hand selection + filter helpers

- [ ] 1.1 Add `enum Hand { left, right, both }` in `state/player_data.dart`.
- [ ] 1.2 Add `@Default(Hand.both) Hand selectedHands` field to `PlayerData` (Freezed); run `build_runner`.
- [ ] 1.3 Add `bool showsStaff(int staff)` on `PlayerData` (`both`→true; `right`→staff==1; `left`→staff>=2) as the single visibility predicate.
- [ ] 1.4 Add `List<TimedNote> get visibleNotes => notes.where((n) => showsStaff(n.staff)).toList()`.
- [ ] 1.5 Restrict `requiredNotesAt(t)` to `visibleNotes` so the gate ignores the hidden hand.
- [ ] 1.6 Add `void setSelectedHands(Hand hand)` to the `Player` notifier (`state = state.copyWith(...)`).

## 2. Painters: filter notes and collapse the hidden staff

- [ ] 2.1 Feed `visibleNotes` (or `Hand` + `showsStaff`) into `SynthesiaPainter` so only the selected hand's columns fall.
- [ ] 2.2 In `StaffPainter`, compute `twoStaff` from the filtered notes so an unselected staff collapses and the kept staff recentres; draw only the kept staff's notes.
- [ ] 2.3 In `PartitionPainter`, parametrise the kept-staff Y-origin/clef on `selectedHands`; when a single hand is selected draw only that staff's lines, clef, key/time signature and notes (collapse the other). Keep the `both` path unchanged.
- [ ] 2.4 Wire the painters' construction sites (`player_screen.dart` / notation view) to pass the current selection from `playerProvider`.

## 3. UI: hand selector in the top bar

- [ ] 3.1 Add a `_HandSelector` widget to `_TopBar` in `screens/player_screen.dart`, matching the `_RangeChooser` pattern (Left/Right/Both), bound to `notifier.setSelectedHands` and reflecting `data.selectedHands`.
- [ ] 3.2 Ensure the selector is present in all three render modes (it lives in the mode-independent top bar).
- [ ] 3.3 (Per design open question) Show the selector only when the loaded document has ≥2 staves, or keep it always visible with the harmless `both` default — decide and implement.

## 4. Tests

- [ ] 4.1 Unit-test `showsStaff`, `visibleNotes`, and `requiredNotesAt` for Left/Right/Both, including that a hidden-hand note is absent from the required set (Wait Mode advances).
- [ ] 4.2 Unit-test `setSelectedHands` updates state immutably and defaults to `both`.
- [ ] 4.3 Widget-test the `_HandSelector`: shows current selection and dispatches the setter; present across modes.
- [ ] 4.4 Widget/paint test that Staff and Synthesia exclude the unselected hand's notes (e.g. via a paint-capture or note-count assertion on the filtered input).
- [ ] 4.5 Refresh/add goldens (tagged `golden`) for Partition and Staff single-hand collapsed layouts (Left-only and Right-only); confirm `both` goldens unchanged.

## 5. Verify & gate

- [ ] 5.1 `cd apps/music && dart run build_runner build --delete-conflicting-outputs`, then `melos run analyze` and `dart format` clean; `dart run custom_lint` passes.
- [ ] 5.2 `flutter test --coverage --exclude-tags golden` green and Flutter line coverage ≥ 80%.
- [ ] 5.3 Manually confirm in-app: in each mode, switching Left/Right/Both hides the right notes/staff and the hidden hand is not awaited in Wait Mode.
- [ ] 5.4 `openspec validate hand-selection-filter --strict` passes.
