# Tasks

## 1. Format and content

- [x] 1.1 Keep an optional `placementUnit` tag on every parsed block in `course_manifest.dart`; verify parser tests for tagged, untagged and unknown-type blocks. *(Kept as `CourseManifest.placementUnits`, a map by block index — see D1.)*
- [x] 1.2 Author `backend/content/courses/placement-piano.json` (track `placement`, five to seven exercises in unit order plus one closing piece step) and regenerate the seed script; verify the extended corpus test asserts tags cover the units in order and the seed re-runs idempotently. *(Seven exercises, one per unit, plus the closing piece; the corpus test has its own placement group.)*
- [x] 1.3 Exclude the `placement` track from the path listing and expose it through a `placementCourseProvider`; verify provider tests with and without a placement course in the listing.

## 2. Result and state

- [x] 2.1 Write the pure result computation (first miss, skip as miss, all solved, piece passed) in `state/placement_core.dart`; verify unit tests for each pattern and for a run without a piece step.
- [x] 2.2 Add `state/placement_notifier.dart` (`startUnit`, `takenAt`, persisted locally) with retake and clear; verify notifier tests for restore, replace, clear.

## 3. Lesson player and path

- [x] 3.1 Run the placement course in the lesson player with a result screen (proposed unit, adjust one up or down, confirm, start from the beginning); verify widget tests for proposal, adjustment and confirmation writing the notifier.
- [x] 3.2 Render units before the starting unit as collapsed "skipped by placement — open" sections keeping their real course states; verify path widget tests.
- [x] 3.3 Make the continue card and the daily-session composition start from the starting unit; verify tests in both modules. *(`skippedUnitsProvider`, consumed by the path, the continue card and the daily session.)*
- [x] 3.4 Add `widgets/placement_offer_card.dart` with its timing rule and the path-header entry; verify widget tests for first visit, after a completed lesson, after a placement.
- [x] 3.5 Add the four-locale strings; verify ARB alignment and `dart run custom_lint`.

## 4. Integration

- [x] 4.1 Integration test: take the placement with scripted misses on unit 3, confirm, see units 1 and 2 collapsed and the continue card on unit 3, clear and see the full path; verify `melos run integration` passes.
