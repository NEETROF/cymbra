## Why

We already trim leading silence so the game starts a short lead-in before the first
sounding note (`first-note-start`). But the end is still driven by the raw `songEndMs`,
which trailing rests inflate — so a piece that ends with several empty measures keeps the
game running over silence after the last note. Nothing is played or judged during that
tail; it just makes the run feel like it drags to a dead stop, degrading the practice
experience. This is the symmetric fix to the trimmed start.

## What Changes

- Add an **effective end** derived from the last sounding note of the currently selected
  hand(s) — the largest `startMs + durationMs` among the notes actually shown and played —
  so trailing rests and empty trailing measures after the last note are skipped.
- Drive end-of-song behaviour off the effective end instead of the raw `songEndMs`:
  - A **scored run** finishes (produces its summary and pauses) right after the last note
    resolves, not after the trailing silence.
  - An **unscored run** loops back to the effective start at the effective end.
  - **Wait Mode** completes at the last note rather than gating through trailing rests.
- Apply in all three render modes (waterfall / scrolling staff / Partition); the set of
  notes played or judged does not change — only where the run ends.
- Fall back to the raw `songEndMs` when the selection has no sounding notes.

## Capabilities

### New Capabilities
- `last-note-end`: derive an effective end from the last sounding note of the selected
  hand(s), trim trailing silence, and anchor loop / scored-run completion / Wait-Mode
  finish to it across every render mode.

### Modified Capabilities
<!-- No existing requirement changes; first-note-start stays as-is and this is its symmetric sibling. -->

## Impact

- `apps/music/lib/state/player_data.dart` — add `effectiveEndMs(...)` pure function
  (symmetric to `effectiveStartMs`) and a selection-scoped `PlayerData.endMs` getter.
- `apps/music/lib/state/player_notifier.dart` — `advance()` end-of-song block uses `endMs`
  (loop wrap, scored-run finish, Wait-Mode completion) instead of the raw `songEndMs`.
- Tests: new `effective_end_test.dart` plus `player_notifier_test.dart` end-of-song cases.
- No data-model or API change; `songEndMs` stays as the raw fallback. UI is unaffected.
