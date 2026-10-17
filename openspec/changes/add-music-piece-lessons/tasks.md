# Tasks

## 1. Format and content

- [x] 1.1 Add `PieceBlock` to `courses/course_manifest.dart` (score reference, hands, speed, waitMode, metronome, readingAid, passPct default 80, prompt) with a defensive parser; verify with parser tests for a full block, missing optional keys, a bad score reference and the forward-compatibility test still skipping unknown types.
- [x] 1.2 Document the block in `backend/content/courses/README.md` with the staged-chain convention and the "last step free-run at full tempo" rule; verify the README renders the example and the corpus test in 1.3 cites it.
- [x] 1.3 Extend `apps/music/test/courses/content_corpus_test.dart` to validate piece blocks against the unit's taught set (note range, figures, accidentals) and the closing-step rule; verify it fails on a deliberately mis-filed fixture and passes on the corpus.
- [x] 1.4 Author the seven closing piece lessons (one per unit) over bundled scores — four short traditional tunes added to the bundle for the early units, Ode to Joy and the Minuet in G for the last two — and regenerate `seed_courses.sql` with `gen_seed_courses.py`; verify the corpus test passes and the seed script regenerates 49 courses.

## 2. Backend progress record

- [ ] 2.1 Migration `music.course_progress` + `passed_at timestamptz` + `best_sync_pct real`; verify the migration applies on a fresh and an existing DB in the Rust integration test.
- [x] 2.2 Add the `RecordCoursePieceResult` RPC (course id, passed, sync %) and `passed_at_ms` / `best_sync_pct` on `CourseProgress` (additive only); verify `buf breaking` against main passes and the generated Dart/Rust compile.
- [x] 2.3 Implement first-pass-wins `passed_at` and greatest-wins `best_sync_pct` in `course_progress.rs` behind the existing trait, with a mockall double for the RPC test; verify unit tests for first pass, later lower run, later higher run and a course without piece steps.
- [x] 2.4 Extend the worker's `purge_user` coverage test to the new columns; verify erasure leaves no row.

## 3. Player lesson context

- [x] 3.1 Add a `LessonContext` (imposed settings, lesson title, step target, passPct) to `PlayerNotifier` run seeding, read in one place instead of `PlayerPreferences` when present; verify notifier tests for every imposed setting and for preferences left untouched.
- [x] 3.2 Skip the pre-play modal on entry in lesson context and show it read-only ("set by the lesson") from the gear; verify widget tests for both.
- [x] 3.3 Show the lesson title and target in the player header in lesson context; verify a widget test.
- [x] 3.4 Add the `backToLesson` summary action as primary in lesson context, keep retry in context; verify summary widget tests for action order and for retry re-seeding from the context.
- [x] 3.5 Add `openScoreForLesson(context, ref, ref: scoreRef, lessonContext)` beside `openScore`, resolving bundled and catalogue references and returning the last `SessionResult`; verify tests for a resolvable piece, a missing catalogue piece and the offline-uncached state.

## 4. Lesson side

- [x] 4.1 Build `widgets/piece_block_view.dart`: piece title, imposed settings summary, target, Start / Try again / Next, passed-visited state from the returned result; verify widget tests for not started, visited, passed, missing piece and offline.
- [x] 4.2 Gate the step in `LessonPlayerScreen` like other exercises (Next enabled after one attempt, skip hatch after the delay) and count passed steps as flawless in the celebration stat; verify lesson player tests.
- [x] 4.3 Extend `CourseCompletion` state and sync with passed + best (local first, server merge first-pass-wins / greatest-wins), through the `CourseProgressService` seam with a mockito mock; verify notifier tests for guest, signed-in, reconnect and regression-never cases.
- [x] 4.4 Show the three node states on the path and point the continue card to the first course neither completed nor passed; verify path widget tests.
- [x] 4.5 Add the four-locale strings; verify ARB alignment and `dart run custom_lint`.

## 5. Integration

- [x] 5.1 Integration test: open unit 1's closing lesson, run the right-hand step with the fixture score, return to the lesson, see it visited or passed, relaunch and see the state kept; verify `melos run integration` passes.
- [x] 5.2 Coverage gates ≥ 80 % on both ecosystems; verify `cargo llvm-cov` and the Flutter lcov check pass.
