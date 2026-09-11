## 1. Dependency

- [x] 1.1 Add `wakelock_plus` to `apps/music/pubspec.yaml` with a comment saying what it is for and that it sits behind an injectable seam (the shape every other plugin dependency in that file uses), then `flutter pub get`.
- [x] 1.2 Confirm no new Android permission and no new Apple entitlement were pulled in: `AndroidManifest.xml` and the `.entitlements` files must be byte-identical after `pub get`. If the merged manifest gains `WAKE_LOCK`, stop — the plugin is taking a CPU wakelock and the design is wrong.

## 2. The seam (services)

- [x] 2.1 Create `apps/music/lib/services/screen_wake_service.dart`: an abstract `ScreenWakeService` with a single idempotent `Future<void> setEnabled({required bool enabled})`, documented as "the platform's keep-the-screen-on request, behind a seam so state and widget tests never load the plugin".
- [x] 2.2 Implement `WakelockPlusScreenWakeService` in the same file — one call to `WakelockPlus.enable()`/`disable()`, wrapped in try/catch with a `debugPrint` on failure (the `offline_key_provider.dart` shape). Keep it ~10 lines: it is the only untestable code in the change (design D6).
- [x] 2.3 Add the `@riverpod ScreenWakeService screenWakeService(Ref ref)` provider returning the production implementation.

## 3. The counted hold (state)

- [x] 3.1 Create `apps/music/lib/state/screen_wake.dart` with a Freezed `ScreenWakeState({@Default(0) int holders, @Default(true) bool foreground})` and a derived `bool get shouldHold => holders > 0 && foreground;`.
- [x] 3.2 Add the `@riverpod class ScreenWake extends _$ScreenWake` notifier with `acquire()`, `release()` and `setForeground(bool)`. `build()` returns the initial state without reading or assigning `state` (notifier rule).
- [x] 3.3 Call the service **only when `shouldHold` flips** between the previous and next state, so a stack of surfaces is one platform call. `release()` must clamp at zero rather than going negative.
- [x] 3.4 Run `dart run build_runner build --delete-conflicting-outputs`.

## 4. The wrapper widget

- [x] 4.1 Create `apps/music/lib/widgets/keep_screen_awake.dart`: `KeepScreenAwake({required Widget child})`, a `ConsumerStatefulWidget` that acquires in `initState` and releases in `dispose`.
- [x] 4.2 Capture the notifier in `initState` and use the captured reference in `dispose` — never `ref.read` during teardown (design D4).

## 5. Wiring

- [x] 5.1 Wrap the body of `PlayerScreen` in `KeepScreenAwake`.
- [x] 5.2 Wrap the body of `LessonPlayerScreen`.
- [x] 5.3 Wrap the body of `DrumCalibrationScreen` (a `ConsumerWidget` — the wrapper is why it stays one).
- [x] 5.4 Wrap the body of `MidiMonitorScreen`.
- [x] 5.5 Extend `_AudioLifecycleObserver` in `main.dart` to call `setForeground(false)` on `paused`/`hidden`/`detached` and `setForeground(true)` on `resumed`. Leave `inactive` untreated, for the same reason the audio path already leaves it untreated — a notification shade or a permission dialog must not dim the screen. Rename the observer to say what it now covers, and update its doc comment.

## 6. Tests

- [x] 6.1 Add the mockito mock for `ScreenWakeService` to the app's `@GenerateNiceMocks` list and regenerate (mocks are the default double — `flutter-testing` skill).
- [x] 6.2 Notifier tests: one acquire enables; a second acquire does **not** call the service again; releasing one of two keeps it enabled; releasing the last disables; an extra release does not go negative or emit a spurious call.
- [x] 6.3 Notifier tests for foreground: backgrounding with holders > 0 disables; resuming with holders > 0 re-enables; resuming with zero holders enables nothing.
- [x] 6.4 Widget test: mounting `KeepScreenAwake` acquires and popping it releases, with the service mocked — asserting no plugin is loaded on the Dart VM.
- [x] 6.5 Widget test per play surface: mounting each of the four screens ends with the hold taken. This is the test that catches a wrapper dropped in a later refactor of a screen's build method.
- [x] 6.6 Service test: a throwing platform call is swallowed and does not propagate (spec's "the platform refuses the request").
- [x] 6.7 Check the surviving play-surface widget tests still pass with the real (unoverridden) provider absent — any test mounting a play surface now needs the override, so add it to the shared test harness rather than to each test.

## 7. Verification

- [x] 7.1 `melos run analyze`, `dart format` from the repo root, and `dart run custom_lint` clean.
- [x] 7.2 `flutter test --coverage --exclude-tags golden` at ≥ 80%, and the merged lcov gate still passes with no change to the exclusion globs in `music-check.yml` (design D6). If it needs one, the adapter grew too big.
- [x] 7.3 `melos run integration` green — this is the Linux/Xvfb path where the Linux screensaver inhibitor has nothing to talk to, so it proves the swallow-errors rule (design D7).
      - Green (2/2) on macOS with the REAL plugin, so a real hold was taken and released around the player. Note the gate is `flutter test integration_test -d macos --exclude-tags capture` — the raw command without that exclusion also runs the store-capture scenario, which needs `flutter drive` and fails at launch. That failure is the invocation, not the code.
- [x] 7.4 Build all five targets, not just the two that reported the bug. On this machine `pod install` needs `LANG=en_US.UTF-8`, and `flutter test -d macos` rewrites the native shells — check `git status` on `ios/` and `macos/` afterwards.
      - Built locally: **Android** (debug APK), **iOS** (`--no-codesign`) and **macOS**. Both Apple `Podfile.lock`s and the macOS `GeneratedPluginRegistrant.swift` gained the plugin and nothing else; `ios/`, `macos/` and `android/` are otherwise untouched. **Linux and Windows are not buildable on this Mac** — left to CI. Low risk on both: the Linux implementation is pure-Dart D-Bus and Windows uses `win32`, already a transitive dependency of `file_picker` and `flutter_secure_storage`.

## 8. On-device validation

- [ ] 8.1 Android phone: open a score, play a passage on a connected MIDI keyboard for longer than the device's screen timeout without touching the screen — the screen stays lit.
- [ ] 8.2 Android phone: let Wait Mode hold at an onset past the screen timeout — the screen stays lit.
- [ ] 8.3 Same two checks on iOS.
- [ ] 8.4 Leave the player, sit on the library past the timeout — the screen sleeps normally.
- [ ] 8.5 Background the app from the player screen and leave the device alone — it sleeps on its normal schedule (this is the one the mobile platforms would fake for us, so verify it rather than assume it).
- [ ] 8.6 Desktop (macOS): open the player, minimise the window, and confirm the machine sleeps on its own schedule — the case that motivated the explicit foreground release (design D1).
- [ ] 8.7 Pull down the notification shade / trigger a permission dialog mid-session and dismiss it — the screen must not dim (the `inactive` exclusion, task 5.5).
- [ ] 8.8 Report back to the two users who raised it.
