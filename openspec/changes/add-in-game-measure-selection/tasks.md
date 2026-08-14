# Tasks — add-in-game-measure-selection

## 1. Transport rewind (notifier)

- [x] 1.1 Add a pure `rewindTargetMs(...)` helper next to `normalizePracticeRange`
      (epsilon rule: >~300 ms into measure `m` → start of `m`, else start of `m−1`;
      clamped to the effective start — range start on a selective run, else `startMs`)
      so the arithmetic is host-testable without a notifier
- [x] 1.2 Add `rewindOneMeasure()` to `player_notifier.dart`: apply the helper,
      silence held voices, reset `gateSatisfied`/`consumedHeld`, clear countdown,
      `_scorer.cancelRun()`, preserve `isPlaying`; no-op when `measureStartMs` is
      empty (demo score)
- [x] 1.3 Unit tests (mockito per `flutter-testing`): epsilon rule both branches,
      stacking taps, clamp at effective start (full + selective run), scored run
      discarded and never re-armed mid-piece, wait-gate latches cleared, playing
      state preserved, no-op without a measure table

## 2. Transport-bar button

- [x] 2.1 Add the measure-rewind button to `_TransportBar` (both `Axis` layouts),
      tap → `notifier.rewindOneMeasure()`, disabled (greyed) when the piece has no
      measure table; keep the restart button unchanged; tooltip + l10n label
- [x] 2.2 Widget tests: tap invokes the notifier action, disabled state without a
      measure table, restart button untouched

## 3. Measure-selection screen

- [x] 3.1 New `MeasureSelectScreen` (own file under `lib/screens/`): vertical
      scrolling Partition reusing the notation layout + `PartitionPainter` with
      `hitRects`, no playhead/dimming/auto-scroll, range tint driven by **local
      draft state**; taps resolved via `measureAtPosition` (first = start,
      second = end order-normalized, re-tap restarts)
- [x] 3.2 Title bar: draft label ("Mesures X–Y"), Confirm (enabled on complete
      draft) → `setPracticeRange` + pop, Whole-piece → `clearPracticeRange` + pop,
      Cancel/back → pop with no session mutation; draft pre-fill precedence:
      active range → per-score saved settings (`practice_settings_store`, clamped)
      → empty
- [x] 3.3 Wire the long-press on the rewind button: `setPlaying(false)` then push
      the route; guard — do nothing when the score has no notation document or no
      measure table
- [x] 3.4 l10n: new keys (button tooltip, screen title, draft label, confirm /
      whole-piece / cancel) in the ARB files + regen
- [x] 3.5 Widget tests: long-press pauses and opens the screen (and is a no-op on
      the demo score), two-tap draft normalization + highlight, re-tap restarts,
      confirm applies the range, cancel leaves `PlayerData` untouched, whole-piece
      clears the range

## 4. Pre-play modal: remove the practice step

- [x] 4.1 Strip `pre_play_setup_modal.dart`: `_practiceSection` (run-type toggle,
      key `practice-run-type`), `_practiceStep`/`_practiceStepBody` (key
      `practice-step-back`), the `_selective`/`_fromMeasure`/`_toMeasure` draft
      state, `_prefillFromSaved`, and the `setPracticeRange`/`clearPracticeRange`
      calls in the apply path — the modal never touches the range again
- [x] 4.2 Verify an armed range survives opening + dismissing the modal (in-game
      entry from the settings drawer included); keep `PracticeRangeControls` /
      `PracticeScoreStrip` (still used by the summary's `practice_range_picker.dart`)
- [x] 4.3 Update/remove the modal's practice widget tests; prune l10n keys that
      became modal-only dead strings (keep those shared with the summary dialog)

## 5. Guided-tour step (help tutorial)

- [x] 5.1 Add `PlayerCoachStep.measureRewind` (last step) in
      `coaching_notifier.dart` and `CoachAnchor.measureRewind` in
      `coach_mark.dart`; register a `CoachTarget` on the transport rewind button
- [x] 5.2 Step copy in `coach_copy.dart` + l10n keys: one bubble teaching tap =
      back one bar, long-press = pick a passage; no input barrier (onboarding
      rule), untargeted centered-bubble fallback when the anchor is not mounted
- [x] 5.3 Tests: coaching notifier walks …hands → measureRewind → done; widget
      test that the tour step renders and that help replay re-runs the sequence
      including the new step

## 6. Gates & validation

- [x] 6.1 `melos run analyze`, `dart format`, `dart run custom_lint` clean
- [x] 6.2 `flutter test --coverage --exclude-tags golden` green, line coverage ≥ 80%
- [x] 6.3 `openspec validate add-in-game-measure-selection --strict` passes
- [x] 6.4 On-device feel pass (iPad + phone landscape): rewind epsilon feel, decide
      the open question on a free-run pre-roll after rewind — validated on macOS,
      Android tablet (SM-P610) and iPhone (2026-08-14); instant resume kept, no
      pre-roll
