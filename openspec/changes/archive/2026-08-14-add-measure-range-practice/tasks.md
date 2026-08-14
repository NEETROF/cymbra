# Tasks

Phased so each phase is independently shippable. Phase 1 delivers a usable, unscored
practice loop with **no backend change**. All work is Dart-side except Phase 2's ingest.

## 1. Phase 1 — Range primitive + unscored selective run (no backend)

- [x] 1.1 Add practice-range fields to `PlayerData` (Freezed): `practiceStartMeasure`,
  `practiceEndMeasure` (nullable/whole-piece default), plus a derived `isSelectiveRun` getter
  (`true` when the range is narrower than the whole piece). Run build_runner.
- [x] 1.2 Make effective bounds range-aware: derive `startMs`/`endMs` from the range via
  `measureStartMs[start]` … `measureStartMs[end+1]` (or `songEndMs`), keeping the whole-piece
  case byte-for-byte identical to today. Add a pure `measureEndMs(i)` helper.
- [x] 1.3 Add `Player.setPracticeRange(int start, int end)` and `clearPracticeRange()` that
  normalize/clamp to `[0, lastMeasure]` (`start ≤ end`), reset `elapsedMs` to the new range
  start, and call `_silenceAll()`.
- [x] 1.4 Gate the scorer: in `_maybeStartRun`/`_atStart`, do **not** arm a scored run when
  `isSelectiveRun`; a selective run follows the existing unscored (`cancelRun`) path.
- [x] 1.5 Range-aware loop-wrap: in `advance`, target the **range start** (not piece start) on
  wrap for a selective run; a selective run with looping off stops at the range end.
- [x] 1.6 Add from/to measure **steppers** + a full-vs-selective toggle to the pre-play setup
  modal; wire to `setPracticeRange`/`clearPracticeRange`. Apply on dismiss.
- [x] 1.7 Add a compact "bars a–b · loop" readout badge to the transport bar during a selective
  run (see design Open Question — transport badge).
- [x] 1.8 Unit tests (host): range normalization/clamp, effective start/end mapping, whole-piece
  == today (no regression), selective run does not arm scoring, wrap targets range start.
- [x] 1.9 Widget tests: setup-modal full/selective toggle + steppers set the range; transport
  badge reflects the range. Run `dart run custom_lint`; keep coverage ≥ 80%.

## 2. Phase 2 — Practice as scoreless activity (backend + proto)

- [x] 2.1 Proto/gRPC: add a `recordPractice`-style RPC (client session id, score id, timestamp
  + timezone; no score) and a per-day **practice count** field on the play-activity aggregate
  returned by `getPlayActivity`. Regenerate Dart + Rust from proto.
- [x] 2.2 Backend: implement `recordPractice` ingest — idempotent by client session id, stored
  separately from scored sessions; aggregate per-day practice counts. Rust tests
  (mockall for repo/port seams); coverage ≥ 80%.
- [x] 2.3 App capture: at practice-session end, enqueue a **scoreless** activity record into the
  durable outbox (reuse `play_sync_notifier`/`play_sync_service` seam), delivered idempotently.
  Record **once per session** on stop/exit, not per lap; only if ≥ one lap/onset elapsed
  (design Open Question — threshold).
- [x] 2.4 Heatmap: surface the per-day practice count distinct from scored plays; a
  practice-only day renders as a neutral active cell (never a failure color); count in
  intensity/tooltip.
- [x] 2.5 Tests: app — practice capture enqueues scoreless record, once-per-session, survives
  restart, idempotent retry (mockito). Widget — heatmap practice-only day is neutral and shows
  the practice count. Coverage ≥ 80% both ecosystems.

## 3. Phase 3 — Loop controls, tap-to-select, persistence, summary entry

- [x] 3.1 Loop count: add `loopCount` (finite `N` / infinite) to state + `advance` (decrement
  per wrap, stop + silence at zero). Add loop-count control (presets 2/4/8/∞) to the setup sheet.
- [x] 3.2 Practice tempo ramp: add `tempoRampStep` to state; increment `speed` at each wrap,
  clamped to `[startSpeed, maxSpeed]`, selective-only. Add the ramp control to the setup sheet.
- [x] 3.3 Tap-to-select on the Partition view: expose per-measure hit rects from
  `PartitionPainter` (reuse `measureX`/`measureWidth`); wrap the score `CustomPaint` in a
  gesture layer (first tap = start, second = end, tap-again resets) and highlight the selection.
- [x] 3.4 Per-score persistence: save/load the range + loop settings keyed by score id via the
  `local-preferences`/store seam; clamp on load to the current measure count (fallback whole
  piece); pre-fill the setup sheet.
- [x] 3.5 Summary entry: add a "practice this section" action to the session-summary modal that
  opens the range picker and starts a selective run; keep actions reachable on short viewports.
- [x] 3.6 Tests (host + widget): loop-count stop-after-N, ramp clamp, ramp not applied to full
  runs, tap two measures sets range + re-tap resets, saved settings pre-fill + stale-range clamp,
  summary "practice this section" opens the picker. `dart run custom_lint`; coverage ≥ 80%.

## 4. Validation & wrap-up

- [x] 4.1 `openspec validate add-measure-range-practice --strict` passes.
- [x] 4.2 `melos run analyze` + `dart format` clean; `cargo fmt --all --check` +
  `cargo clippy --workspace --all-targets -- -D warnings` clean.
- [x] 4.3 Full test + coverage gate green (Rust `cargo llvm-cov` ≥ 80%; Flutter
  `flutter test --coverage` + `very_good_coverage`). Regenerate frb if the Rust public API
  changed (Phase 2).
- [ ] 4.4 Manual/integration sanity: full run still scores + uploads identically; selective run
  loops, ramps, is unscored, and shows as a practice on the profile heatmap.
