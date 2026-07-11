## Context

The player (`apps/music/lib/state/player_notifier.dart` + `player_data.dart`) drives a
playhead (`elapsedMs`) over a sorted list of `TimedNote`s and receives the player's
key events through `noteOn`/`noteOff` (fed by `midiServiceProvider` or the keyboard
fallback). Today the only feedback loop is **Wait Mode**, which *freezes time* at each
onset until the required pitches are attacked — a pure learning aid with no notion of
"how well" the note was played. Render modes are `synthesia` (waterfall painter),
`staff` (horizontal scrolling grand staff), and `partition` (engraved sheet). There is
no scoring, no accuracy record, and no post-song UI.

Constraints from `CLAUDE.md`: Riverpod 2 + Freezed codegen for all state; dependencies
injected as providers and overridden with fakes in tests (no constructor injection);
pure/testable logic must live in host-testable `*_core`-style modules (mirrors
`midi_core.dart`, `audio_core.dart`); ≥80% line coverage both ecosystems; localized UI
via the existing `l10n` flow (en/fr/it/es). Local prefs go through the injectable
`PreferencesService` seam; secure storage is reserved for auth.

## Goals / Non-Goals

**Goals:**
- A live, deterministic **synchronization percentage** derived from timing, correctness,
  and sustain, computed by a **pure host-testable core** driven by (playhead, note-on,
  note-off) events.
- Tasteful, **learning-safe** Guitar-Hero–style feedback (sync gauge + per-20% tiers +
  hit sparks + combo) layered *additively* over the Synthesia and horizontal-staff
  painters, never altering the notes the player reads.
- An **end-of-song summary modal** with per-dimension breakdown and a **horizontal-score
  replay** that highlights mistakes, driven entirely from a recorded per-note judgment
  list.
- A **serializable session-result record** persisted locally, shaped for the deferred
  server-sync change.

**Non-Goals:**
- Any server transport, upload queue, offline-first sync, or reconnect logic (deferred).
- The **user profile page** listing pieces per instrument (deferred; depends on sync).
- Scoring inside the Partition view (no live play surface). Scoring **does** run in Wait
  Mode — see D2 for the mode-adaptive timing model.
- Rust engine / FFI changes — scoring is entirely Dart-side.
- New third-party dependencies.

## Decisions

### D1 — Scoring is a separate Riverpod notifier fed by the player, not baked into `Player`

A new `PerformanceScorer` notifier (`state/performance_scoring.dart`) holds the run
state; a pure `performance_scoring_core.dart` holds the math. `player_notifier` stays the
single source of playhead + input truth and **forwards events** (onset crossed, note-on
with timestamp, note-off with timestamp) to the scorer; the scorer never drives playback.

- *Why*: keeps Wait Mode / playback requirements untouched (proposal lists no modified
  capabilities), keeps the scoring math free of Flutter/timing side-effects so it is unit
  testable to the 80% bar, and isolates the feedback so it can be suppressed.
- *Alternative rejected*: extending `PlayerData` with score fields — bloats the hottest
  notifier, entangles scoring with the audio/metronome advance loop, and makes the core
  hard to test in isolation.
- *Event source*: rather than polling, `Player.advance` already computes onset crossings
  and `noteOn/noteOff` already have the events; expose them (e.g. a small event sink /
  callback the scorer subscribes to, or the scorer watches `PlayerData` deltas). Prefer an
  explicit event interface over diffing state so timestamps are exact.

### D2 — Mode-adaptive timing model (works in Wait Mode too)

The timing dimension measures a different quantity depending on Wait Mode, because Wait
Mode freezes the playhead at each onset so an absolute-tempo offset is undefined there. The
verdict function is shared (same ordered bands, ms windows); only the input quantity differs:

- **Wait Mode off (free tempo):** target time = `note.startMs / speed` in the run clock; on a
  matching note-on, verdict = f(**signed offset**). Windows (illustrative, tunable core
  constants): |Δ| ≤ 40ms → `perfect`; ≤ 90ms → `good`; ≤ 160ms → `early`/`late` by sign;
  the onset closes `missed` when the playhead passes `onset + missWindow`.
- **Wait Mode on (gated):** when the gate opens on an onset, stamp `gateOpenMs`; on the
  correct attack, verdict = f(**reactionMs = pressMs − gateOpenMs**) against reaction windows
  (e.g. ≤ 120ms → `perfect`; ≤ 300ms → `good`; else `late`). There is no `missed` verdict in
  Wait Mode — the gate blocks until the pitch is pressed — but **wrong presses while the gate
  is open still count against correctness** (D-correctness), so Wait Mode grades accuracy +
  reaction rather than tempo.

In both modes the first qualifying attack binds the onset (reuse the existing "consume the
hold" idea from Wait Mode so a repeat needs a fresh attack); a note-on matching no open onset
= wrong/extra note. The `gateOpenMs` stamp comes from the same `advance`/gate transition
`player_notifier` already computes, so no new timing source is introduced.

- *Why*: one grading pipeline, two input quantities — keeps the core simple and lets the
  gauge/summary/effects work identically in both modes. Reuses the onset/`consumedHeld`
  semantics already proven in `advance`.
- *Speed*: free-run offsets are measured in the **score clock** (divide real Δ by `speed`) so
  50%/200% practice speeds grade fairly; reaction time is real wall-clock (the skill being
  measured is human reaction, independent of playback speed).

### D3 — Sustain: ratio of held time to intended duration, clamped

For a bound onset, track its note-off; `sustainRatio = clamp(heldMs / intendedMs, 0, 1)`
with a lower "credit floor" (e.g. holding ≥85% counts as full) and **no penalty for
over-hold**. Contributes a per-note sustain sub-score.

### D4 — Synchronization percentage = weighted blend of three dimension accumulators

Maintain running accumulators for timing, correctness, sustain; `sync% = 100 * (wT·T +
wC·C + wS·S)` with weights summing to 1 (start ~0.5/0.3/0.2, constants in the core).
Defined before the first judgment (starts at a neutral value, e.g. 100 or a seeded
baseline — chosen so an empty run doesn't read as failure). Percentage is a pure function
of the accumulators → trivially unit-testable.

### D5 — Feedback tier is a pure function of the percentage

`tier(sync%) = (sync% / 20).floor().clamp(0,4)` yields 5 bands. The gauge/effects widgets
watch `tier` for escalation; because it is pure it is tested without rendering. Hit sparks
and combo live in a transient visual layer (an overlay/painter fed by a short-lived
"recent hits" list in scorer state), so they fade and never persist. The activation gate is
on **render mode only** (`mode ∈ {synthesia, staff}`), **not** on Wait Mode — the gauge and
effects show in both Wait-Mode states. Guitar-Hero polish (color by verdict, spark
intensity, combo streak text) is presentation-only.

- *Learning-safe rule (spec `gamified-feedback`)*: feedback widgets render in a layer
  **above** but **outside** the note lanes / expected-key highlights, are gated by an
  "effects on" flag, and read the same verdicts the scorer recorded — a `missed` verdict
  can never map to a success visual.

### D6 — Session-result record: immutable Freezed model, JSON-serializable

`SessionResult { pieceId, title, hands, overallSyncPct, RunMode runMode, freeSyncPct?,
waitSyncPct?, freeOnsetCount, waitOnsetCount, timing/correct/sustain aggregates,
verdictCounts, bestCombo, List<NoteJudgment> notes, playedAtMs, speed }`, with
`NoteJudgment { noteIndex/pitch/startMs, bool waitMode, verdict, timingOffsetMs (when
waitMode false) OR reactionMs (when waitMode true), sustainRatio, wrong }` and
`enum RunMode { free, wait, mixed }`. `freeSyncPct`/`waitSyncPct` are the per-mode
sub-scores (null when that mode had no onsets) that feed the two leaderboards.
Freezed + `toJson/fromJson` so the deferred server change serializes it unchanged. This is
the single object powering the summary modal, the local persistence, and the replay.

### D10 — Per-onset mode stamping + finalize-time classification (handles mid-run toggles)

The scorer stamps every judged onset with `waitMode` = the Player's Wait-Mode state at the
instant of judgment (read from `PlayerData.waitMode`), never a single run-level flag. A
mid-run toggle just changes which metric the next onsets record; nothing resets. At song end
the scorer derives `runMode` (`free` if `waitOnsetCount == 0`, `wait` if `freeOnsetCount ==
0`, else `mixed`) and computes each sub-score over only its subset of onsets. The verdict
ordinal (perfect…missed) is mode-agnostic, so the overall `overallSyncPct` and the two
per-mode sub-scores are all just weighted verdict blends over different onset subsets — one
core function parameterized by which onsets it folds.

- *Why this over disqualifying toggled runs*: never punishes the learner for switching an
  assist mid-piece, keeps every run in the profile, and gives the deferred server change the
  raw material to apply *any* leaderboard policy (route mixed by dominant mode = compare
  `freeOnsetCount` vs `waitOnsetCount`; or exclude mixed = check `runMode == mixed`).
- *Alternative rejected*: resetting/ending the run on toggle — hostile UX and loses data.
- *Alternative rejected*: a single run-level `waitMode` bool — wrong by construction since
  the mode is not constant over the run (the problem the user raised).

### D7 — Replay reuses the real `StaffPainter` with an in-place mistake overlay

The replay renders the **actual** scrolling-staff engraving by reusing `StaffPainter`
(same notes/measures/clefs/armature as play), extended with an optional `mistakeColors`
map (note-index → ring colour) so mistakes are ringed **on the note itself**, in place —
not as an abstract scatter. It scrubs a real playhead (`elapsedMs`) driven by a `Ticker`,
with **synchronized audio** via the same `scoreNoteEdges` seam the player uses, and a
transport (play/pause + seek slider). Below the staff, a tappable mistake list (built from
the judgments, labelled by measure via `measureStartMs`) seeks the playhead to a chosen
note. Grading is **not** re-run — the marks come from the recorded `NoteJudgment`s; the
audio is playback only. The score context (`ReplayScore`) is captured from `PlayerData`
when the run finishes, since the piece is unchanged. A clean run shows a no-mistakes
message instead of an empty list. Correctly-played notes render normally.

The captured `SessionResult.notes` carry `noteIndex` into the run's `visibleNotes` snapshot,
which the replay passes back to `StaffPainter` so the ring lands on the right head.

### D8 — Local persistence via `PreferencesService`, JSON under a namespaced key

Store the last `SessionResult` as JSON under e.g. `lastSessionResult`. Reuses the tested
seam; a fake in tests avoids native storage. No server call in this change. (A future
change swaps in a keyed history / upload queue; the JSON shape is the contract.)

### D9 — Where the pure logic lives (coverage)

`performance_scoring_core.dart` (verdict windows, sustain ratio, sync% blend, tier) is
plain Dart with no Flutter imports → covered by unit tests. The notifier, widgets, painters,
and modal are thin. The gauge/effects painters follow the golden-test pattern (tagged
`golden`, excluded from the cross-platform gate).

## Risks / Trade-offs

- **Grading fairness at high speed or with MIDI latency** → windows are in score-clock ms
  and tunable constants; ship sensible defaults and validate on a real device before
  tightening. Do not couple windows to frame rate.
- **Effects distract or occlude the learning surface** → hard rule D5 (render outside note
  lanes, verdict-consistent, suppressible); covered by explicit `gamified-feedback`
  scenarios and a golden that asserts notes remain visible.
- **Scope creep toward server sync / profile** → firmly deferred; only the *serializable
  local record* lands here, giving the follow-up a stable contract.
- **Event plumbing from `Player` to scorer risks timing drift** → pass explicit event +
  timestamp rather than diffing `PlayerData`; single ticker remains authoritative.
- **Sync% "feels wrong" for a sparse start** → seed a neutral baseline and weight recent
  judgments; treat exact weights/windows as tunable, not spec-locked.

## Migration Plan

Purely additive, feature-flaggable behind the existing mode/Wait-Mode conditions:
1. Land the pure core + models + tests (no UI wired) — safe, invisible.
2. Wire the scorer notifier to `Player` events; add the gauge + effects layer to Synthesia
   and staff; gate on `!waitMode && mode ∈ {synthesia, staff}`.
3. Add the summary modal + local persistence + replay overlay.
4. Add l10n strings (en/fr/it/es).
Rollback = the gate flag; nothing changes Wait Mode / playback so disabling scoring
restores today's behavior exactly.

## Open Questions

- Exact tolerance windows, sustain floor, and dimension weights — pick defaults now, tune
  on-device; expose as core constants, not spec requirements.
- Sync-gauge visual form (radial vs. linear vs. combo-integrated) — resolve during UI, keep
  the "does not occlude" constraint.
- Should the neutral starting sync% be 100 (drops on mistakes) or 0 (builds up)? Leaning
  100-with-decay for a more encouraging feel; confirm during UX review.
- Whether wrong/extra notes should mute their audio or sound normally — default: sound
  normally (matches current free-play), revisit if it feels bad.
