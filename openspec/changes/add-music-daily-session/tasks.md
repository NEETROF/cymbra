# Tasks

## 1. Composition

- [x] 1.1 Write `state/daily_session_core.dart`: the pure `composeDailySession` over courses, completion, passed state, today's ledger and goal, including the path-finished mode; verify unit tests for start of path, mid-path with and without piece steps, finished path, and all-done-today.
- [x] 1.2 Add `DailyGoal` (light/normal/full) persisted through `preferencesServiceProvider`; verify notifier tests for default normal, change and restore.

## 2. Today's ledger

- [x] 2.1 Write `state/today_ledger.dart`: a Freezed `{localDay, lessonIds, pieceStepKeys, reviewKeys}` persisted locally, cleared on local-day rollover computed like the streak's offset; verify tests for write, restore, rollover and a clock change.
- [x] 2.2 Add `widgets/daily_session_listener.dart` recording lesson completions, returned lesson runs and finished reviews into the ledger, and raising the completion cue once per day; verify widget tests that each event writes once and the cue fires exactly once. *(Writes happen at the three recording call sites — lesson finish, piece runner, review finish — see D2; the listener widget `daily_session_listener.dart` raises the cue once.)*
- [x] 2.3 Merge server-known completion and passed state into "done today" for the lesson and piece items; verify notifier tests with a mockito `CourseProgressService` reporting today's completion. *(`completed_at_ms` added to `CourseProgress` for it; mockito `CourseProgressService` in `today_ledger_test.dart`.)*

## 3. Review mini-lesson

- [x] 3.1 Build a one-block synthetic manifest from a `readPlay`, `rhythmTap` or `earChoice` block of the latest completed unit and open it in the existing lesson player; verify tests that the block renders, gates and finishes into the ledger.

## 4. Block widget

- [x] 4.1 Build `widgets/daily_session_block.dart`: three items with done state, the goal ring, the streak line (from `streakProvider`, absent signed out); mount it in the Learn door's reserved slot; verify widget tests for every composition state, goal size and guest.
- [x] 4.2 Add the goal-size setting row and the four-locale strings; verify ARB alignment, `dart run custom_lint` and a widget test for the setting. *(The goal chips live in the block, see D3.)*
- [x] 4.3 Add the block golden on the pinned platform; verify the golden test passes.

## 5. Integration

- [x] 5.1 Integration test: open the Learn door, do the lesson item, see the ring advance, relaunch the same day and see it kept; verify `melos run integration` passes.
