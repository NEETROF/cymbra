## Context

Playback timing is **100% Dart-driven** — the Rust side is a stateless synth (no timeline,
no sequencer, no seek). The playhead is a single `PlayerData.elapsedMs` advanced frame-by-frame
by a `Ticker` calling `Player.advance(dt)`. Measure→time already exists as `measureStartMs`
(`player_data.dart:209`), and **free playback already loops** back to `startMs` today
(`player_notifier.dart:497`) — the loop-wrap even silences held notes via `_silenceAll()`.

Scoring is a separate, mature system: a scored run is armed by `_maybeStartRun` only when
`visibleNotes.isNotEmpty && _atStart(s)` (`player_notifier.dart:116`), captures the **whole
piece's** notes for the selected hand(s), tallies a blended sync% + combo + per-note verdicts,
and at `endMs` produces a `SessionResult` that is shown in a modal, persisted locally, and
uploaded (`recordPlaySession`) to feed leaderboards and the profile activity heatmap. Crucially,
`finishRun` marks every un-played captured note as `missed` (`performance_scoring.dart:272`),
and looping vs scoring are already **mutually exclusive** — an active scored run finishes into
a summary, only a cancelled (unscored) run loops.

This is why a measure subrange cannot naively reuse the scored path: it would either upload a
~5% grade or, if scored over the subrange, be trivially farmable. The design keeps practice
**entirely on the existing unscored loop path** and lets the scorer stay idle.

## Goals / Non-Goals

**Goals:**
- One primitive — an active measure range on a run — that expresses both "play these measures"
  (stop at end) and "loop these measures" (wrap at end).
- A **selective run is unscored**: no scored-run activation, no sync%, no leaderboard, no
  `SessionResult` upload — with zero risk of corrupting existing scoring stats.
- Practice still **counts as activity** on the profile (a scoreless per-day practice count).
- Loop the range with a **loop count** (finite N / infinite) and a **practice tempo ramp**.
- Pick the range **two ways**: setup-sheet steppers (all render modes) and tap-on-score
  (Partition view).
- **Persist** the range + loop settings **per score**.
- No Rust/synth change; pure range/loop/gate logic stays host-testable; coverage ≥ 80%.

**Non-Goals:**
- Scoring, grading, or ranking partial runs; any "practice leaderboard".
- Sub-measure A/B markers; multiple named sections/presets per score (one saved selection only).
- Auto-detecting hard passages; adaptive difficulty.
- Any change to the scoring math itself.

## Decisions

### D1 — One range primitive; "full run" is the default range
`PlayerData` gains a practice range (measure indices), defaulting to the whole piece. The
effective `startMs`/`endMs` become **range-aware**: a range `[a, b]` maps via
`measureStartMs[a]` … `measureStartMs[b+1]` (or `songEndMs`). A run whose range is the whole
piece is a **full run** and behaves *exactly* as today (no regression); any narrower range is a
**selective (practice) run**. Rationale: "play a section" and "loop a section" are the same
`[start,end]` with a stop-vs-wrap flag — modeling them as two features would double the UI and
state. Alternative rejected: separate `trimRange` and `loopRange` fields (redundant, ambiguous
when both set).

### D2 — Selective run = the existing unscored loop path; the scorer never arms
The run-start guard (`_maybeStartRun`) SHALL NOT arm a scored run when the run is selective;
the selective path uses the current `cancelRun`/loop-wrap behavior. Rationale: reuses a proven
seam (including `_silenceAll()`), and keeps scoring integrity by construction — there is no
partial `SessionResult` to suppress because none is produced. Alternative rejected: score the
subrange then flag/exclude server-side — needs a proto flag + server anti-farm logic and still
risks leaking into stats. The user's decision ("practice must not be scored") makes this moot.

### D3 — Loop count + tempo ramp ride on `advance`'s wrap
The loop-wrap in `advance` targets the **range start** instead of the piece start. A finite
loop count decrements each wrap and stops (silence + pause) at zero; infinite never stops until
the user acts. The **tempo ramp** increments `speed` by a chosen step at each wrap, **clamped to
the existing max speed (2.0)** and never below the start speed. Laps are continuous — **no
per-lap modal**. Rationale: the wrap is the one existing loop seam; ramp/count are small
counters on it. Alternative rejected: a per-lap summary modal (jarring for drilling).

### D4 — Scoreless practice-activity record (the one bridge-crossing part)
A completed **practice session** is captured as a scoreless activity record — client session id,
score id, timestamp/timezone — through the **same durable outbox + idempotent delivery** as
scored sessions, but carrying **no `SessionResult`/sync%**. Server aggregation and
`getPlayActivity` return a **per-day practice count** distinct from the scored-play count.
Chosen shape: a dedicated `recordPractice`-style RPC (cleaner than overloading
`recordPlaySession` with a nullable score). **Recorded once per practice session on stop/exit,
not per lap**, so looping does not inflate the count. Rationale: reuses the no-loss/idempotent
guarantees; keeps practice out of the scoring tables entirely. Alternative rejected: a
`kind` enum on `recordPlaySession` (couples practice to the scored schema/validation).

### D5 — Two range pickers behind one range setter
Both pickers set the same `Player.setPracticeRange(a, b)`:
- **Steppers** in the pre-play setup sheet — from/to measure, clamped to `[0, measureCount-1]`,
  `a ≤ b`. Works in all three render modes; fully widget-testable.
- **Tap-on-score** — `PartitionPainter` already computes per-measure `measureX`/`measureWidth`
  (`partition_painter.dart:388`); expose per-measure hit rects and wrap the score `CustomPaint`
  in a gesture layer (first tap = start, second = end, tap-again to reset). Partition mode only;
  other modes rely on the steppers.
Rationale: one controller method, two input surfaces; the tap layer is additive and degrades
gracefully. Alternative rejected: a draggable scrubber/range slider (imprecise at measure
granularity on small screens).

### D6 — Entry points reuse existing modals
- **`pre-play-setup`** gains a full-vs-selective choice + the range/loop picker; the choice is
  applied to the session that begins.
- **`session-summary`** gains a "practice this section" action that opens the picker for a
  selective run, next to see-mistakes/retry/quit.
Rationale: these are already the "start a game" and "after a run" moments the user named.

### D7 — Persist per score via the local-preferences seam
The range + loop settings persist keyed by **score id** (reusing the `local-preferences`/store
seam used by `session_summary_store`). On reopen, the saved selection pre-fills the picker.
**Clamp on load** to the current measure count (a re-imported score may have changed), falling
back to the whole piece if invalid. Rationale: cheap, offline, no backend. Alternative rejected:
server-side per-user-per-score settings (out of proportion for v1).

## Risks / Trade-offs

- **Regression on full runs** → the whole-piece range MUST resolve to today's exact
  `startMs`/`endMs` and scored-run activation. Mitigation: treat "range == whole piece" as the
  literal current path; golden/unit test that a default run still scores and uploads identically.
- **Wait Mode + loop at the range boundary** → each lap must re-gate cleanly at the range start.
  Mitigation: reuse `_silenceAll()` on wrap; the gate keys off onsets at `elapsedMs`, which is
  reset to the range start.
- **Tempo ramp runaway** → clamp to `[startSpeed, 2.0]`; ramp only applies to selective runs.
- **Inflated activity from looping** → record the practice-activity event **once per session on
  stop**, not per lap; idempotent by client session id so a retry is a no-op.
- **Heatmap ambiguity** → a day with *only* practice has no sync%, so it MUST render distinctly
  from a failed scored day (not as 0%/red). Mitigation: practice count drives intensity/tooltip
  only, never the success color; a practice-only day is a neutral "active" cell.
- **Stale saved range** after a score is re-imported with a different measure count → clamp/reset
  on load (D7).
- **Scope size** → six touched capabilities. Mitigation: phased tasks (below); Phase 1 ships a
  usable practice loop with no backend change.

## Migration Plan

No data migration. Additive proto/gRPC (`recordPractice` + practice-count field on the activity
aggregate) — old clients simply never call it; the heatmap tolerates a zero practice count.
Rollout is app-forward-compatible; the backend change deploys before the app feature is enabled.

**Phasing (each phase independently shippable):**
1. **Primitive** — range state + range-aware bounds; selective run stays unscored (scorer gate);
   loop-wrap to range start; setup-sheet steppers + full/selective choice. *No backend.*
2. **Activity** — scoreless practice capture → outbox → `recordPractice` → aggregation →
   heatmap practice count. *Backend + proto.*
3. **Extras & polish** — loop count, tempo ramp, tap-on-score picker, per-score persistence,
   summary "practice this section" action.

## Open Questions

- **Loop-count UI granularity** — expose ∞ + a small preset set (e.g. 2/4/8/∞) or a free numeric
  stepper? (Proposed: presets + ∞ for v1.)
- **Practice-activity threshold** — does a practice session count as activity only after it
  actually plays some notes (avoid a "started then quit" inflating the count)? (Proposed: count
  only if at least one lap/onset elapsed.)
- **Where the range readout lives during play** — a compact badge in the transport bar vs the
  setup sheet only. (Proposed: a small "bars a–b · loop" badge in the transport bar.)
