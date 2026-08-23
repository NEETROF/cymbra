## 1. One clock pair, picked per use

- [x] 1.1 Add `ScoreClocks` (emission + heard) with `judgmentClock` / `sustainClock` to
  `performance_scoring_core.dart` — the mode picks the judgment clock, the note's binding
  mode picks the sustain clock.
- [x] 1.2 Assemble the pair in one place: `PlayerData.clocksAt` / `clocks`;
  `judgmentClockAt` delegates to `judgmentClock` so the pinned rule has one home.
- [x] 1.3 Scorer entry points (`noteOn`, `noteOff`, `tick`, `finishRun`) take the pair;
  sustain finalization (`noteOff`, `tick` auto-finalize, `finishRun` stragglers) measures
  each note on `sustainClock`.
- [x] 1.4 Route the five notifier call sites through `state.clocks` / `clocksAt(next)`.

## 2. Finish the run on the judgment clock

- [x] 2.1 Add `PlayerData.scoredRunEndMs` — the first playhead whose judgment clock has
  reached `endMs` (the end itself in Wait Mode and at offset 0).
- [x] 2.2 `advance()` finalizes a scored run at `scoredRunEndMs` instead of `endMs`;
  non-scored loops still wrap at `endMs` on the emission clock.

## 3. Tests

- [x] 3.1 Behavioural repro: free run, offset 300, last onset within the offset of the
  end, attacked in time with the sound — run still active, attack `perfect`, no `missed`
  (fails pre-fix: run already finalized, note `missed`).
- [x] 3.2 Pin: Wait Mode under an offset still finalizes exactly at the piece end.
- [x] 3.3 Behavioural repro: C4 bound at the Wait gate, Wait Mode toggled mid-hold,
  released at its full duration — sustain 1.0 (fails pre-fix: 0.6, short by exactly the
  offset).
- [x] 3.4 Scorer-level pins with split clocks: a Wait-bound hold released in free run
  keeps the emission clock; a free-bound hold is not stretched by a toggle to Wait Mode.
- [x] 3.5 Unit pins: `clocksAt`, and `scoredRunEndMs` in both modes and at offset 0.
- [x] 3.6 `flutter test --exclude-tags golden` (1586 passing), `flutter analyze`,
  `dart run custom_lint`, `dart format` clean.

## 4. Spec

- [x] 4.1 Delta spec: extend `performance-scoring` / Output Offset Applied To The Timing
  Reference (sustain clock, run finalization) on top of the `fix-output-offset-units`
  text.
- [ ] 4.2 Manual check on a real delayed route (Bluetooth): hold a note across a Wait
  Mode toggle; play a piece's final notes in free run.
- [ ] 4.3 Archive after review/merge (`/opsx:archive fix-output-offset-scoring-edges`),
  after `fix-output-offset-units`.
