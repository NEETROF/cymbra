## 1. Device-class helper (single source of truth)

- [ ] 1.1 Add `apps/music/lib/layout/device_class.dart` with a `DeviceClass { phone, tablet, desktop }` enum and a `deviceClassOf(BuildContext)` function + `context.deviceClass` extension reading `MediaQuery.sizeOf(context).shortestSide`.
- [ ] 1.2 Apply breakpoints (phone `< 600`, tablet `< 900`, else desktop; desktop/web platforms → desktop) with the thresholds defined as named constants for easy tuning.
- [ ] 1.3 Unit-test the helper at 812×375 (phone), 1024×768 (tablet), and a desktop size, asserting the resolved class for each.

## 2. Adaptive keyboard height

- [ ] 2.1 In `player_screen.dart`, remove the fixed `_keyboardHeight = 150` and compute the height inside the existing render `LayoutBuilder` from `constraints.maxHeight` (fraction), clamped to a legible min/max (start ~96–180 px).
- [ ] 2.2 Thread the computed height into the keyboard `SizedBox`, the `CustomPaint` size, and the `pitchAt` hit-test call so geometry stays consistent.
- [ ] 2.3 Verify the render area above the keyboard keeps a non-zero, usable height on the smallest supported phone viewport.

## 3. Adaptive top-bar chrome

- [ ] 3.1 Make `_TopBar` read `context.deviceClass` and select padding + title/subtitle font sizes from a small phone-vs-tablet/desktop token set.
- [ ] 3.2 Preserve the Material minimum (48 px) tap target for each icon button/chip even when visual padding is reduced on phones.
- [ ] 3.3 Confirm the trailing control cluster (back, MIDI status, tempo chip, settings) fits without horizontal overflow on the narrowest phone viewport.

## 4. Landscape lock verification (smartphone)

- [ ] 4.1 Confirm the Flutter runtime lock (`main.dart`) plus native iOS and Android orientation config exclude portrait on phones; adjust native config if any phone still allows portrait.

## 5. Tests & coverage

- [ ] 5.1 Add widget tests that set `tester.view.physicalSize`/`devicePixelRatio` to phone (812×375) and tablet (1024×768) landscape and assert: keyboard height is smaller on phone and within clamp bounds, render area height > 0, and `tester.takeException()` is null (no overflow).
- [ ] 5.2 Add a test that the smartphone landscape scenario keeps the UI in landscape (device class = phone, layout uses the phone branch).
- [ ] 5.3 Run `cd apps/music && dart run build_runner build --delete-conflicting-outputs`, then `flutter test --coverage --exclude-tags golden`; keep Flutter line coverage ≥ 80%.
- [ ] 5.4 Refresh golden tests on the pinned platform if chrome/keyboard dimensions shifted (`flutter test --tags golden --update-goldens`).

## 6. Pre-PR checks

- [ ] 6.1 `melos run analyze` and `dart format` clean; `dart run custom_lint` passes.
- [ ] 6.2 `openspec validate support-smartphone-landscape --strict` passes.
