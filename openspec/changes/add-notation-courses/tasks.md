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

- [ ] 3.1 `_CoursesSection` above `_FavoritesBody` in `library_screen.dart` (no menu; never blocks favorites; omits when empty), **grouped by track + level** + course **tile** with completion indicator
- [ ] 3.2 `LessonPlayerScreen` running blocks at the user's pace; renders `text`, `diagram` (existing painters), `image`/`video` (from URL); a Next/skip gate; leaveable anytime
- [ ] 3.3 `question` block: multiple-choice / true-false with immediate feedback, never hard-blocks continuing; accessible controls
- [ ] 3.4 ARB keys for section title + player/tile chrome (en/fr/es/it) — UI chrome only (content is manifest-inline)
- [ ] 3.5 Widget-test: section above favorites, tile opens the player, step-through + skip, question feedback non-blocking, an `unsupported`/media block is skipped without error

## 4. Interactive blocks: playKey + score

- [ ] 4.1 `playKey` block: prompt to play a note/chord, validated via the existing MIDI + on-screen-keyboard + scoring seams; advance on correct input; always a non-blocking skip; hint after N tries
- [ ] 4.2 `score` block: parse inline MusicXML → `ScoreDocument`, engrave via `PartitionPainter`; when `playable`, embed the player/scoring so the user performs it (gate = performed)
- [ ] 4.3 Widget-test with faked MIDI/keyboard + scoring seams: correct input advances `playKey`, skip works; `score` renders and (playable) can complete

## 5. Cross-device completion + badge

- [ ] 5.1 Migration: `course_progress` (`user_id`, `course_id`, `completed_at`, `play_count`), cascade on erasure
- [ ] 5.2 `CourseProgressStore` trait + Postgres impl + **host-testable idempotent award core** (first completion sets `completed_at` + awards the badge; replay bumps `play_count` only, never re-awards)
- [ ] 5.3 gRPC `RecordCourseCompletion(courseId)` / `GetCourseProgress()` on `ScoreService`; wire the store
- [ ] 5.4 Add the course-completion badge to `curation-rewards`; award in the core; surface via existing badge feedback
- [ ] 5.5 Flutter completion notifier: local cache (`shared_preferences`, guest) reconciled with the server on load (non-blocking); record on finish; best-effort push local completions on sign-in
- [ ] 5.6 Tests — Rust: first completion awards once, replay/other-device never re-awards (`cargo llvm-cov` ≥ 80% core). Flutter: signed-in completion shows across a fresh container, guest stays local, load-in-flight still opens the course

## 6. First-wave course content (seed)

- [ ] 6.1 Author the **first-wave** manifests fully in `{en,fr,es,it}` per `catalogue.md` (~11: Track A beginner `sol-portee-notes`/`sol-cles`/`sol-nom-notes`/`sol-valeurs`/`sol-silences`/`sol-mesure`/`sol-alterations` + Track B `app-prise-en-main`/`app-mode-synthesia`/`app-mode-horizontal`/`app-mode-partition`), each `instrument: "piano"` with `track`/`level`
- [ ] 6.2 Add them to the seed script (§1.4); verify they parse + play through the engine end-to-end
- [ ] 6.3 Leave the remaining catalogue as a documented **backlog** (data-only; no app release needed to add more)

## 7. Supersede the old sketch + quality gate

- [ ] 7.1 Remove the superseded `notation-lessons` capability from `add-notation-help` (delete its delta spec, drop from that proposal/tasks); re-validate `add-notation-help --strict`
- [ ] 7.2 `flutter analyze` + `dart format` + `dart run custom_lint` clean; `cargo fmt --check` + `cargo clippy -D warnings` clean
- [ ] 7.3 Coverage ≥ 80% both ecosystems
- [ ] 7.4 `openspec validate add-notation-courses --strict` passes
