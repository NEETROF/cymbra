## 1. Split the judgment clock by mode

- [x] 1.1 Add `PlayerData.judgmentClockAt(playheadMs)` — the unshifted playhead in Wait Mode, the heard position in free run — plus `judgmentClockMs` for the current playhead.
- [x] 1.2 Route all five scorer call sites in `player_notifier.dart` (`noteOn` live, `noteOff`, `tick`, the held-note `noteOn`, `finishRun`) through it, removing the hand-duplicated `next - outputOffsetMs`.
- [x] 1.3 Document on `referenceMs` the wall-clock/score-clock unit error, and why rescaling by `speed` is not the fix.

## 2. Tests

- [x] 2.1 Repair the vacuous Wait Mode assertion in `test/state/output_offset_test.dart`: assert the press was judged at all before asserting what it scored.
- [x] 2.2 Unit-pin `judgmentClockMs` in both modes, and pin that neither clock moves with the tempo.
- [x] 2.3 Behavioural guard: a mid-run tempo tap under an offset resolves no pending onset — verified to FAIL against the rejected `* speed` version.
- [x] 2.4 Run the Wait Mode credit test at a non-1x speed.
- [x] 2.5 Tighten `isNot(TimingVerdict.perfect)` to the exact verdict, so a bound-but-early press is not confused with a rejected extra note.
- [x] 2.6 Verify every new assertion FAILS against pre-fix semantics (one clock for both modes).
- [x] 2.7 `flutter test --exclude-tags golden`, `flutter analyze`, `dart run custom_lint`, `dart format` clean.

## 3. Spec

- [x] 3.1 Delta spec: `performance-scoring` / Output Offset Applied To The Timing Reference.
- [ ] 3.2 Manual check on a real delayed route (Bluetooth), Wait Mode and free run.
- [ ] 3.3 Archive after review/merge (`/opsx:archive fix-output-offset-units`).

## 4. Follow-up (NOT in this change)

- [ ] 4.1 Propose the drain design for the unit error: the heard position becomes a delayed copy of the playhead, so the lag is correct at every speed AND continuous across a tempo change. Tasks 2.2 and 2.3 are its acceptance criteria.
