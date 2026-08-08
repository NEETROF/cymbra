## Context

`add-notation-help` shipped contextual, in-place help (long-press a staff symbol → localized
bubble) and a glossary, reusing the `feature-discovery` coach mechanism. It deliberately deferred a
guided *course* to a "phase 2" (`notation-lessons`) sketched as local-only. This change replaces
that sketch with a richer, decided scope: courses live **on the home screen**, are **infinitely
replayable**, track completion **across devices** (so: backend), award a **badge**, and are written
in a **self-contained manifest** that could later carry community-authored courses.

Grounding in the existing code:
- The home/library screen (`apps/music/lib/screens/library_screen.dart`) already reserves a slot
  "pinned above the favorites list" (~line 140) with a `_FavoritesBody`; the Courses section slots
  in there.
- A rewards/badges system exists (`curation-rewards`, `reward-unlocks`,
  `services/curator_rewards_service.dart`); the completion badge composes with it.
- Durable, cross-device, per-user persistence already has a pattern to mirror (`play-activity-sync`,
  leaderboards): a Rust store behind a trait seam + a gRPC on `ScoreService`, with a host-testable
  core and mockall doubles; the Flutter side keeps the native/gRPC seam injectable.
- Diagrams can reuse the notation painters/SMuFL glyphs from `add-notation-help`.

State is Riverpod 2 + Freezed; UI never calls services directly; coverage gate ≥ 80% both
ecosystems. This change is **sequenced after `add-notation-help`** (shares its notation vocabulary).

## Goals / Non-Goals

**Goals (2a + 2b):**
- A home-screen **Courses** section above favorites, one tile per course, with a completion badge/indicator.
- A **versioned, inline-localized, self-contained course manifest** format + at least one bundled
  first-party course; parsing is defensive (unknown `schemaVersion` → skip, not crash).
- A self-paced **lesson player** (explanation / diagram / quiz), skippable, quiz non-blocking,
  **infinitely replayable**.
- **Cross-device completion**: server-persisted for signed-in users, read back anywhere; guests
  local until sign-in.
- A **completion badge** on first finish (once per course), via the existing rewards system.

**Non-Goals:**
- **2c — community-authored courses** (catalogue + propose + moderation of third-party manifests):
  the format is *designed for it* but it is **not built here**.
- No virtual/AI tutor, no LLM, no TTS.
- No change to points/shop economics beyond adding one badge.
- Not re-opening `add-notation-help`'s contextual help (only removing its superseded
  `notation-lessons` sketch).

## Decisions

### 1. Course content lives in a self-describing manifest, not the app's ARB
A course is a JSON **manifest** with `schemaVersion`, `id`, and ordered `steps`
(`explanation | diagram | quiz`). **All copy is localized inline** as `{en, fr, es, it}` maps inside
the manifest. First-party courses ship under `assets/courses/**`.
- *Why:* the explicit ask is that the scripting could **later carry courses authored by others**.
  App ARB keys can't localize arbitrary third-party text, so content must be self-contained in the
  file. First-party courses use the same format for consistency and to dogfood it. Only the *UI
  chrome* (buttons, section title) stays in ARB.
- *Alternative rejected:* ARB keys per lesson string — blocks community courses, would force a
  re-migration later.
- *Defensive parsing:* an unknown/greater `schemaVersion` skips that course (and logs), so a newer
  community course never crashes an older app.

### 2. Diagrams reference built-in renderers, not shipped images
A `diagram` step references a **built-in diagram id** (e.g. "treble-clef", "quarter-vs-eighth")
rendered by the existing notation painters/SMuFL glyphs, rather than embedding bitmaps.
- *Why:* keeps manifests tiny/portable and diagrams crisp/themed. A future community course can only
  pick from the app's known diagram ids (a safe, closed set) — arbitrary asset embedding is a 2c
  concern with its own moderation.

### 3. Cross-device completion mirrors the play-activity persistence pattern
Completion is a small per-user fact: `(userId, courseId) → { completedAt, playCount }`. Persist it
**server-side** via a new store (trait seam + Postgres table, migration, cascade on erasure) and a
gRPC pair on `ScoreService` — `RecordCourseCompletion(courseId)` and `GetCourseProgress()` — with a
**host-testable award core** (idempotent: first completion awards the badge, replays only bump the
count). The Flutter side caches completion locally and reconciles with the server through the
injectable client seam; a **guest** keeps completion in `shared_preferences` until sign-in.
- *Why:* reuses a proven, testable shape; keeps the ≥80% Rust core out of the thread/hardware glue.
- *Alternative rejected:* a full durable outbox like scored runs — completion is idempotent and
  low-stakes; a direct record + read is enough. (If offline resilience is wanted later, the same
  outbox can wrap it.)

### 4. The completion badge is awarded server-side, once per course
The award decision lives with the store (it knows "first time"), composing with `curation-rewards`,
so replays never re-award and two devices can't double-award. The client surfaces the badge through
the existing rewards feedback.
- *Why:* idempotency and cross-device correctness belong on the server; the client only renders.

### 5. Home placement as a distinct section, additive to favorites
A `_CoursesSection` renders above `_FavoritesBody` in the library. It reads the course list + a
completion map from a Riverpod notifier; tiles are lightweight and the section never blocks the
favorites (which stay scrollable below).
- *Why:* the ask is explicit — courses visible on the home screen, not behind a menu.

### 6. Layer this cleanly over `add-notation-help`
Reuse its painters/glossary content for diagram rendering; **remove** the now-superseded
`notation-lessons` capability from `add-notation-help` so there is a single source of truth. The
help/tips surface may still link to courses, but the primary entry is the home section.

## Risks / Trade-offs

- **Scope: 2b brings the backend in.** → Keep 2a (home section + manifest + bundled course + local
  completion) independently valuable and land it first inside this change; 2b (server persistence +
  badge) builds on it. If backend sequencing slips, the local completion still ships.
- **Manifest format churn once community courses come (2c).** → Version it (`schemaVersion`) from day
  one and parse defensively, so first-party and future community courses coexist; the closed diagram-
  id set keeps third-party content safe.
- **Double-award / device races on the badge.** → Award is server-side and idempotent per
  `(userId, courseId)`; unit-tested in the host-testable core.
- **Guest → sign-in completion loss.** → On sign-in, best-effort push local completions to the
  server (idempotent), matching how other guest→account transitions are handled.
- **Home clutter.** → One compact section above favorites; collapses/omits gracefully when there are
  no courses.
- **Coverage across two ecosystems.** → Host-testable Rust award/selection core (mockall store) and
  Flutter manifest-parsing + player + sync behind seams; native lib not required for either.

## Migration Plan

Additive. Order:
1. **2a:** manifest model + defensive parser + bundled course; home Courses section + tiles; lesson
   player (explanation/diagram/quiz, replay); **local** completion. (No backend.)
2. **2b:** Postgres table + migration (cascade on erasure); store trait + host-testable award core;
   `RecordCourseCompletion`/`GetCourseProgress` gRPC; Flutter sync + guest→account push; completion
   badge via `curation-rewards`.
3. Remove the superseded `notation-lessons` capability from `add-notation-help`.
Rollback: additive table and additive UI; disabling the section/RPC leaves play unaffected.

## Open Questions

- **Course catalogue for launch:** how many first-party courses and their topics/order (at least one
  "reading the staff" course to start; the rest can follow without format change).
- **Badge design/threshold:** one badge per course, or a single "finished your first course" badge
  plus a "completed all courses" badge? (Starting point: one per course, awarded once.)
- **Diagram id set:** the initial closed list of built-in diagram ids the manifest may reference.
- **2c community pipeline** (catalogue + propose + moderation) is deferred to its own change.
