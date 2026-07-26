## 1. Effective-start helper (pure)

- [x] 1.1 Add a `kStartLeadInMs` constant (bounded lead-in, ~1 beat / fall-in budget) in a shared spot (e.g. `player_data.dart` or `countdown.dart`).
- [x] 1.2 Add a pure `effectiveStartMs(List<TimedNote> visibleNotes, {double leadInMs})` helper: `0` when empty; otherwise `(firstOnset - leadInMs).clamp(0, firstOnset)`.
- [x] 1.3 Expose `PlayerData.startMs` (getter) returning `effectiveStartMs(visibleNotes, leadInMs: kStartLeadInMs)` so state and guards share one source of truth.
- [x] 1.4 Unit-test the helper: empty → 0; onset within lead-in → 0 (clamped); onset well past lead-in → `onset - leadIn`; whole-piece vs selected-hand filtering.

## 2. Seed the playhead at the effective start

- [x] 2.1 `_applyNotation(...)`: seed `elapsedMs: <effective start>` instead of `0` (`player_notifier.dart:~187`).
- [x] 2.2 `_loadDemo(...)`: seed `elapsedMs: <effective start>` instead of `0` (`player_notifier.dart:~218`).
- [x] 2.3 `restart()` / `restartFromTop()`: return to the effective start, not `0` (`player_notifier.dart:~389`).
- [x] 2.4 `setSelectedHands(...)`: recompute and seed the effective start for the new selection, not `0` (`player_notifier.dart:375-380`).
- [x] 2.5 `advance()` loop wrap-around: wrap to the effective start instead of the top (`player_notifier.dart:471-474`), reusing the seek-safe seam so no double note/tick fires.

## 3. Move the "fresh start" guards off absolute zero

- [x] 3.1 Add a `_atStart(PlayerData s)` predicate (`s.elapsedMs <= s.startMs`) centralising the fresh-start check.
- [x] 3.2 `startPlayback()`: arm the countdown when `_atStart` (fresh start) instead of `elapsedMs == 0` (`player_notifier.dart:324`).
- [x] 3.3 `_maybeStartRun()`: open the scored run when `_atStart` instead of `elapsedMs == 0` (`player_notifier.dart:117`); keep the `visibleNotes.isEmpty` guard.
- [x] 3.4 Confirm resume-from-pause is still excluded (a paused playhead is `> startMs`, so no re-trim / no countdown).

## 4. Verify adjacent behaviours at a non-zero start

- [x] 4.1 Free run: countdown freezes the playhead at the effective start, then the first note falls in and lands after GO (no pre-sound).
- [x] 4.2 Wait Mode: the brief lead-in advances in real time and the gate freezes at the first onset; leading rests beyond the lead-in are not played.
- [x] 4.3 Score audio: the first note sounds exactly once when time passes its onset (check `scoreNoteEdges` boundary with `from == effectiveStart`).
- [x] 4.4 Metronome + Partition cursor: beats re-align on the seeded start with no spurious tick; the cursor lands on the first note's measure.

## 5. Tests & gates

- [x] 5.1 `player_notifier_test.dart`: load / restart / hand-switch / loop all seed the effective start (add a leading-rest fixture where the first note is in measure 3).
- [x] 5.2 Assert a scored run opens and (free run) the countdown arms at a non-zero effective start.
- [x] 5.3 Assert a piece that starts at time zero is unchanged (regression), and resume-mid-piece does not re-trim.
- [x] 5.4 `melos run generate` (Freezed/Riverpod codegen), `melos run analyze` + `dart format`, `dart run custom_lint` clean.
- [x] 5.5 `flutter test --coverage --exclude-tags golden` green with line coverage ≥ 80%.

## 6. Spec validation

- [x] 6.1 `openspec validate game-start-first-note --strict` passes.
