> Sequenced after `add-notation-help` (shares its notation painters/glyphs for `diagram`/`score`).
> Courses are server-stored from day one; the client owns the block format + forward-compat. Media
> hosting and community authoring (2c) are out of scope for this change.

## 1. Backend: course storage + delivery

- [ ] 1.1 Migration: `music.courses` (`id`, `status`, `sort_order`, `schema_version`, `title JSONB` inline-i18n, `content JSONB` opaque manifest, timestamps); index for listing
- [ ] 1.2 `CourseRepo` trait + Postgres impl in `cymbra-music` (list published, get by id); minimal server-side validation (well-formed JSON + `schemaVersion` present) — content otherwise opaque
- [ ] 1.3 gRPC `ListCourses` / `GetCourse` on `ScoreService` (proto + `melos gen-grpc`), returning metadata + the manifest blob; music-scope auth consistent with other content
- [ ] 1.4 Seed script for first-party courses (inserts manifests, like sound fonts/scores)
- [ ] 1.5 Rust tests (mockall repo double) for list/get + validation; `cargo llvm-cov` ≥ 80% on the core

## 2. Client: manifest model + forward-compatible block engine

- [ ] 2.1 Freezed `CourseManifest` (`schemaVersion`, `id`, inline-i18n `title`/`summary`, `blocks[]`) and a **Block** union: `text`, `diagram`, `image`, `video`, `question`, `playKey`, `score`, plus an `unsupported` fallback
- [ ] 2.2 Defensive parser: unknown block `type` (or unsupported capability) → `unsupported` (skipped at play time), unknown top-level `schemaVersion` → course declined; inline-i18n resolver (current locale, fallback `en`)
- [ ] 2.3 Course-source seam (gRPC client) + Riverpod provider for the course list, with a local cache for offline
- [ ] 2.4 Unit-test parsing: valid manifest → ordered blocks; **injected unknown block still yields a completable course**; schema-version decline; i18n fallback; cache round-trip

## 3. Home "Cours" section + lesson player (display + quiz blocks)

- [ ] 3.1 `_CoursesSection` above `_FavoritesBody` in `library_screen.dart` (no menu; never blocks favorites; omits when empty) + course **tile** with completion indicator
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

## 6. Supersede the old sketch + quality gate

- [ ] 6.1 Remove the superseded `notation-lessons` capability from `add-notation-help` (delete its delta spec, drop from that proposal/tasks); re-validate `add-notation-help --strict`
- [ ] 6.2 `flutter analyze` + `dart format` + `dart run custom_lint` clean; `cargo fmt --check` + `cargo clippy -D warnings` clean
- [ ] 6.3 Coverage ≥ 80% both ecosystems
- [ ] 6.4 `openspec validate add-notation-courses --strict` passes
