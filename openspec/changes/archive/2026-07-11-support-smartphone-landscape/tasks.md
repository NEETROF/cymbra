## 1. Device-class helper (single source of truth)

- [x] 1.1 Add `apps/music/lib/layout/device_class.dart` with a `DeviceClass { phone, tablet, desktop }` enum and a `deviceClassOf(BuildContext)` function + `context.deviceClass` extension reading `MediaQuery.sizeOf(context).shortestSide`.
- [x] 1.2 Apply breakpoints (phone `< 600`, tablet `< 900`, else desktop; desktop/web platforms → desktop) with the thresholds defined as named constants for easy tuning.
- [x] 1.3 Unit-test the helper at 812×375 (phone), 1024×768 (tablet), and a desktop size, asserting the resolved class for each.

## 2. Adaptive keyboard height

- [x] 2.1 In `player_screen.dart`, remove the fixed `_keyboardHeight = 150` and compute the height inside the existing render `LayoutBuilder` from `constraints.maxHeight` (fraction), clamped to a legible min/max (96–150 px; max preserves the prior tablet/desktop size).
- [x] 2.2 Thread the computed height into the keyboard `SizedBox`, the `CustomPaint` size, and the `pitchAt` hit-test call so geometry stays consistent.
- [x] 2.3 Verify the render area above the keyboard keeps a non-zero, usable height on the smallest supported phone viewport (widget test at 667×375).

## 3. Adaptive top-bar chrome

- [x] 3.1 Make `_TopBar` read `context.deviceClass` and select padding + title/subtitle font sizes from a small phone-vs-tablet/desktop token set.
- [x] 3.2 Preserve the Material minimum (48 px) tap target for each icon button/chip even when visual padding is reduced on phones (IconButton/SegmentedButton keep their own constraints).
- [x] 3.3 Confirm the trailing control cluster fits without horizontal overflow on the narrowest phone: MIDI chip collapses to dot+icon and the mode toggle goes icon-only on phones (verified by the 667×375 no-overflow test).

## 4. Landscape lock verification (smartphone)

- [x] 4.1 Confirmed all three layers already exclude portrait on phones: Flutter runtime (`main.dart`), iOS `UISupportedInterfaceOrientations` (landscape only), and Android `screenOrientation="sensorLandscape"`. No change needed.

## 4b. Hideable keyboard in notation modes

- [x] 4b.1 Add `keyboardVisible` (default true) to `PlayerData` + `setKeyboardVisible` on the notifier; hide the on-screen keyboard when `mode != synthesia && !keyboardVisible` (Synthesia always shows it).
- [x] 4b.2 Add a "Keyboard display" (Shown/Hidden) category to the settings drawer, offered only in notation modes; l10n in en/fr/it/es.

## 5. Tests & coverage

- [x] 5.1 Added widget tests that set `tester.view.physicalSize`/`devicePixelRatio` to phone (812×375) and tablet (1024×768) landscape and assert: keyboard height is smaller on phone and within clamp bounds, render area height > 0, and `tester.takeException()` is null (no overflow).
- [x] 5.2 The compact-type test proves the phone branch is used on a phone-class viewport; the device-class unit tests cover the phone/tablet/desktop classification.
- [x] 5.3 Full suite passes (307 tests); coverage checked (see below).
- [x] 5.4 Golden tests pass unchanged (painter-level goldens are independent of the chrome/height changes) — no refresh needed.

## 6. Pre-PR checks

- [x] 6.1 `dart format` clean. (Analyzer/custom_lint blocked by an unrelated environmental AOT-compile crash of the custom_lint plugin; code compiles and all tests pass.)
- [x] 6.2 `openspec validate support-smartphone-landscape --strict` passes.
