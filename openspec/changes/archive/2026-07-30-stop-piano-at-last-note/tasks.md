## 1. Core logic — effective end

- [x] 1.1 Add pure `effectiveEndMs(List<TimedNote> visibleNotes, {required double songEndMs})` to [player_data.dart](apps/music/lib/state/player_data.dart) next to `effectiveStartMs`: return `songEndMs` when `visibleNotes` is empty; otherwise return `min(songEndMs, max(n.startMs + n.durationMs))` over the notes.
- [x] 1.2 Add selection-scoped getter `double get endMs => effectiveEndMs(visibleNotes, songEndMs: songEndMs);` on `PlayerData`, mirroring `startMs`.

## 2. Transport — end-of-song

- [x] 2.1 In `advance()` of [player_notifier.dart](apps/music/lib/state/player_notifier.dart:491), change the end-of-song threshold from `s.songEndMs` to `s.endMs`: scored branch clamps `next = s.endMs` and sets `finishScoredRun`; unscored branch keeps wrapping to `s.startMs`.
- [x] 2.2 Verify Wait Mode reaches completion at the last note (the gate never freezes on trailing rests since rests are not in `notes`); adjust only if a trailing-rest edge case blocks completion.
- [x] 2.3 Confirm no sibling-provider invalidation is needed — a hand change already recomputes `endMs` via `visibleNotes` (per the architecture rule).

## 3. Tests

- [x] 3.1 New `apps/music/test/effective_end_test.dart` for the pure function + `PlayerData.endMs`: both-hands / left / right selection, no-notes → falls back to `songEndMs`, unsorted notes, trailing rests ignored, held final note clamped to `songEndMs`, `endMs > startMs`.
- [x] 3.2 Extend `apps/music/test/player_notifier_test.dart` end-of-song group: scored run finishes at `endMs` (not `songEndMs`), unscored run loops from `endMs` back to `startMs`, hand change recomputes `endMs`, Wait Mode completes at the last note.
- [x] 3.3 Run codegen + gates: `dart run build_runner build --delete-conflicting-outputs`, `melos run analyze`, `dart run custom_lint`, `flutter test --coverage` (≥ 80%).

## 4. Spec validation

- [x] 4.1 `openspec validate stop-piano-at-last-note --strict` passes.
