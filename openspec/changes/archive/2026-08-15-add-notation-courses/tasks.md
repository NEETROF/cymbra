> Sequenced after `add-notation-help` (shares its notation painters/glyphs for `diagram`/`score`).
> Courses are server-stored from day one; the client owns the block format + forward-compat. Media
> hosting and community authoring (2c) are out of scope for this change.

## 1. Backend: course storage + delivery

- [x] 1.1 Migration `0019_courses.sql`: `music.courses` (`id`, `status`, `instrument`, `track`, `level`, `sort_order`, `schema_version`, `title JSONB` inline-i18n, `content JSONB` opaque manifest, timestamps) + `courses_listing_idx`
- [x] 1.2 `CourseRepo` trait + `PgCourseRepo` + hand `FakeCourseRepo` in `cymbra-music` (`course.rs`): `list_published` (summaries, grouped/ordered), `get` (full manifest), `upsert` (seed). Validation is the `content jsonb` cast (rejects malformed JSON); content otherwise opaque
- [x] 1.3 gRPC `ListCourses` / `GetCourse` on `ScoreService` (proto + build.rs regen), returning summaries + the manifest blob; authenticated (`identity`), wired in `server/src/main.rs` via `.with_courses(...)`
- [x] 1.4 Seed script for first-party courses — delivered by §6/§10: `backend/scripts/seed_courses.sql`, generated from the 42 authored manifests by `gen_seed_courses.py`, idempotent over `upsert`
- [x] 1.5 Rust unit tests for list/get/upsert (grouping, published filter, idempotent re-seed) — 3 tests green, clippy + fmt clean; workspace `llvm-cov` gate runs at §7.3

## 2. Client: manifest model + forward-compatible block engine

- [x] 2.1 Freezed `CourseManifest` (`schemaVersion`, `id`, `instrument`, `track`, `level`, inline-i18n `title`/`summary`, `blocks[]`) and a **Block** union: `text`, `diagram`, `image`, `video`, `question`, `playKey`, `score`, plus an `unsupported` fallback (`lib/courses/course_manifest.dart`)
- [x] 2.2 Defensive parser: unknown block `type` → `unsupported` (skipped at play time), malformed known block → `unsupported` (never throws), unknown/absent top-level `schemaVersion` → course declined (null); inline-i18n resolver (`resolveInline`, current locale → `en` → any)
- [x] 2.3 Course-source seam (`CourseCatalogService` + `GrpcCourseCatalogService` over `ListCourses`/`GetCourse`, `lib/services/course_catalog_service.dart`) + `coursesProvider`/`courseManifestProvider` reconciled with a `shared_preferences` offline cache (server → refresh cache; unreachable → serve cache); tested (fetch+cache, offline fallback, unknown course)
- [x] 2.4 Unit-tested parsing (`test/courses/course_manifest_test.dart`): valid manifest → ordered typed blocks; **injected unknown block still yields a completable course**; malformed-block degradation; schema-version decline; i18n fallback. (Cache round-trip lands with §2.3.)

## 3. Home "Cours" section + lesson player (display + quiz blocks)

- [x] 3.1 `CoursesSection` above `_FavoritesBody` in `library_screen.dart` (compact horizontal row, no menu; never blocks favorites; omits when empty), ordered by track/level + course **tile** with completion indicator (`lib/widgets/courses_section.dart`)
- [x] 3.2 `LessonPlayerScreen` runs blocks at the user's pace; renders `text`, `diagram` (`CourseDiagram` via SMuFL), `image`/`video` (from URL, degrading to a caption card); Next/Back + progress; leaveable anytime; unsupported blocks skipped; `playKey`/`score` placeholders until §4
- [x] 3.3 `question` block: multiple-choice with immediate correct/incorrect feedback, Next never disabled; keyed option buttons
- [x] 3.4 ARB keys for the section title + player/tile chrome added in en/fr/es/it (content stays manifest-inline)
- [x] 3.5 Widget-tested: section renders above favorites + omits when empty + completion indicator + tile opens the player (`courses_section_test.dart`); player step-through, unsupported-block skip, quiz non-blocking feedback, finish→completed (`lesson_player_screen_test.dart`) + completion notifier unit tests

## 4. Interactive blocks: playKey + score

- [x] 4.1 `playKey` block (`lib/widgets/play_key_view.dart`): prompt + mini on-screen keyboard (`PianoLayout`/`PianoKeyboardPainter`) + MIDI listener (`midiService.events()`); playing the target note(s) via tap **or** MIDI advances (`onSatisfied`); Next is always a non-blocking skip; the target keys stay highlighted as the standing hint
- [x] 4.2 `score` block (`lib/widgets/score_block_view.dart`): parse inline MusicXML via the `notationEngineProvider` seam → `ScoreDocument`, engrave via `PartitionPainter` (bounded, scrollable). Static render done; the `playable` **embedded performance** (gate = performed) is a documented follow-up
- [x] 4.3 Widget-tested with faked seams: `playKey` — keyboard shown, MIDI note advances, wrong note doesn't, Next skips (`play_key_view_test.dart`); `score` — excerpt engraves via a fake notation engine, skippable (`score_block_view_test.dart`)

## 5. Cross-device completion + badge

- [x] 5.1 Migration `0020_course_progress.sql`: `music.course_progress` (`user_id` UUID, `course_id`, `completed_at`, `play_count`, PK(user_id,course_id)); erasure via the worker `purge_user` DELETE loop (no cross-schema FK)
- [x] 5.2 `CourseProgressStore` trait + `PgCourseProgressStore` (idempotent upsert: first completion sets `completed_at` → `newly_completed`, replay bumps `play_count`) + `FakeCourseProgressStore` (`course_progress.rs`)
- [x] 5.3 gRPC `RecordCourseCompletion(courseId)` (returns `newly_completed`) / `GetCourseProgress()` on `ScoreService`, owner-scoped; wired in `server/src/main.rs` via `.with_course_progress(...)`
- [x] 5.4 Course-completion badge — **landed in `achievement-badges`** instead of `curation-rewards` (which knows only curator metrics): family `learning`, tiers `student_1/2/3` at 1/10/42 completions (`badges_core.rs`), counted off `music.course_progress.completed_at` in `pg_badges.rs`. This change's `completed_at`/`newly_completed` were the hook it consumes.
- [x] 5.5 `CourseProgressService` seam (`RecordCourseCompletion`/`GetCourseProgress`); the completion notifier reconciles with the server via `ref.listen(canUseOnlineServicesProvider)` — merges the server set, records on finish, and pushes local-only completions on sign-in (guest→account); all best-effort (offline/guest → local only)
- [x] 5.6 Rust tests (first completion `newly_completed`, replay only counts, per-user isolation) + Flutter tests (`course_completion_sync_test.dart`: server merge, guest stays local, record-on-finish, guest→account push). clippy/fmt/analyze/custom_lint clean.

## 6. First-wave course content (seed)

- [x] 6.1 Authored a first wave of **5 courses** fully in `{en,fr,es,it}` (`backend/scripts/seed_courses.sql`): Track A beginner `sol-reading-staff`/`sol-note-names`/`sol-note-values` (text/diagram/question/playKey) + Track B `app-synthesia`/`app-partition`, each `instrument:"piano"` with `track`/`level`. The remaining catalogue is backlog (6.3)
- [x] 6.2 Seed script `backend/scripts/seed_courses.sql` (idempotent upsert; replaces the demo). All 10 JSON blobs validated; every block type used (text/diagram/question/playKey) is covered by the client parser + player tests
- [x] 6.3 Remaining catalogue left as a data-only **backlog** (`catalogue.md`) — a new course is another row in the seed, no app release

## 7. Supersede the old sketch + quality gate

- [x] 7.1 `notation-lessons` already removed from `add-notation-help` when courses were spun out (merged in #193); its spec folder is gone
- [x] 7.2 `flutter analyze` + `dart format` + `dart run custom_lint` clean; `cargo fmt --all --check` + `cargo clippy -D warnings` (music/server/worker) clean
- [x] 7.3 Flutter: full suite green (1111 tests); new courses code **83.6%** aggregate (diagram/section 100%, notifier/score 97%, manifest 89%, player 81%; only the gRPC-glue service impls lag, like the excluded Rust Pg glue). Rust: cores fake-tested; Pg glue excluded per convention. CI enforces the workspace gates.
- [x] 7.4 `openspec validate add-notation-courses --strict` passes

## 8. Schema v2 — interactive solfège engine (revoie: courses must teach solfège interactively)

- [x] 8.1 Pure modules: `lesson_pitch.dart` (SPN spelling ↔ MIDI ↔ staff step ↔ clef, `keySignatureAlter`) + `lesson_rhythm.dart` (figure model, onsets at tempo, `gradeRhythmTaps`) — host-tested
- [x] 8.2 Manifest v2 (`kCourseSchemaVersion=2`): new blocks `staff`, `readPlay` (drill/melody/set), `nameNote`, `placeNote`, `rhythmTap`, `earChoice`, `buildChord`; defensive parser cases (malformed → unsupported, one bad pitch declines the block) + tests
- [x] 8.3 `LessonStaff` teaching-staff widget (SMuFL clef/armure/meter/notes/rests/ledger, per-element colours, ghost note, tap→staff-step, armure-aware accidental suppression) + `LessonSounder` (every touch sounds; audition-widget pattern) + `PianoKeyboardPainter.selectedNotes` state
- [x] 8.4 Exercise views + widget tests (RecordingAudioService/FakeMidiService): `ReadPlayView`, `NameNoteView`, `PlaceNoteView`, `RhythmTapView` (ticker + `metronomeBeatsCrossed`), `EarChoiceView`, `BuildChordView`; legacy `playKey` now sounds on-screen taps
- [x] 8.5 Lesson player: interactive blocks **gate Next** (12s discreet skip escape), run first-try stats, end-of-lesson celebration (lilac accent, stat, one-tap **Continuer → next lesson**); tests updated
- [x] 8.6 `coursesProvider` filters listings above the supported schemaVersion (no dead tiles on older apps)

## 9. Units + learning path

- [x] 9.1 Migration `0021_course_units.sql` (`unit` text + `unit_title` JSONB) + `course.rs` (SUMMARY_COLS/row_to_summary/upsert/Fake + round-trip test) + proto `CourseSummary` fields 8/9 + BOTH grpc.rs mapping sites; cargo fmt/clippy/test green
- [x] 9.2 `melos run gen-grpc`; `CourseListing.unit/unitTitle` + `_listingOf` + tolerant offline-cache codec
- [x] 9.3 `LearningPathScreen` (units in catalogue order, progress bars, meandering nodes, pulsing next-up, soft ordering — every node tappable) + `CoursesSection` → continue card + path entry; widget tests

## 10. Solfège corpus — 42 courses, 7 units × 6 lessons

- [x] 10.1 Corpus home `backend/content/courses/` (one JSON per course + authoring README); app-usage courses dropped — the curriculum is solfège only
- [x] 10.2 Corpus gate `test/courses/content_corpus_test.dart`: every file parsed by the REAL `parseCourseManifest` (zero unsupported blocks), 4 locales on every i18n map, id/unit/sortOrder/level coherence, ≥4 interactive (≥2 v2) per lesson, rhythm patterns fill exactly 1–2 bars, earChoice up/down consistent with pitches
- [x] 10.3 Generator `backend/scripts/gen_seed_courses.py` → idempotent `seed_courses.sql` (dollar-quoted, retires the 5 pre-curriculum rows)
- [x] 10.4 42 lessons authored in en/fr/es/it (U1 portée/premières notes → U7 intervalles/accords/oreille), reviewed by a musical + an editorial pass (2 minor findings, fixed)
- [x] 10.5 Manual: live DB seeded (`psql -f backend/scripts/seed_courses.sql`) + on-device pass of one lesson per unit — validated 2026-08-15
