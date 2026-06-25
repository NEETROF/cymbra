## 1. State: expected-by-hand + near-miss helpers

- [x] 1.1 Add `Set<int> expectedNotesForHand(double t, {required bool rightHand})` to `PlayerData` (window test like `requiredNotesAt`, filtered by `staff == 1` right / `staff >= 2` left).
- [x] 1.2 Add a pure `int nearMissPitch(int expected, {required int lowBound, required int highBound, required Set<int> avoid, required int Function(int) nextRandom})`: a pitch within ±3 of `expected`, not in `avoid`, clamped to range; fall back to nearest valid in-range non-avoided pitch.

## 2. Input: four assist keys

- [x] 2.1 In `_PlayerScreenState`, remove `_keyToPitch`; add `Map<LogicalKeyboardKey, Set<int>> _assistPressed`.
- [x] 2.2 Rewrite `_onKey`: map `keyA`→left-correct, `keyZ`→right-correct, `keyQ`→left-near-miss, `keyS`→right-near-miss. On key-down compute pitches, `noteOn` each, store; on key-up `noteOff` the stored set and clear. Other keys → ignored.
- [x] 2.3 Correct keys use `expectedNotesForHand`; near-miss keys pick via `nearMissPitch` off one expected note (avoid set = all expected pitches), using a `Random` and the player's `keyboardBounds` for range.
- [x] 2.4 No-op when the hand has no expected note.

## 3. Tests

- [x] 3.1 Unit-test `expectedNotesForHand`: right→staff-1 pitches, left→staff-2 pitches, empty when nothing due.
- [x] 3.2 Unit-test `nearMissPitch`: result within ±3, never in `avoid`, always within bounds (incl. an edge-of-range expected note).
- [x] 3.3 Widget-test: right-correct key satisfies Wait Mode (gate unblocks); near-miss key does not unblock and produces a note ≠ expected.
- [x] 3.4 Widget-test: a former pitch key (e.g. keyD) now produces no note; on-screen keyboard still plays exact notes.
- [x] 3.5 Update the existing "computer keyboard fallback" test to the new assist scheme.

## 4. Verify & gate

- [x] 4.1 `cd apps/music && dart run build_runner build --delete-conflicting-outputs`; `melos run analyze`, `dart format`, `dart run custom_lint` clean.
- [x] 4.2 `flutter test --coverage --exclude-tags golden` green and Flutter line coverage ≥ 80%.
- [ ] 4.3 Manually confirm in-app: in Wait Mode, the correct-hand key advances; the near-miss key plays a wrong nearby note and does not advance.
- [x] 4.4 `openspec validate assisted-practice-keys --strict` passes.
