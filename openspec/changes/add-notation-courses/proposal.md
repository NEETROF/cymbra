## Why

`add-notation-help` teaches a symbol **in place** (long-press → bubble), but it doesn't teach a
beginner to *read and play* as a skill. New users need short, guided **courses** they find
immediately on the home screen, work through at their own pace, replay freely, and see progress on
**across their devices**, with a reward for finishing.

Rather than a one-off lesson viewer, this introduces a **generic, versioned format for interactive
content** — a document of typed **blocks** (explanation, diagram, image, video, question, play-a-key,
embedded score…) that an engine interprets. First-party courses are authored in it now; the same
format and the same server delivery can later carry courses **authored by other people** (moderated,
2c). This is the reusable foundation the ask calls for ("a file norm for many interactive actions").

This change supersedes the deferred `notation-lessons` slice sketched inside `add-notation-help`
(local-only, viewer-only); that sketch is removed there.

## What Changes

- **Courses live on the server, in the database, from day one.** A course is a **JSON manifest
  stored as a `JSONB` blob** in a `music.courses` table (metadata + content), delivered to the app
  by gRPC (`ListCourses` / `GetCourse`). The app **does not bundle courses**; it fetches them (with
  a local cache for offline). First-party courses are **seeded** into the DB (a script, like
  sound fonts / scores).
- **A generic interactive-content format (a block DSL).** A manifest has a `schemaVersion`, an id,
  inline-localized metadata, and an ordered list of **blocks**. Each block has a `type`, localized
  content, an optional **media reference (URL)**, and an **advance gate** (how the user moves on).
  The format is **forward-compatible**: an app that meets a block `type` it doesn't understand
  **skips it gracefully**, so new block types can be published server-side before every client
  supports them.
- **The v1 block vocabulary:** `text`, `diagram` (built-in notation diagram rendered by the existing
  painters), `image` (URL), `video` (URL), `question` (multiple-choice / true-false with feedback),
  **`playKey`** (the user must play a note/chord on the on-screen keyboard **or MIDI** — validated
  through the existing MIDI + keyboard + scoring seams), and **`score`** (an embedded notation
  excerpt rendered by the existing renderer, optionally **playable** by reusing the player/scoring).
  The format reserves room for more block types later without a breaking change.
- **A "Cours" section on the home screen, above the favorites.** The library (start screen) gains a
  Courses section pinned above the favorited scores, **one tile per course** with a **completion
  indicator** — not hidden behind a menu.
- **A self-paced lesson player** that runs a manifest's blocks: read, watch, answer, play. Quiz/play
  blocks give immediate feedback but never hard-block progress (a skip/continue is always offered);
  a course is **skippable, never a prerequisite to play, and replayable without limit**.
- **Cross-device completion (backend) + a badge.** Completion is persisted server-side and read back
  on any device (guests local until sign-in); first completion awards a **badge** via the existing
  rewards system (once per course; replays don't re-award; award decided server-side so devices
  can't double-award).
- **Media hosting is deferred (not the format).** `video`/`image` blocks are in the format and the
  player renders them from a URL, but where first-party media is hosted (object storage/CDN) is
  decided later; the first seeded course leans on `text`/`diagram`/`question`/`playKey`/`score`.
- **Out of scope (2c, deferred): community-authored courses** — a catalogue + propose + moderation of
  third-party manifests, reusing the scores/sound-font pipeline. The format and the server storage
  are built to support it, but the authoring/moderation UX is not built here. No virtual/AI tutor.

## Capabilities

### New Capabilities
- `notation-courses`: the server-stored (`JSONB`) course manifests and their gRPC delivery; the
  **generic, versioned, forward-compatible interactive-block format** and its engine (v1 blocks:
  text, diagram, image, video, question, playKey, score); the home-screen Courses section + tiles
  with a completion indicator; the self-paced lesson player (skippable, infinitely replayable);
  **cross-device completion**; and the **completion badge**.

### Modified Capabilities
- `curation-rewards`: add a **course-completion badge** to the rewards catalogue, awarded once when a
  user first completes a course. (Badge set only; points/shop rules unchanged.)

## Impact

- **Rust backend** (`cymbra-music` + `backend`): a `music.courses` table (id, status, order,
  `schema_version`, inline-localized title, **`content JSONB`**) + a per-user `course_progress` table
  (completion + play count, cascade on erasure) via migrations; a `CourseRepo` + `CourseProgressStore`
  (trait seams, Postgres impls, host-testable idempotent award core); gRPC `ListCourses` / `GetCourse`
  / `RecordCourseCompletion` / `GetCourseProgress` on `ScoreService`; a **seed script** for
  first-party courses. Badge award composes with `curation-rewards`. The manifest is **stored/served
  as an opaque blob** — the backend does not need to understand blocks (forward-compat lives in the
  client); light server-side validation (schemaVersion present, JSON well-formed).
- **App** (`apps/music`): a **course-manifest model + block engine** (Freezed union over block types,
  defensive parser skipping unknown types, inline-i18n resolver); a **lesson-player** driving the
  blocks, with `playKey` wired to the MIDI/keyboard/scoring seams and `score` to the notation
  renderer/player; a **Courses section** above `_FavoritesBody` in `library_screen.dart` with tiles;
  Riverpod state for the course list (server + cache), the active lesson, and **completion** (local
  cache reconciled with the server via the injectable client seam); the completion badge via the
  existing rewards feedback.
- **Relates to** `add-notation-help` (shared painters/glyphs for `diagram`/`score`; supersedes its
  `notation-lessons` slice), `midi` / `keyboard-display` / `performance-scoring` (the `playKey` and
  playable-`score` gates), `score-notation` / `web-notation-render` (`score` rendering),
  `curation-rewards` (badge), `saved-catalog-library` (home layout), `play-activity-sync` (durable
  cross-device pattern), `backend-score-storage` / `soundfont-storage` (blob/seed patterns),
  `state-management`, `app-localization` (UI chrome; course *content* is manifest-inline), and — for
  2c later — `score-moderation` / the propose pipeline (#170/#168).
- **Coverage**: Rust ≥ 80% on the repo/award core (mockall doubles); Flutter ≥ 80% on manifest
  parsing + the block engine, the Courses section/tiles, the lesson-player flow (incl. question and a
  faked playKey/score gate), and completion sync behind the injectable seams.
