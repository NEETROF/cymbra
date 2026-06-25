## 1. State: onset helpers

- [x] 1.1 Add `Set<int> onsetNotesAt(double t)` to `PlayerData` returning pitches whose `startMs` equals the current onset (reuse the ±1ms tolerance).
- [x] 1.2 Add `int? nextOnset(double t)` returning the first note start strictly greater than `t` (distinct onsets only).
- [x] 1.3 Keep the window-based `requiredNotesAt` only if still needed; otherwise mark it superseded by the onset set (note any callers, e.g. the keyboard painter feedback).

## 2. Notifier: onset gate with press latch

- [x] 2.1 Add gate tracking to `Player`: the active onset and a latched `Set<int> _pressedForGate`.
- [x] 2.2 In `noteOn(pitch)`, when the gate is active and `pitch` is in the active onset set, add it to `_pressedForGate` (validation). Keep updating `activeNotes` for feedback.
- [x] 2.3 Rewrite `advance(dtMs)`: freeze at the current onset while `_pressedForGate` does not yet cover the onset set; when covered, advance and reset the latch for the next onset. Preserve the `blocked` flag toggling.
- [x] 2.4 Enforce "right moment": presses before the playhead reaches an onset must not fill `_pressedForGate` for that onset (only count once the gate is active there).
- [x] 2.5 Reset the latch when the gate advances and when Wait Mode is toggled on mid-piece (anchor to the onset at the current playhead).

## 3. Feedback wiring

- [x] 3.1 Point the keyboard's expected/correct feedback at the active onset set (the same gate), so a validated-and-released note stops showing as expected. No change to the `keyboard-display` Three-State requirement wording.
- [x] 3.2 If the `hand-selection-filter` change is present, intersect the onset set with the visible-hand filter so hidden-hand onsets are neither awaited nor shown. _(N/A — hand-selection-filter not implemented yet; revisit when it lands.)_

## 4. Tests

- [x] 4.1 Press-then-release advances: pressing the required key at the onset and releasing before the next tick keeps the gate released.
- [x] 4.2 Early press does not pre-satisfy: a press before the playhead reaches the onset leaves the gate blocking on arrival.
- [x] 4.3 Chord: gate releases only after all onset pitches are pressed (any order); partial chord keeps blocking.
- [x] 4.4 Wrong/missing key keeps the playhead frozen at the onset.
- [x] 4.5 Wait Mode off advances without gating; toggling Wait Mode on mid-piece anchors the gate at the current onset.
- [x] 4.6 Repeated same pitch at consecutive onsets requires a fresh press at the second onset.

## 5. Verify & gate

- [x] 5.1 `cd apps/music && dart run build_runner build --delete-conflicting-outputs`; `melos run analyze`, `dart format`, `dart run custom_lint` clean.
- [x] 5.2 `flutter test --coverage --exclude-tags golden` green and Flutter line coverage ≥ 80%.
- [x] 5.3 Manually confirm in-app: in Wait Mode, a quick correct tap advances the cascade; holding is not needed; pressing early does not skip the wait.
- [x] 5.4 `openspec validate wait-on-keypress --strict` passes.
