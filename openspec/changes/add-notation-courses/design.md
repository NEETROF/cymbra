## Context

`add-notation-help` shipped contextual, in-place help and deferred a guided *course* to a "phase 2"
(`notation-lessons`) sketched as a local, viewer-only feature. This change replaces that with a
decided, larger scope: courses are **server-stored interactive content**, delivered to the app,
shown **on the home screen**, **infinitely replayable**, tracked **across devices** (badge on
finish), and written in a **generic, versioned block format** that can later carry
community-authored courses.

Grounding in existing code/patterns:
- Home/library screen reserves a slot above the favorites (`library_screen.dart`, `_FavoritesBody`).
- Rewards/badges exist (`curation-rewards`, `services/curator_rewards_service.dart`).
- Durable per-user server state has a pattern to mirror (`play-activity-sync`, leaderboards): a Rust
  store behind a trait seam + gRPC on `ScoreService`, host-testable core, mockall doubles; Flutter
  keeps the gRPC seam injectable.
- Blob content + seed pipeline exists for scores (MusicXML) and sound fonts.
- The notation renderer (`PartitionPainter`/`StaffPainter`) and the diagram content from
  `add-notation-help` render `diagram`/`score` blocks; the MIDI + on-screen-keyboard + scoring seams
  (`midi`, `keyboard-display`, `performance-scoring`) validate `playKey` and playable `score` blocks.

State is Riverpod 2 + Freezed; UI never calls services directly; coverage ≥ 80% both ecosystems.
Sequenced after `add-notation-help`.

## Goals / Non-Goals

**Goals:**
- Courses **stored in the DB as a JSON (`JSONB`) manifest**, delivered by gRPC, seeded for
  first-party content; **no bundled courses**, with a local cache for offline.
- A **generic, versioned, forward-compatible block format** + an engine that runs it; v1 blocks:
  `text`, `diagram`, `image`, `video`, `question`, `playKey`, `score`.
- A **home Courses section** above favorites (tile + completion indicator); a **self-paced lesson
  player**; skippable; **infinitely replayable**.
- **Cross-device completion** (server-persisted, guest local until sign-in) + a **completion badge**
  (once per course, server-decided).

**Non-Goals:**
- **2c — community-authored courses** (catalogue + propose + moderation of third-party manifests):
  the format + server storage are built *for* it; the authoring/moderation UX is not built here.
- **Media hosting**: `video`/`image` blocks are supported by the engine (render from a URL), but the
  first-party media hosting pipeline (object storage/CDN) is decided later; the first seeded course
  avoids media.
- No virtual/AI tutor; no LLM/TTS.
- No points/shop economy change beyond one badge.

## Decisions

### 1. The manifest is an opaque `JSONB` blob the server stores and serves; the client owns the format
`music.courses` holds `(id, status, order, schema_version, title JSONB, content JSONB, …)`. The
backend treats `content` as an **opaque blob** — it validates only that it is well-formed JSON with a
`schemaVersion`, and serves it. All block semantics and **forward-compatibility live in the client**.
- *Why:* the format will evolve (new block types) faster than the backend should care; keeping the
  server format-agnostic means new block types ship without a backend release, and community content
  (2c) is just more rows. gRPC `ListCourses`/`GetCourse` deliver it; a seed script inserts
  first-party courses (like sound fonts/scores).
- *Alternative rejected:* a fully-typed relational schema for blocks — brittle against an evolving,
  community-extensible DSL.

### 2. The interactive format is a discriminated union of typed blocks, forward-compatible
A manifest = `{ schemaVersion, id, title/i18n, blocks:[…] }`. A **block** = `{ type, …content (i18n),
media? (url), gate? }`. The client models blocks as a **Freezed union**, and the parser is
**defensive**: an unknown `type` (or a block needing an unsupported capability) becomes a
**skippable "unsupported" block**, never a parse failure — so an older app survives newer
server-published content. Media is **URL-referenced**, never inlined.
- *Why:* content is now server-driven and clients update independently; forward-compat is mandatory,
  not nice-to-have. A closed, declarative vocabulary (no executable code; closed diagram-id and
  target sets) keeps future third-party content safe.
- v1 blocks: `text`, `diagram`, `image`, `video`, `question`, `playKey`, `score` (see §3–§4).

### 3. `playKey` reuses the MIDI + keyboard + scoring seams
A `playKey` block names a note/chord; the player listens on the **same input seams the game uses**
(on-screen keyboard + connected MIDI, validated like a required note), advancing when the correct
input arrives, always offering a non-blocking skip.
- *Why:* the app already has the whole input+validation stack; the course should make the user *play*,
  reusing it rather than re-implementing input.

### 4. `score` reuses the notation renderer and (optionally) the player
A `score` block carries a short **inline MusicXML** excerpt (self-contained → community-ready),
parsed by the existing engine into a `ScoreDocument` and engraved by `PartitionPainter`. Marked
`playable`, it embeds the existing player/scoring so the user performs it (gate = performed).
- *Why:* excerpts belong to lessons ("read this, now play it"); inline MusicXML keeps a course
  self-contained; rendering/playing reuse existing machinery.
- *Alternative considered:* reference a catalog `scoreId` — couples a course to catalog rows and
  breaks self-containment; inline wins for portability (a `scoreId` variant can be added later).

### 5. Cross-device completion + badge mirror the play-activity pattern, award server-side
Completion is `(userId, courseId) → { completedAt, playCount }` in `course_progress` (migration,
cascade on erasure). gRPC `RecordCourseCompletion(courseId)` / `GetCourseProgress()`; a **host-
testable, idempotent award core** (first completion sets `completedAt` and awards the badge via
`curation-rewards`; replays only bump `playCount`, never re-award). Flutter caches completion locally
and reconciles with the server; a **guest** keeps it in `shared_preferences` and best-effort pushes
on sign-in.
- *Why:* proven testable shape; idempotency/anti-double-award belong on the server.

### 7. Manifests are instrument-typed and track/level-tagged; content is a backlog, not a blocker
Every manifest carries `instrument` (`piano` now), `track` (solfège / app-usage / technique) and
`level` (beginner / intermediate / advanced), so the home section **groups** tiles and a future
**drums** track slots in by swapping only the interactive blocks (`playKey` → a pad block) — passive
blocks and the engine are unchanged. The proposed first-party catalogue (~50 courses across the three
tracks) is in `catalogue.md`; a **first wave** (beginner solfège + app basics, ~11 courses) is
authored/seeded to prove the engine end-to-end, and the rest is a **content backlog** shipped by
inserting rows — a course is data, so new courses need no app release.

### 6. Home placement as a distinct section; supersede the old sketch
A `_CoursesSection` above `_FavoritesBody`, reading the course list + completion map from a notifier;
never blocks favorites. **Remove** the superseded `notation-lessons` capability from
`add-notation-help`.

## Risks / Trade-offs

- **Big surface (backend + a content engine) in one change.** → Slice it (see Migration): backend
  storage/delivery → the block engine + player with the cheap blocks (`text`/`diagram`/`question`) →
  the hard interactive blocks (`playKey`, `score`) → completion+badge. Each slice ships and is tested.
- **Forward-compat regressions.** → The "unknown block → skip" path is a first-class, tested
  requirement; a course with an injected unknown block must still complete.
- **`playKey`/`score` input flakiness.** → Reuse the exact game input+scoring seams (already tested);
  fake them in widget tests; always offer skip.
- **Manifest size / media.** → Media by URL only; `score` uses short excerpts; the blob stays small.
- **Guest → account completion loss.** → Idempotent best-effort push on sign-in.
- **Server storing untrusted JSON (esp. 2c).** → Server validates shape minimally and treats content
  as data; the client never executes it; closed vocabularies; media host allow-listing is a 2c
  moderation concern.

## Migration Plan

Additive. Order (each a verified slice):
1. **Backend storage + delivery:** `music.courses` (+`schema_version`,`content JSONB`) migration;
   `CourseRepo` + gRPC `ListCourses`/`GetCourse`; seed script; minimal server validation.
2. **Manifest model + block engine (client):** Freezed block union + defensive/forward-compat parser
   + inline-i18n resolver; unit tests (incl. unknown-block skip).
3. **Home Courses section + tiles + lesson player** with the display/quiz blocks
   (`text`,`diagram`,`image`,`video`,`question`); fetch + cache.
4. **Interactive blocks:** `playKey` (MIDI/keyboard/scoring seam) and `score` (renderer + optional play).
5. **Completion + badge:** `course_progress` migration; store + idempotent award core; gRPC record/get;
   Flutter cross-device sync + guest push; badge via `curation-rewards`.
6. Remove the superseded `notation-lessons` from `add-notation-help`.
Rollback: additive tables/RPCs/UI; disabling the section leaves play unaffected.

## Open Questions

- **Media hosting** (object storage/CDN) for `video`/`image` — **decided: provision a dedicated OVH
  bucket for course media** (mirroring the private sound-font bucket), referenced by URL in
  manifests. The bucket + a delivery route are reserved now; the first course still avoids media, and
  the upload/authoring flow for media is wired when the first media-bearing course needs it.
- **First-party catalogue**: proposed in `catalogue.md` (~50 courses, 3 tracks, leveled); the first
  wave to author fully is listed there. Remaining courses are a content backlog.
- **Badge design**: one badge per course vs a "first course" + "all courses" pair (start: one per course).
- **Closed sets**: the initial `diagram` id list and `playKey`/`score` note-spec shape.
- **2c** (community catalogue + propose + moderation) is a separate later change.

## Decisions — v2 revision (interactive solfège)

### 8. Exercises are typed v2 blocks over a dedicated lesson staff
The first wave leaned on text/quiz; the target is a curriculum the learner *does*. v2 adds seven
blocks — `staff` (display), `readPlay` (staff→keyboard, drill/melody/set), `nameNote`, `placeNote`
(tap the staff), `rhythmTap` (metronome + pad, pure grading), `earChoice` (synth sequence + chips),
`buildChord` (toggle keys) — rendered by a purpose-built `LessonStaff` painter (SMuFL toolbox;
armure-aware accidental suppression; tap→step hit-testing) rather than the score-driven engravers,
which expose none of the exercise affordances (highlights, ghost previews, hit steps). Manifest
pitches are written spellings (`"F#4"`), never MIDI, so staff degree and enharmonics survive; all
sounding goes through one `LessonSounder` (every touch is audible — the silent playKey gap is
closed). `kCourseSchemaVersion` bumps to 2 and the listing filters unsupported versions, so older
apps never see v2 tiles (the decline path stays for the cache).

### 9. Kind gating, not free skipping
Interactive blocks now gate Next (a lesson is something you do), but the gate never traps: a
discreet "Passer" appears after 12 s, wrong answers cost nothing and are heard (a wrong key plays
its own pitch — the most musical error message), and the celebration counts first-try successes
without ever showing a failure state. Questions stay non-blocking as originally decided.

### 10. Units are catalogue data; the path is the product surface
`music.courses` gains `unit` + inline-i18n `unit_title` (migration 0021, CourseSummary fields 8/9),
so the 42-lesson curriculum groups into 7 named units without an app release. Home shrinks to one
continue card (next lesson + unit progress); a full `LearningPathScreen` lists units and lesson
nodes — completed/next/later — with soft ordering (visual hierarchy guides; nothing is locked).

### 11. The corpus is validated JSON files, compiled to seed SQL
42 hand-escaped SQL literals would be unmaintainable; courses live as one JSON file each under
`backend/content/courses/`, gated by a Flutter test that runs the REAL parser over every file
(zero-unsupported, locale completeness, musical lints: bar-filling rhythms, ear answers consistent
with pitches), and compiled by `gen_seed_courses.py` into the idempotent seed. The app-usage
tutorials are retired: the curriculum teaches solfège, not the app.
