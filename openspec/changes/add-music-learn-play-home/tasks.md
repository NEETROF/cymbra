# Tasks

## 1. Door state and flag

- [x] 1.1 Add `state/home_door.dart`: a Riverpod notifier with a Freezed state (`play` | `learn`), restored from and persisted through `preferencesServiceProvider`, default Play; verify with notifier tests covering restore, select and persistence (mockito preferences mock).
- [x] 1.2 Declare `music.learn_door.enabled` beside `drums_access.dart` with an `or: true` default and a `learnDoorEnabledProvider`; verify with a unit test that a missing flag reads on and an explicit off reads off.
- [x] 1.3 Register the flag key in the flags back-office seed/list so it can be flipped in production; verify it appears in the admin flags console fixture test.

## 2. Header switcher

- [x] 2.1 Add `widgets/home_door_switcher.dart` (SegmentedButton, Play / Learn, icon-only under 600 px) wired to the notifier; verify with widget tests for selection, compact mode and the drums rule (Play only when the instrument context is drums).
- [x] 2.2 Mount it in the home app bar's bottom slot beside the instrument switcher, absent when the flag is off; verify with widget tests for both switchers visible, instrument switcher alone, and flag-off.
- [x] 2.3 Add the four-locale strings (door labels, "Continue learning" chip); verify `melos run analyze` and the ARB alignment test pass.

## 3. Door bodies

- [x] 3.1 Extract the learning path's unit sections and lesson nodes from `learning_path_screen.dart` into a reusable `widgets/learning_path_body.dart` used by both the pushed screen and the Learn door; verify the existing path widget tests still pass unchanged.
- [x] 3.2 Build the Learn door body: reserved daily-session slot (empty widget), continue card, path body; verify with widget tests for the guest case and the empty-courses case.
- [x] 3.3 Build the Play door body: today's slivers minus the courses card, plus the "Continue learning" header chip when a lesson is in progress; verify with widget tests that no courses card precedes the favorites and the chip switches the door.
- [x] 3.4 Keep the single-home fallback (courses card above favorites) when the flag is off; verify with a widget test under the flag-off override.

## 4. Transitions

- [x] 4.1 Swap door bodies through an `AnimatedSwitcher` (fade + 24 px slide, 280 ms, easeOutCubic) that honours `MediaQuery.disableAnimations`; verify with widget tests pumping frames with and without reduced motion.
- [x] 4.2 Wrap the continue card, lesson nodes and score cards' title rows in `Hero` tags matched in the lesson player, path screen and player headers; verify with widget tests that the Hero flight completes and the destination renders.

## 5. Integration

- [x] 5.1 Refresh the home goldens on the pinned platform (`flutter test --tags golden --update-goldens`) for Play, Learn, drums and flag-off; verify the golden tests pass.
- [x] 5.2 Run the integration test on the real app: switch doors, open a lesson from Learn, open a score from Play, relaunch and land on the remembered door; verify `melos run integration` passes.
