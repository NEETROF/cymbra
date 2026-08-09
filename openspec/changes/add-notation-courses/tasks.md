> Sequenced after `add-notation-help` (shares its notation painters/glyphs for `diagram`/`score`).
> Courses are server-stored from day one; the client owns the block format + forward-compat. Media
> hosting and community authoring (2c) are out of scope for this change.

## 1. Backend: course storage + delivery

- [x] 1.1 Migration `0019_courses.sql`: `music.courses` (`id`, `status`, `instrument`, `track`, `level`, `sort_order`, `schema_version`, `title JSONB` inline-i18n, `content JSONB` opaque manifest, timestamps) + `courses_listing_idx`
- [x] 1.2 `CourseRepo` trait + `PgCourseRepo` + hand `FakeCourseRepo` in `cymbra-music` (`course.rs`): `list_published` (summaries, grouped/ordered), `get` (full manifest), `upsert` (seed). Validation is the `content jsonb` cast (rejects malformed JSON); content otherwise opaque
- [x] 1.3 gRPC `ListCourses` / `GetCourse` on `ScoreService` (proto + build.rs regen), returning summaries + the manifest blob; authenticated (`identity`), wired in `server/src/main.rs` via `.with_courses(...)`
- [ ] 1.4 Seed script for first-party courses — deferred to §6 (needs the authored manifests); `upsert` + the migration already support seeding
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
- [ ] 5.4 **Deferred** — add the course-completion badge to `curation-rewards` (its badges are computed from curator metrics, so a completion badge needs a new category). The server `newly_completed` signal is the once-per-course hook it will use.
- [x] 5.5 `CourseProgressService` seam (`RecordCourseCompletion`/`GetCourseProgress`); the completion notifier reconciles with the server via `ref.listen(canUseOnlineServicesProvider)` — merges the server set, records on finish, and pushes local-only completions on sign-in (guest→account); all best-effort (offline/guest → local only)
- [x] 5.6 Rust tests (first completion `newly_completed`, replay only counts, per-user isolation) + Flutter tests (`course_completion_sync_test.dart`: server merge, guest stays local, record-on-finish, guest→account push). clippy/fmt/analyze/custom_lint clean.

## 6. First-wave course content (seed)

- [ ] 6.1 Author the **first-wave** manifests fully in `{en,fr,es,it}` per `catalogue.md` (~11: Track A beginner `sol-portee-notes`/`sol-cles`/`sol-nom-notes`/`sol-valeurs`/`sol-silences`/`sol-mesure`/`sol-alterations` + Track B `app-prise-en-main`/`app-mode-synthesia`/`app-mode-horizontal`/`app-mode-partition`), each `instrument: "piano"` with `track`/`level`
- [ ] 6.2 Add them to the seed script (§1.4); verify they parse + play through the engine end-to-end
- [ ] 6.3 Leave the remaining catalogue as a documented **backlog** (data-only; no app release needed to add more)

## 7. Supersede the old sketch + quality gate

- [ ] 7.1 Remove the superseded `notation-lessons` capability from `add-notation-help` (delete its delta spec, drop from that proposal/tasks); re-validate `add-notation-help --strict`
- [ ] 7.2 `flutter analyze` + `dart format` + `dart run custom_lint` clean; `cargo fmt --check` + `cargo clippy -D warnings` clean
- [ ] 7.3 Coverage ≥ 80% both ecosystems
- [ ] 7.4 `openspec validate add-notation-courses --strict` passes
