# Design — in-game measure navigation & selection

## Context

The transport bar (`_TransportBar`, `apps/music/lib/screens/player_screen.dart`) holds
restart / play-pause / speed / Wait Mode. Restart calls `restartFromTop()` (jump to the
effective start + get-ready countdown). The measure-range practice capability
(in-flight change `add-measure-range-practice`) provides everything below the UI:

- `setPracticeRange(start, end)` / `clearPracticeRange()` — the single mutation points;
  a narrower range makes the run **selective = unscored by construction**
  (`_maybeStartRun` refuses selective runs and any start that is not the top).
- Per-score persistence of the range (`practice_settings_store.dart`, D7).
- `PartitionPainter` already emits per-measure `hitRects` (content coordinates),
  `measureAtPosition` resolves a tap, and the painter tints an active practice range.
- `PlayerData` has `measureStartMs`, `measureAt(elapsedMs)`, `measureCount`,
  `practiceStartMeasure/EndMeasure`, `startMs` (trimmed effective start).

Today the range is chosen in the pre-play setup modal (steppers + horizontal score
strip) or by tapping the in-game Partition (immediate apply on second tap, Partition
render mode only — hidden on phones).

## Goals / Non-Goals

**Goals:**
- One-tap "go back a bar" from the transport bar, stackable to the effective start,
  usable mid-play without breaking leaderboard integrity.
- A deliberate, full-screen measure-selection mode reachable from the same button
  (long-press), working in **every** render mode and on phones, that never mutates the
  live session until confirmed.
- The in-game mode becomes the **single deliberate selection surface**: the pre-play
  setup modal's practice section/step is removed and the modal becomes range-neutral.

**Non-Goals:**
- Changing the existing immediate tap-select inside the in-game Partition view.
- Reworking the end-of-run summary's "practice this section" dialog (kept; it shares
  `PracticeRangeControls`/`PracticeScoreStrip`). Routing it to the new mode is a
  possible follow-up, not this change.
- Any backend/back-office work; practice-session accounting is untouched.

## Decisions

### D1 — A second transport button, not an overload of restart

The rewind control is a **new button** next to the existing restart button (icon:
`Icons.replay` family / "−1 bar" affordance; restart keeps `Icons.skip_previous`).
Tap = rewind one measure; long-press = enter selection mode.

*Alternative rejected*: merging onto the restart button (tap = −1, restart via
stacking). Full restart is a frequent action; making it N taps on a long piece is
worse, and a tap/long-press/double-tap triple overload is undiscoverable.

### D2 — Rewind semantics (audio-player convention)

Let `m` = measure containing the playhead. A tap moves the playhead to the **start of
`m`** when the playhead is more than a small epsilon (~300 ms of song time) past that
start; otherwise to the **start of `m − 1`**. Repeated taps therefore stack naturally
(after the first tap the playhead sits at a measure start). Clamped at the **effective
start** — `practiceStartMeasure` when a range is active, else the piece's trimmed
start (`startMs`).

Each rewind: silences held voices, resets the Wait-Mode gate state
(`gateSatisfied`/`consumedHeld`), clears any countdown, **cancels the in-flight scored
run** (`_scorer.cancelRun()`), and preserves the playing/paused state — no countdown
replay, the point is a quick "again from the top of the bar". Integrity holds by
construction: after a mid-piece rewind `_maybeStartRun` will not re-arm (not at the
top), so a rewound run can never submit a score; restart-from-top re-arms as before.

New notifier method `rewindOneMeasure()` beside `restart()`; no-op when the piece has
no measure table (demo score) — the button renders disabled then.

### D3 — Selection mode is a route pushed over the player screen

A full-screen `MaterialPageRoute` (new widget, e.g. `MeasureSelectScreen`) — not a
render mode, not a dialog. The player screen stays in the tree underneath, so the
auto-dispose `playerProvider` (and the MIDI/audio session) stays alive. **Entering
pauses playback** (`setPlaying(false)`) — the only session mutation before confirm;
browsing/drafting changes nothing else.

*Alternative rejected*: an in-place overlay/mode inside the player screen — more state
tangled into an already 66 KB screen, and the keyboard/transport would need explicit
hiding; a route gives "no piano, own title bar" for free.

### D4 — The selection view reuses the Partition engraving verbatim

The new screen embeds the same layout + `PartitionPainter` pipeline as
`_PartitionView`, vertical scroll, but: no playhead cursor, no auto-scroll, no
dimming, and the practice-range tint is fed from **local draft state** instead of
`PlayerData`. Tap resolution reuses `hitRects` + `measureAtPosition`. Draft gestures
mirror the existing flow: first tap = start, second tap = end (order-normalized),
re-tap begins a new draft. With the keyboard absent the engraving gets the full
viewport height, which is what makes it workable on landscape-locked phones where the
in-game Partition segment is hidden.

Scores without notation or without a measure table (demo) cannot open the mode; the
long-press does nothing there (same disabled state as D2).

### D5 — Title-bar contract: draft is local until confirmed

The title bar shows the draft ("Mesures X–Y", l10n) and three actions:
- **Confirm** (enabled once a complete draft exists) → `setPracticeRange(start, end)`
  then pop. All existing semantics follow: selective unscored run, playhead to range
  start, per-score persistence.
- **Whole piece** → `clearPracticeRange()` then pop (back to a scored full run).
- **Cancel / back** → pop, session untouched (still paused where it was).

Entering pre-fills the draft from the active range when the run is already selective,
else from the per-score saved practice settings (see D6), else empty.

### D6 — The pre-play modal loses its practice step and goes range-neutral

`pre_play_setup_modal.dart` drops the full-run vs section toggle (`_practiceSection`),
the second step (`_practiceStep`/`_practiceStepBody`), and the apply/pre-fill logic
that called `setPracticeRange`/`clearPracticeRange` on dismiss. The modal **never
touches the active range again** — an armed range survives opening and dismissing it.

What this preserves and how:
- **D7 per-score persistence** ("pre-filled when the score is reopened") relocates:
  the selection mode's draft pre-fills from `practice_settings_store` when no range
  is active. The store and its save/clear sites in the notifier are unchanged.
- **Loop settings**: none to rehome — the implementation settled on "a selective run
  always loops endlessly", so the step carried no loop UI.
- The end-of-run summary dialog (`showPracticeRangePicker`) keeps working — it owns
  its widgets (`practice_range_picker.dart` + shared controls) and does not go
  through the modal.

*Alternative rejected*: keeping the modal step alongside the new mode — two
deliberate surfaces for the same choice, double maintenance, and the modal step was
the pain point that motivated this change.

### D7 — Discovery: a final guided-tour step, not a new hint

The existing player guided tour (`PlayerCoachStep`: pianoSound → midiDevice → hands,
one shared coaching controller) gains a **last step** `measureRewind`, anchored on
the transport rewind button via a new `CoachAnchor` (registered with `CoachTarget`
on the button). One copy teaches both gestures: tap = back one bar, long-press =
pick a passage to practice. Per the onboarding rules a coach step installs **no
input barrier**, and when the anchor is not mounted (e.g. the setup surface still
covers the screen) the overlay's existing fallback — an untargeted centered bubble —
still delivers the copy. Being a tour step makes it **replayable from the help/tips
screen for free** (the replay re-runs the whole sequence), which is what keeps the
possibility re-findable.

*Alternative rejected*: a standalone one-time `CoachHint` + help topic — a hint
cannot point at the control "in place" as well as the tour does, and the tour is the
established home for "here is where to tap" player controls.

## Risks / Trade-offs

- [Rewind silently un-scores the current run] → Same rule the capability already
  established for ranges (unscored by construction); restart-from-top re-arms a scored
  run. No summary appears for a rewound run — consistent with selective runs.
- [Long-press discoverability] → tooltip on the button + the guided-tour step (D7)
  teaching both gestures, replayable from help.
- [Desktop/mouse long-press feels unusual] → Flutter's long-press fires on mouse
  press-and-hold; acceptable. A context-menu alternative can come later if needed.
- [Accidental rewind taps mid-performance] → cheap to recover (playback continues);
  the destructive action (scored-run cancel) is exactly what the player asked for by
  rewinding.
- [Route on top of a live session] → notifier is auto-dispose but keep-alive is
  guaranteed by the underlying screen staying mounted; entering pauses playback so no
  audio runs behind the picker.
- [Removing the modal step strands users who knew it] → the long-press entry is the
  replacement, and the guided-tour step (D7) teaches it — including on replay from
  help for existing users. The summary-dialog path (drill what you just missed) is
  untouched.
- [Archive ordering] → the `pre-play-setup` REMOVED delta targets a requirement the
  in-flight parent change adds; archive `add-measure-range-practice` first.

## Migration Plan

Pure additive UI in `apps/music`; no data, schema, or API change. Ship behind nothing
(no flag needed — low blast radius); rollback = revert the commit.

## Open Questions

- ~~Should a rewind in free-run (non-Wait) replay a 1-beat pre-roll instead of
  resuming instantly?~~ **Resolved (2026-08-14)**: instant resume validated on
  device (macOS, Android tablet, iPhone) — no pre-roll.
