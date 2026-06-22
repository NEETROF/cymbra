## Why

The on-screen keyboard is hard-coded to C4–C6 (15 white keys) and gives a single
pressed-key color. Learners need to see the right slice of the keyboard for the
piece they are playing, choose a larger layout (up to a full 88-key P45), and get
clear "press this key" vs "you played it correctly" feedback. A full 88-key view
is only legible if the app stays in landscape, so orientation must be locked.

## What Changes

- **Force landscape** app-wide (Flutter + iOS + Android); portrait is disabled.
- Add a **runtime keyboard-range chooser**: defaults to the full **88-key**
  piano, with **auto-fit** to the loaded piece and presets (25/37/49/61/76 keys)
  selectable at runtime.
- Add **three-state color feedback** on the keyboard: a "press this key" color
  (teal) for notes required now, a "correct" color (green) when the required key
  is held, and the existing purple for any other pressed key.
- Decouple the keyboard/Synthesia range from the hard-coded `lowPitch=60/highPitch=84`
  by computing the range from state and feeding the single shared `PianoLayout`.

## Capabilities

### New Capabilities
- `keyboard-display`: landscape-locked presentation, a runtime keyboard-range
  chooser (auto-fit to the piece + fixed presets), and three-state visual
  feedback (expected / correct / pressed) on the on-screen keyboard.

### Modified Capabilities
<!-- None: the time-based `score` and `midi` capabilities are unchanged. -->

## Impact

- **Dart**: new `apps/music/lib/painters/keyboard_range.dart` (pure auto-fit +
  presets); `lib/state/player_data.dart` (`KeyboardRangeMode` enum + field +
  `keyboardBounds` getter), `lib/state/player_notifier.dart` (`setKeyboardRange`);
  `lib/painters/piano_keyboard_painter.dart` (`requiredNotes` + colors);
  `lib/screens/player_screen.dart` (range chooser, layout wiring, required-notes);
  `lib/main.dart` (orientation). Generated `*.g.dart`/`*.freezed.dart` via
  `build_runner`.
- **Native**: `apps/music/ios/Runner/Info.plist` and
  `apps/music/android/app/src/main/AndroidManifest.xml` (landscape only).
- **Tests/CI**: new pure unit tests for `keyboard_range`, notifier/state tests,
  painter widget + golden, screen chooser test — line coverage stays ≥ 80%.
- **Users**: app opens locked to landscape; a keyboard-size chooser appears; the
  required key is highlighted and turns green when played correctly.
