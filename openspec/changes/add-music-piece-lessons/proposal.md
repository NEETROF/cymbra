# Proposal

## Why

The learning path teaches reading, rhythm and the ear in 42 lessons, but it never makes
the learner **play a piece**. The lesson player and the score player are two worlds: a
learner finishes "Do, ré, mi" and is left to find a score in the library on their own, with
no suggested piece, no imposed hand or tempo, and no bar to clear. Every piano-learning app
people compare Cymbra to closes each notion with a target piece — right hand, left hand,
hands together, tempo up. This change adds that missing step as one new block in the
existing course format, so the path finally leads to the keyboard.

## What Changes

- **A `piece` lesson block.** A course block that opens the **existing score player** on a
  catalogue or bundled piece with **imposed settings** — hands (right / left / both),
  playback speed, Wait Mode on or off, metronome, reading aid — and a **success threshold**
  (overall synchronisation percentage, default 80 %). When the run ends, the player returns
  to the lesson with the session result: the step is **passed** when the threshold is met,
  **visited** otherwise. A visited step never traps the learner: *Try again* and *Next* are
  both offered.
- **Staged chains are authored, not coded.** A lesson closes a notion with a sequence of
  `piece` blocks on the same score: right hand at 60 % speed in Wait Mode, left hand the
  same, hands together at 60 %, hands together at full tempo without Wait Mode. The chain
  is content; the block only needs per-step hands, speed and gating.
- **Passed state is persisted across devices.** The per-course progress record gains a
  *passed* timestamp and the *best synchronisation* reached on the course's piece steps,
  recorded idempotently like completion. The path shows three states on a piece node:
  not started, visited, passed.
- **The player knows it is in a lesson.** In lesson context the pre-play setup modal is not
  shown (settings are imposed), the header shows the lesson's name and target, and the
  end-of-run summary's primary action is **Back to the lesson**. The player's own
  preferences are untouched: imposed settings are a per-run override, never persisted.
- **Repertoire content.** Each of the seven solfège units ends with a piece lesson that uses
  only the notions taught so far, drawn from the bundled scores and the public catalogue's
  redistributable pieces (licence-filtered at ingest). About fifteen pieces across the path
  in the first batch, chosen for note range and rhythm, not fame.
- **Forward-compatible by construction.** Older apps skip an unknown `piece` block, so the
  content can be published before every client is updated.

## Capabilities

### New Capabilities
- `music-piece-lessons`: the `piece` block and its imposed settings, the success threshold
  and the passed / visited outcome, the player's lesson context, the cross-device passed
  record, and the repertoire-content rule for the path.

### Modified Capabilities
- `notation-courses`: "The lesson player runs the v1 block types" gains the `piece` block
  (the player opens a catalogue piece with imposed settings and reports the outcome).

## Impact

Products: **Music only**. Consumed, not new: the score player and its scoring, Wait Mode,
hand selection, speed and metronome, the reading aid, the offline score cache, the catalogue
licence filtering, the course manifest format and its server storage, course progress.
New: one block type, a lesson context in the player, two additive fields on the course
progress contract, the repertoire content.

- Flutter: `courses/course_manifest.dart` (`PieceBlock`), a `widgets/piece_block_view.dart`,
  `screens/player_screen.dart` and `screens/pre_play_setup_modal.dart` (lesson context),
  `widgets/session_summary_modal.dart` (back-to-lesson action), `state/player_notifier.dart`
  (per-run imposed settings), `state/course_completion_notifier.dart` (passed + best).
- Backend: `music.course_progress` gains `passed_at` and `best_sync_pct`;
  `RecordCourseCompletionRequest` gains `passed` and `best_sync_pct`, `CourseProgress`
  gains `passed_at_ms` and `best_sync_pct` — **additive** fields, `buf breaking` passes.
- Content: `backend/content/courses/*.json` gains seven closing piece lessons; the seed
  script regenerates `seed_courses.sql`.
- Tests: manifest parsing (unknown fields, old apps skip), block view states, player lesson
  context, summary action, notifier passed/best merge, Rust repo + RPC with mockall doubles,
  content corpus test extended to `piece` blocks.
