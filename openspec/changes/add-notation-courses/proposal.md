## Why

`add-notation-help` teaches a symbol **in place** (long-press → bubble) and lists them in a
glossary, but it doesn't teach a beginner to *read the staff* as a skill — there is no guided,
self-paced path. New users need short **courses** they can find immediately (not buried in a menu),
work through at their own pace, replay freely, and see their progress on — **across their devices**,
with a little reward for finishing. The scripting should also be a **data format** that could later
carry courses authored by other people, reusing the app's existing propose→moderate→publish
pipeline (mirroring scores #170 and sound fonts #168).

This change supersedes the deferred `notation-lessons` slice sketched inside `add-notation-help`
(which assumed local-only, no backend): the requirements below replace it and it is removed there.

## What Changes

- **A "Cours" section on the home screen, above the favorites.** The library (start screen) gains a
  **Courses** section pinned above the favorited scores, with **one tile per course** — not hidden
  behind the help menu. Each tile shows the course and a **completion indicator**.
- **A versioned, self-describing course document format (community-ready).** Courses are
  **declarative manifests** (ordered steps: explanation / diagram / quiz) with a `schemaVersion`,
  **all copy localized inline in the manifest** (`{en, fr, es, it}`) rather than through the app's
  ARB — so a course is fully self-contained and a third party could author one later. First-party
  courses ship **bundled** (`assets/courses/**`); the format is designed so community courses can
  arrive by the same schema over the network after moderation (2c, deferred).
- **A self-paced lesson player.** Stepping through a course: explanation text, a diagram (reusing
  the existing notation painters/glyphs where possible), and light **quiz** steps that give
  immediate feedback but never block progress. **Skippable**, never a prerequisite to play, and
  **replayable infinitely**.
- **Cross-device completion (backend).** A course's completed state is persisted **server-side** and
  read back on any device, so the home tile shows "completed" everywhere — not just where it was
  finished. Guests get local-only completion until they sign in.
- **A completion badge.** Finishing a course awards a **badge** through the existing rewards/badges
  system, shown to the user.
- **Out of scope (2c, deferred): community-authored courses.** A course catalog + propose +
  moderation of third-party courses (reusing the scores/sound-font pipeline) is designed for by the
  manifest format but **not built here**. No virtual/AI tutor anywhere (future vision only).

## Capabilities

### New Capabilities
- `notation-courses`: the home-screen Courses section + per-course tiles with a completion
  indicator; the **versioned, inline-localized course manifest format** and the bundled first-party
  course(s); the self-paced lesson player (explanation / diagram / quiz, skippable, infinitely
  replayable); **cross-device completion** persistence; and the **completion badge**.

### Modified Capabilities
- `curation-rewards`: add a **course-completion badge** to the rewards catalogue, awarded when a
  user first finishes a course. (Only the badge set changes; the points/shop rules are untouched.)

## Impact

- **Rust backend** (`cymbra-music` + `backend`): a per-user **course-progress** store (new table
  via migration; completion + replay count, keyed by course id, cascading on account erasure) and
  gRPC (`RecordCourseCompletion` / `GetCourseProgress` on `ScoreService`, following the
  play-activity persistence pattern); host-testable selection/award core kept out of the hardware
  glue. Badge award composes with `curation-rewards`.
- **App** (`apps/music`):
  - `lib/screens/library_screen.dart`: a **Courses** section above `_FavoritesBody`.
  - A **course manifest** model + asset loader (`assets/courses/**`, inline i18n), a lesson-player
    screen, and Riverpod state for the course list, the active lesson, and **completion** (local
    cache reconciled with the server through the injectable client seam).
  - Completion badge surfaced through the existing rewards feedback.
- **Relates to** `add-notation-help` (shared notation vocabulary/painters for diagrams; supersedes
  its `notation-lessons` slice), `curation-rewards` / `reward-unlocks` (badge), `saved-catalog-
  library` (home layout it sits above), `play-activity-sync` (the durable cross-device persistence
  pattern reused), `state-management`, `app-localization` (UI chrome; course *content* is
  manifest-inline), and — for 2c later — `score-moderation` / the propose pipeline (#170/#168).
- **Coverage**: Rust ≥ 80% on the progress/award core (mockall doubles for the store); Flutter
  ≥ 80% on the manifest parsing, course list/tiles, lesson-player flow, and completion sync behind
  the injectable seams.
