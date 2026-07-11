## 1. Scoring core (pure, host-testable)

- [ ] 1.1 Add `state/performance_scoring_core.dart`: mode-adaptive timing-verdict function —
  Wait-Mode-off uses signed score-clock offset (→ perfect/good/early/late/missed);
  Wait-Mode-on uses reaction time from gate-open to attack (→ perfect/good/late, no miss) —
  with tunable ms-window constants for each
- [ ] 1.2 Add sustain-ratio helper (held/intended, clamped 0–1, credit floor, no over-hold
  penalty) to the core
- [ ] 1.3 Add dimension accumulators + `syncPercent` blend (weighted timing/correctness/
  sustain, defined-before-first-judgment baseline) to the core
- [ ] 1.4 Add pure `feedbackTier(syncPercent)` → 0–4 band function to the core
- [ ] 1.5 Unit-test the core: each verdict boundary for **both** timing models (free-tempo
  offset and Wait-Mode reaction time), sustain edges, wrong-note handling, sync% trends
  up/down, tier up/down-crossings (drive coverage on the pure module)

## 2. Session-result model

- [ ] 2.1 Add Freezed `NoteJudgment` (with per-note `waitMode` stamp + `timingOffsetMs` xor
  `reactionMs`), `RunMode { free, wait, mixed }`, and `SessionResult` (with `runMode`,
  `freeSyncPct?`/`waitSyncPct?`, per-mode onset counts) models in `state/session_summary.dart`
  with `toJson`/`fromJson`, per design D6/D10
- [ ] 2.2 Run `build_runner`; unit-test round-trip serialization and aggregate/verdict-count
  derivation, including per-mode sub-score presence/absence

## 3. Scoring notifier wired to the player

- [ ] 3.1 Expose player events the scorer needs (onset-crossed, **gate-open** timestamp for
  Wait Mode, note-on/note-off with timestamps) from `player_notifier.dart` without changing
  Wait Mode / playback
- [ ] 3.2 Add `@riverpod` `PerformanceScorer` notifier consuming those events, holding run
  state (accumulators, recent-hits, combo, per-note judgments)
- [ ] 3.3 Gate run activation on `mode ∈ {synthesia, staff}` (Wait Mode on **or** off; not
  Partition); stamp each judged onset with the live `waitMode` state and pick its timing model
  accordingly; start on play-from-start
- [ ] 3.4 At song end, derive `runMode` (free/wait/mixed) from per-onset counts and compute
  the overall sync% plus per-mode sub-scores (`freeSyncPct`/`waitSyncPct`, absent when that
  mode had no onsets) into the finalized `SessionResult`
- [ ] 3.5 Unit-test the notifier with fake providers: run gating in both Wait-Mode states and
  suppression in Partition; per-onset mode stamping across a mid-run toggle (no reset);
  `runMode` classification (pure free, pure wait, mixed); per-mode sub-score presence;
  wrong-note recording; combo increment/reset; final record produced only for scored runs

## 4. Sync gauge + tiered feedback UI

- [ ] 4.1 Build the sync-gauge widget bound to live sync% and tier, positioned clear of
  notes/hit-line/keyboard; shown only during a scored run
- [ ] 4.2 Add a transient hit-effect layer (spark intensity by verdict) + combo display over
  the Synthesia and horizontal-staff views, suppressible via an effects flag
- [ ] 4.3 Enforce learning-safe constraints: effects never recolor/occlude upcoming notes or
  expected-key highlights; a `missed` verdict never shows a success effect
- [ ] 4.4 Wire gauge + effects into `player_screen.dart` for both scored render modes
- [ ] 4.5 Widget test: gauge visible during a scored run in **both** Wait-Mode states; hidden
  in Partition. Golden (tagged `golden`): notes remain visible under the effects layer

## 5. Session-summary modal + local persistence

- [ ] 5.1 Build the summary modal: overall sync%, per-dimension breakdown, best combo,
  per-verdict counts, and the run classification — for a `mixed` run show both the tempo and
  reaction sub-scores (labelled), for a pure run show the single relevant one; actions replay
  / retry / dismiss
- [ ] 5.2 Show the modal at song end for scored runs only; wire retry (reset piece) and
  dismiss
- [ ] 5.3 Persist the last `SessionResult` as JSON via `PreferencesService` under a
  namespaced key; re-open the last summary; no server transmission
- [ ] 5.4 Widget/unit tests: modal contents from a record, modal not shown for unscored runs,
  persistence via fake preferences (no native storage)

## 6. Mistake replay on the horizontal score

- [ ] 6.1 Add a replay overlay reusing the scrolling-staff painter, highlighting notes by
  verdict (miss / mistimed / poor-sustain / wrong) driven only by `SessionResult` judgments
- [ ] 6.2 Launch replay from the summary modal; scrub a virtual playhead over the record
  (no live input, no audio grading); correct notes render un-flagged
- [ ] 6.3 Widget test: mistakes highlighted, correct notes not flagged, replay independent of
  live input

## 7. Localization

- [ ] 7.1 Add en/fr/it/es strings for the gauge labels, tier/combo copy, summary modal, and
  replay legend via the existing `l10n` flow; regenerate localizations

## 8. Validation & gates

- [ ] 8.1 `dart run build_runner build --delete-conflicting-outputs`; `melos run analyze` +
  `dart format` + `dart run custom_lint` clean
- [ ] 8.2 `flutter test --coverage --exclude-tags golden` green with line coverage ≥ 80%
  (refresh goldens on the pinned platform)
- [ ] 8.3 `openspec validate gamify-piano-practice --strict` passes
