> Sequenced after `add-notation-help` (shares its notation painters/glyphs for diagrams). 2a is
> backend-free and lands first; 2b adds the backend (cross-device completion + badge). 2c
> (community-authored courses) is out of scope for this change.

## 1. Course manifest format + bundled course (2a)

- [ ] 1.1 Define the `CourseManifest` model (Freezed): `schemaVersion`, `id`, inline-localized `title`/`subtitle` (`{en,fr,es,it}`), `difficulty`, ordered `steps` where a step is `Explanation { text i18n, diagram? }` | `Diagram { id }` | `Quiz { prompt i18n, options i18n[], answerIndex, feedback i18n }`
- [ ] 1.2 Add a defensive JSON parser: unknown/greater `schemaVersion` → skip the course (log), never crash; validate step shape; a helper to resolve an inline-i18n field for the current locale (fallback to `en`)
- [ ] 1.3 Add an asset loader for `assets/courses/**` (declare the dir in `pubspec.yaml`) exposed via a Riverpod provider (course list), with the native/asset access behind an injectable seam for tests
- [ ] 1.4 Author at least one bundled first-party course manifest ("Reading the staff": staff/lines, note names, treble vs bass clef, accidentals, note/rest durations) in all four languages
- [ ] 1.5 Define the closed set of built-in **diagram ids** and a `CourseDiagram` widget that renders each via the existing notation painters/SMuFL glyphs (reused from `add-notation-help`)
- [ ] 1.6 Unit-test parsing (valid, unknown schema skipped, i18n fallback) and diagram-id resolution

## 2. Home "Cours" section + tiles (2a)

- [ ] 2.1 Add a `_CoursesSection` above `_FavoritesBody` in `library_screen.dart` (reachable with no menu; never blocks favorites; omits gracefully when there are no courses)
- [ ] 2.2 Add a course **tile** widget showing the course + a **completion indicator**; tapping opens the lesson player
- [ ] 2.3 Add ARB keys for the section title + tile/player chrome (en/fr/es/it) — UI chrome only (course content stays manifest-inline)
- [ ] 2.4 Widget-test: section renders above favorites, a completed course's tile shows the indicator, tapping opens the player

## 3. Lesson player (2a)

- [ ] 3.1 Add a `LessonPlayerScreen` stepping through a course's steps at the user's pace (explanation / diagram / quiz), skippable, leaveable at any time
- [ ] 3.2 Quiz step: immediate feedback on answer, never blocks continuing; accessible controls (no single-gesture dependency)
- [ ] 3.3 On reaching the end, mark the course completed (see §4); allow **infinite replay** (reopening a completed course replays it, completion stays)
- [ ] 3.4 Widget-test the flow: step-through, skip, quiz feedback non-blocking, completion on finish, replay

## 4. Completion state — local first (2a)

- [ ] 4.1 Add a Riverpod completion notifier holding a `courseId → completed` map, backed locally by `shared_preferences` (guest path), with `markCompleted(courseId)` and recon*ready* for the server source (§6)
- [ ] 4.2 Wire the home tiles + player to the completion notifier; unit-test local completion persistence + replay leaves it set

## 5. Backend course-progress store + gRPC (2b)

- [ ] 5.1 Migration: a per-user `course_progress` table (`user_id`, `course_id`, `completed_at`, `play_count`), cascading on account erasure; renumber to the next free migration index
- [ ] 5.2 Add a `CourseProgressStore` trait + Postgres impl in `cymbra-music`; keep an idempotent, host-testable **award core** (first completion → award badge + set `completed_at`; replay → bump `play_count` only)
- [ ] 5.3 Add gRPC `RecordCourseCompletion(courseId)` and `GetCourseProgress()` on `ScoreService` (proto + `melos gen-grpc`); wire the store; music-scope auth as the other user data
- [ ] 5.4 Rust tests (mockall store double) for the award core: first-completion awards once, replay bumps count and never re-awards; `cargo llvm-cov` ≥ 80% on the core

## 6. Cross-device sync + guest→account (2b)

- [ ] 6.1 Add the gRPC client seam in Flutter; on course-list load (signed in) fetch `GetCourseProgress` and reconcile into the completion notifier; reading never blocks opening a course
- [ ] 6.2 On completion (signed in) call `RecordCourseCompletion`; on sign-in, best-effort push any local guest completions (idempotent)
- [ ] 6.3 Widget/unit-test: signed-in completion shows across a fresh container (server source), guest stays local, load-in-flight still opens the course

## 7. Completion badge (2b)

- [ ] 7.1 Add the course-completion badge to `curation-rewards`; award it in the store's award core on first completion (once per course), surfaced through the existing rewards/badge feedback
- [ ] 7.2 Test: first completion awards the badge once; replay does not re-award

## 8. Supersede the old sketch + quality gate

- [ ] 8.1 Remove the superseded `notation-lessons` capability from `add-notation-help` (delete its delta spec, drop it from that proposal/tasks, note it moved here); re-validate `add-notation-help --strict`
- [ ] 8.2 `flutter analyze` + `dart format` + `dart run custom_lint` clean; `cargo fmt --check` + `cargo clippy -D warnings` clean
- [ ] 8.3 Coverage ≥ 80% both ecosystems (Flutter manifest/section/player/sync; Rust award core)
- [ ] 8.4 `openspec validate add-notation-courses --strict` passes
