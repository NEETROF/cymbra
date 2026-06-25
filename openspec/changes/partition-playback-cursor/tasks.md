## 1. Derivation: per-measure timing

- [x] 1.1 Add `List<int> measureStartMs` to `DerivedPlayback` and populate it in `notationToTimedNotes` from `measureStartDiv × msPerDivision` (first measure = 0).
- [x] 1.2 Thread `measureStartMs` into `PlayerData` (set in `_applyNotation`); default empty for the demo score.
- [x] 1.3 Add a pure helper `({int index, double fraction})? measureAt(double t)` on `PlayerData`: find the measure whose `[startMs, nextStartMs)` contains `t` and the fraction within it (null when empty/out of range).

## 2. Painter: cursor + note highlighting

- [x] 2.1 Add `elapsedMs`, `measureStartMs`, `songEndMs`, `activeNotes` to `PartitionPainter`; update `shouldRepaint` to include `elapsedMs`/`activeNotes`.
- [x] 2.2 Compute the active measure + fraction; draw a vertical cursor on its system at `measureLeft + fraction × measureWidth`, spanning the system's staves.
- [x] 2.3 Highlight notes whose `[positionDivisions, +durationDivisions)` contains `fraction × divPerMeasure` in the active measure, using the keyboard's expected/correct colors (expected unless pitch ∈ activeNotes → correct).
- [x] 2.4 No-op safely when `measureStartMs` is empty (demo / no score).

## 3. View: look-ahead auto-scroll

- [x] 3.1 Give `_PartitionView` a `ScrollController`; expose the painter's per-system height (const/helper) to compute the cursor system's y.
- [x] 3.2 On playhead change, animate the scroll so the cursor stays visible and the next system is pre-revealed before the current measure ends (lead ≈ one system); skip when everything fits or the target barely moved.
- [x] 3.3 Dispose the controller; guard against scheduling a scroll every tick.

## 4. Tests

- [x] 4.1 Unit-test `measureStartMs` derivation (start at 0; spacing matches divisions×tempo).
- [x] 4.2 Unit-test `measureAt`: position inside a measure → right index + fraction; before start / after end → null.
- [x] 4.3 Widget-test: in Partition mode with a loaded score, advancing the playhead repaints the painter and moves the scroll offset (look-ahead).

## 5. Verify & gate

- [x] 5.1 `cd apps/music && dart run build_runner build --delete-conflicting-outputs`; `melos run analyze`, `dart format`, `dart run custom_lint` clean.
- [x] 5.2 `flutter test --coverage --exclude-tags golden` green and Flutter line coverage ≥ 80%.
- [ ] 5.3 Manually confirm in-app: load a multi-system score, play in Partition mode — cursor advances, current notes light, view scrolls ahead smoothly; Wait Mode freezes the cursor.
- [x] 5.4 `openspec validate partition-playback-cursor --strict` passes.
