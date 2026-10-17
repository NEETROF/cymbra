# Tasks

## 1. Motion system

- [x] 1.1 Add `theme/motion.dart` (quick/standard/celebrate durations, curves) and `Motion.of(context, duration)` reading `MediaQuery.disableAnimations`; verify unit tests that scaled durations are zero under reduced motion.
- [x] 1.2 Adopt the tokens in the existing countdown, path pulse, door switch and Hero transitions where the literal already matches; verify the existing widget tests still pass.

## 2. Celebration surface

- [x] 2.1 Build `painters/confetti_painter.dart` with a fixed seeded pool in a `Float32List` and one controller; verify a golden at three progress values and a test that the particle count equals the constant. *(Golden: `test/screenshots/motion_shots_test.dart`, three progress values; count: `test/painters/confetti_painter_test.dart`.)*
- [x] 2.2 Build `widgets/celebration_surface.dart` (icon scale-in, burst, title, optional counter, actions enabled at the end); verify widget tests pumping the sequence and the reduced-motion final frame.
- [x] 2.3 Move the lesson celebration, reward celebration, level-up, daily-session complete and streak milestone onto the surface, keeping their keys, strings and actions; verify their existing widget tests pass unchanged plus one test per moment for the animated path. *(Lesson, reward, level-up and streak milestone are on the surface; the daily-session complete moment lands with `add-music-daily-session`, which builds it on the surface directly.)*
- [x] 2.4 Add `services/haptics_service.dart` behind `hapticsProvider` and call it from the surface on pass and completion only; verify mock-based tests for pass, completion, miss and reduced haptics. *(Called by the surface — every caller is a pass or a completion; a miss raises no surface. Haptic on/off: `celebration_surface_test.dart`; miss: `lesson_piece_step_test.dart`.)*

## 3. Counters

- [x] 3.1 Build `widgets/animated_counter.dart` tweening an int through the localized formatter, animating increases only; verify widget tests for increase, decrease and reduced motion.
- [x] 3.2 Wrap the standing pill value, the summary "+N" cue and the daily ring in it; verify widget tests that the pill counts up after a reward revision and shows a debit at once. *(Pill and "+N" cue done; the daily ring lands with `add-music-daily-session`.)*

## 4. Drawn path

- [x] 4.1 Build `painters/path_curve_painter.dart` (cubic through node centres, filled to last completed, dashed beyond); verify goldens for an empty, partial and complete unit. *(Goldens in `motion_shots_test.dart`: empty, partial, complete.)*
- [x] 4.2 Animate the check and the segment fill on completion, and the one-per-session unit header light, driven by a listener widget; verify widget tests pumping a completion and a rebuild without replay. *(Driven by the unit section itself — the completion is a state change the section already rebuilds on; the once-per-session memory is the `litUnitsProvider` keepAlive provider.)*

## 5. Integration

- [ ] 5.1 Measure the confetti and path animations on the reference low-end Android device with the performance overlay; verify no frame above 16 ms across three runs and record the figures in the PR. *(Needs the device in hand — not possible from the development machine; left for the dogfood pass.)*
- [x] 5.2 Refresh goldens for the celebration surface and the path on the pinned platform; verify the golden tests pass.
