## Context

The player screen (`apps/music/lib/screens/player_screen.dart`) renders a piece in three
`RenderMode`s — `staff`, `synthesia`, `partition` (`player_data.dart:28`) — driven by the
`Player` Riverpod notifier (`player_notifier.dart`) over `PlayerData` (`player_data.dart`).
The playhead is `PlayerData.elapsedMs` and **always seeds to `0`**. Notes are `List<TimedNote>`
(sorted by `startMs`); rests are a separate render-only `List<TimedRest>` channel that never
enters the playable/scored set. A piece that opens with rests therefore has
`notes.first.startMs > 0` while `elapsedMs` still starts at `0`, so the player watches dead air
(free run) or ticks through leading measures in real time before the Wait-Mode gate (wait mode).

There is no `seek()`, no `startOffset`, and no "first note" concept today. Everything that
starts a run seeds `elapsedMs: 0`, and two guards branch on the playhead being at zero:

- `_applyNotation(...)` → `elapsedMs: 0` (`player_notifier.dart:187`)
- `_loadDemo(...)` → `elapsedMs: 0` (`player_notifier.dart:~218`)
- `restart()` → `elapsedMs: 0` (`player_notifier.dart:389`)
- `setSelectedHands(...)` → `elapsedMs: 0` (`player_notifier.dart:375-380`)
- `advance()` loop wrap-around resets toward the top (`player_notifier.dart:471-474`)
- `startPlayback()` arms the countdown only when `state.elapsedMs == 0` (`player_notifier.dart:324`)
- `_maybeStartRun()` opens a scored run only when `elapsedMs == 0` (guard at `player_notifier.dart:117`)

## Goals / Non-Goals

**Goals:**
- Begin playback a short, bounded lead-in before the first sounding note of the selected
  hand(s), skipping leading rests / empty measures, in all three render modes and both
  Wait-Mode states.
- Apply this to every fresh start: load, restart/Retry, hand-switch restart, loop wrap.
- Keep the get-ready countdown, Wait-Mode gate, scored-run opening, score audio, and metronome
  correct when the playhead starts at a non-zero position.
- Pieces that already start at (or near) time zero behave exactly as today.

**Non-Goals:**
- No trimming of *trailing* silence or of interior rests after the first note.
- No new UI, settings, or persisted preference — the trim is automatic.
- No change to MusicXML parsing, the Rust engine, `TimedNote`/`TimedRest` models, or scoring
  math.
- Not a general `seek()`/scrubbing feature — only the initial start position.

## Decisions

### 1. Effective start = `firstVisibleOnset − leadIn`, clamped to ≥ 0

Add a pure, host-testable helper that computes the start position from the current state:

```dart
// pure function (in player_data.dart or a small helper), so it is unit-testable
double effectiveStartMs(List<TimedNote> visibleNotes, {double leadInMs}) {
  if (visibleNotes.isEmpty) return 0;
  final firstOnset = visibleNotes.map((n) => n.startMs).reduce(min); // notes are sorted → first
  return (firstOnset - leadInMs).clamp(0, firstOnset).toDouble();
}
```

- **Selected-hand scope**: computed from `visibleNotes` (the notes shown/played for the current
  `Hand` selection), not the whole piece. Matches what the player sees and what the scorer
  judges; `setSelectedHands` already restarts, so recomputation is natural. *Alternative — use
  the whole-piece `notes` — rejected*: it would leave dead air for a hand that enters late.
- **Bounded lead-in** (not exactly on the onset): in Synthesia/Staff a note whose `startMs ==
  elapsedMs` sits on the hit line with no fall-in; the lead-in restores the approach animation
  and a beat of pulse while still trimming the bulk of the silence. *Alternative — start exactly
  on the onset — rejected* for the abrupt no-animation entry.
- **Lead-in size**: a single small constant (e.g. `kStartLeadInMs`), on the order of one beat /
  the Synthesia fall-in duration. Kept as a named constant so it is easy to tune; deriving it
  from tempo/look-ahead is possible later but a fixed budget is enough and simplest.

### 2. Replace the zero-seeds and the two zero-guards with the effective start

Introduce it once and reuse everywhere:

- Seed `elapsedMs: effectiveStartMs(...)` in `_applyNotation`, `_loadDemo`, `restart`, and
  `setSelectedHands`.
- Loop wrap-around in `advance()` wraps to the effective start instead of the top.
- `startPlayback()` arms the countdown when the playhead is **at the effective start** (fresh
  start) rather than `== 0`.
- `_maybeStartRun()` opens the run when the playhead is at the effective start rather than `== 0`.

A tiny predicate — `bool _atStart(PlayerData s) => s.elapsedMs <= s.startMs` (using a
`PlayerData.startMs` getter that returns the effective start) — centralises both guards so the
"fresh start" meaning lives in one place. Resume-from-pause is still distinguished because a
paused playhead is `> startMs`.

### 3. Countdown / gate / audio behaviour falls out for free

- **Free run**: `startPlayback()` arms the countdown; `advance()`'s countdown branch already
  returns without advancing, so the playhead simply stays frozen at the (non-zero) effective
  start until GO. After GO it advances through the lead-in and the first note falls in and lands
  on the beat.
- **Wait Mode**: no countdown; the playhead advances the brief lead-in in real time (via the
  existing `nextOnsetAfter` clamp) and freezes at the first onset — the note visibly approaches
  first. Leading rests beyond the lead-in are never played.
- **Score audio** (`scoreNoteEdges`, `from <= startMs < to`): starting *before* the first onset
  means the first note still sounds exactly once when time advances past it — no pre-sound,
  because at the frozen start `from == to`.
- **Metronome / Partition cursor**: seeding a non-zero playhead is a seek; the metronome's
  "seek re-aligns the beat without a spurious tick" requirement already covers it, and the
  Partition cursor derives from `measureAt(elapsedMs)`, so it lands on the first note's measure.

## Risks / Trade-offs

- **The two `== 0` guards are easy to miss** → centralise in `_atStart`/`PlayerData.startMs` and
  cover with unit tests asserting a run opens and a countdown arms at a non-zero start.
- **Anacrusis / pickup measures** → the pickup note *is* the first onset, so only pure leading
  rests are trimmed; the pickup is preserved. No special case needed.
- **A tiny residual lead-in of dead air remains** (bounded, ~1 beat) → intentional, for the
  fall-in animation and pulse; far better than 3–4 empty measures.
- **Loop seam** → wrapping to the effective start (not 0) must not double-fire the first note or
  a metronome tick; reuse the existing seek-safe wrap path and assert with a loop test.
- **Demo score** has `elapsedMs` effectively starting at its first note (usually 0) → behaviour
  unchanged; covered by keeping existing player tests green.

## Migration Plan

Pure additive behaviour change, no data or API migration. If it regresses, the trim is
localised to the start-position seed and two guards, so reverting is a small, self-contained
change. No feature flag planned (automatic, low-risk), but the lead-in constant gives a quick
knob if the entry feels off.

## Open Questions

- Exact `kStartLeadInMs` value (fixed ms vs. one beat derived from tempo). Start with a fixed
  constant tuned against a leading-rest fixture; revisit if the fall-in feels short at very slow
  or very fast tempos.
