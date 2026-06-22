## 1. Force landscape

- [x] 1.1 `main.dart`: `SystemChrome.setPreferredOrientations([landscapeLeft,
  landscapeRight])` before `runApp` (+ `services.dart` import)
- [x] 1.2 `ios/Runner/Info.plist`: remove portrait entries from both
  `UISupportedInterfaceOrientations` and `~ipad` arrays
- [x] 1.3 `android/app/src/main/AndroidManifest.xml`: add
  `android:screenOrientation="sensorLandscape"` to `.MainActivity`

## 2. Pure auto-fit core

- [x] 2.1 Create `lib/painters/keyboard_range.dart`: `kPianoLowest=21`,
  `kPianoHighest=108`, `computeKeyboardRange(KeyboardRangeMode, List<TimedNote>)`,
  `presetKeyCount`, `extension on KeyboardRangeMode { String get label }`
- [x] 2.2 Implement auto (snap/min-span/clamp/coverage/empty-fallback) and preset
  (anchor/shift/widen/clamp/coverage) logic

## 3. State

- [x] 3.1 `lib/state/player_data.dart`: `enum KeyboardRangeMode {auto, keys25,
  keys37, keys49, keys61, keys76, keys88}`, field
  `@Default(KeyboardRangeMode.keys88) keyboardRange`, getter `keyboardBounds`
- [x] 3.2 `lib/state/player_notifier.dart`: `setKeyboardRange(mode)` mutator
- [x] 3.3 `dart run build_runner build --delete-conflicting-outputs`

## 4. Wiring & UI

- [x] 4.1 `lib/screens/player_screen.dart`: build the shared `PianoLayout` from
  `data.keyboardBounds`
- [x] 4.2 Add `_RangeChooser` `PopupMenuButton<KeyboardRangeMode>` to `_TopBar`
- [x] 4.3 Pass `requiredNotes: data.requiredNotesAt(data.elapsedMs)` to the
  keyboard painter

## 5. Keyboard feedback

- [x] 5.1 `lib/painters/piano_keyboard_painter.dart`: add `requiredNotes`;
  green/teal/purple precedence in `_drawKey`; halo for highlighted states;
  black-key outline/cap; extend `shouldRepaint`

## 6. Tests & checks (≥ 80%)

- [x] 6.1 `test/painters/keyboard_range_test.dart`: auto (empty/min-span/coverage/
  clamp) + presets (anchor/widen/clamp) + `label`/`presetKeyCount`
- [x] 6.2 Extend `test/player_notifier_test.dart`: `setKeyboardRange` +
  `keyboardBounds`
- [x] 6.3 Extend painter test: `shouldRepaint` on `requiredNotes`; pump painter
  hitting all three color branches; `golden`-tagged keyboard golden
- [x] 6.4 Extend `test/widgets/player_screen_test.dart`: chooser renders + tap
  updates `keyboardRange`
- [x] 6.5 `flutter test --coverage --exclude-tags golden` ≥ 80%; `melos run
  analyze`, `dart format`, `dart run custom_lint` clean
- [x] 6.6 `openspec validate dynamic-keyboard-display --strict` passes
