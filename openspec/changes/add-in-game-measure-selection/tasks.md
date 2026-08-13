# Tasks — add-in-game-measure-selection

## 1. Transport rewind (notifier)

- [ ] 1.1 Add a pure `rewindTargetMs(...)` helper next to `normalizePracticeRange`
      (epsilon rule: >~300 ms into measure `m` → start of `m`, else start of `m−1`;
      clamped to the effective start — range start on a selective run, else `startMs`)
      so the arithmetic is host-testable without a notifier
- [ ] 1.2 Add `rewindOneMeasure()` to `player_notifier.dart`: apply the helper,
      silence held voices, reset `gateSatisfied`/`consumedHeld`, clear countdown,
      `_scorer.cancelRun()`, preserve `isPlaying`; no-op when `measureStartMs` is
      empty (demo score)
- [ ] 1.3 Unit tests (mockito per `flutter-testing`): epsilon rule both branches,
      stacking taps, clamp at effective start (full + selective run), scored run
      discarded and never re-armed mid-piece, wait-gate latches cleared, playing
      state preserved, no-op without a measure table

## 2. Transport-bar button

- [ ] 2.1 Add the measure-rewind button to `_TransportBar` (both `Axis` layouts),
      tap → `notifier.rewindOneMeasure()`, disabled (greyed) when the piece has no
      measure table; keep the restart button unchanged; tooltip + l10n label
- [ ] 2.2 Widget tests: tap invokes the notifier action, disabled state without a
      measure table, restart button untouched

## 3. Measure-selection screen

- [ ] 3.1 New `MeasureSelectScreen` (own file under `lib/screens/`): vertical
      scrolling Partition reusing the notation layout + `PartitionPainter` with
      `hitRects`, no playhead/dimming/auto-scroll, range tint driven by **local
      draft state**; taps resolved via `measureAtPosition` (first = start,
      second = end order-normalized, re-tap restarts)
- [ ] 3.2 Title bar: draft label ("Mesures X–Y"), Confirm (enabled on complete
      draft) → `setPracticeRange` + pop, Whole-piece → `clearPracticeRange` + pop,
      Cancel/back → pop with no session mutation; draft pre-fill precedence:
      active range → per-score saved settings (`practice_settings_store`, clamped)
      → empty
- [ ] 3.3 Wire the long-press on the rewind button: `setPlaying(false)` then push
      the route; guard — do nothing when the score has no notation document or no
      measure table
- [ ] 3.4 l10n: new keys (button tooltip, screen title, draft label, confirm /
      whole-piece / cancel) in the ARB files + regen
- [ ] 3.5 Widget tests: long-press pauses and opens the screen (and is a no-op on
      the demo score), two-tap draft normalization + highlight, re-tap restarts,
      confirm applies the range, cancel leaves `PlayerData` untouched, whole-piece
      clears the range

## 4. Pre-play modal: remove the practice step

- [ ] 4.1 Strip `pre_play_setup_modal.dart`: `_practiceSection` (run-type toggle,
      key `practice-run-type`), `_practiceStep`/`_practiceStepBody` (key
      `practice-step-back`), the `_selective`/`_fromMeasure`/`_toMeasure` draft
      state, `_prefillFromSaved`, and the `setPracticeRange`/`clearPracticeRange`
      calls in the apply path — the modal never touches the range again
- [ ] 4.2 Verify an armed range survives opening + dismissing the modal (in-game
      entry from the settings drawer included); keep `PracticeRangeControls` /
      `PracticeScoreStrip` (still used by the summary's `practice_range_picker.dart`)
- [ ] 4.3 Update/remove the modal's practice widget tests; prune l10n keys that
      became modal-only dead strings (keep those shared with the summary dialog)

## 5. Gates & validation

- [ ] 5.1 `melos run analyze`, `dart format`, `dart run custom_lint` clean
- [ ] 5.2 `flutter test --coverage --exclude-tags golden` green, line coverage ≥ 80%
- [ ] 5.3 `openspec validate add-in-game-measure-selection --strict` passes
- [ ] 5.4 On-device feel pass (iPad + phone landscape): rewind epsilon feel, decide
      the open question on a free-run pre-roll after rewind
