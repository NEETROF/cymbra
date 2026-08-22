## Why

`fix-output-offset-units` (PR #269) split the judgment clock by mode and listed two
defects as still open. Both are real, both were reproduced against the shipping code
before fixing, and neither needs the drain design the unit error is waiting on:

**1. A note held across a mid-hold Wait Mode toggle loses a whole offset of sustain.**
The scorer received ONE pre-picked clock per call — the current mode's judgment clock.
`noteOff` therefore measured `held = clock - note.startMs` on whatever mode was active at
*release* time, while the note had been bound on the other mode's clock. A note bound at
the Wait Mode gate (emission clock) and released in free run (heard clock) came out
`outputOffsetMs` short. Measured: offset 200, C4 [0,500) held for its full duration —
sustain 0.6 instead of 1.0. The reverse toggle stretches the hold by the offset instead,
vaulting partial holds over the full-credit floor.

**2. The tail of the piece is unreachable in free run under an offset.** The transport
finalized a scored run when the *emission* clock reached the piece end, and `finishRun`
then resolved every pending onset as `missed` — but the judgment clock was still
`outputOffsetMs` behind, so the player following the sound had not yet heard those
onsets. Any onset within `outputOffsetMs` of the end could not be credited at all (the
scorer was already inactive when the press arrived), and the last note's sustain was
truncated by the offset. Measured: offset 300, last onset 200 ms before the end, pressed
exactly in time with the delayed sound — judged `missed`; the same run judges it
`perfect` after the fix.

## What Changes

- **The scorer receives both clocks and picks per use.** Every scorer entry point takes a
  `ScoreClocks` pair (emission + heard) assembled in one place
  (`PlayerData.clocksAt`). Attacks, gate stamps and miss windows keep the mode's judgment
  clock (`judgmentClock`, the same rule `judgmentClockAt` pins). A bound note's
  **sustain** is measured on the clock that *bound* it (`sustainClock`, keyed on the
  note's mode stamp), in `noteOff`, the `tick` auto-finalize and the `finishRun`
  stragglers — so a hold can never straddle two clocks (defect 1).
- **A scored run finalizes on the judgment clock.** The transport finishes a scored run
  at `PlayerData.scoredRunEndMs` — the first playhead whose judgment clock has reached
  the piece end. In Wait Mode and at offset 0 that is the end itself (no behaviour
  change); in free run under an offset the run keeps judging through the
  `outputOffsetMs` drain tail, and the summary appears when the *heard* playhead reaches
  the end (defect 2). Non-scored loops still wrap on the emission clock, which is what
  keeps the score audio seamless.

At the default offset of 0 both clocks coincide and nothing changes, at any speed, in
either mode.

## Deliberately NOT changed: the unit error

The wall-clock/score-clock unit error (offset applied unscaled at speeds other than 1x)
remains open, waiting on the drain design (task 4.1 of `fix-output-offset-units`). Both
behaviours here are expressed through the same accessors the drain will reimplement —
`clocksAt` for the pair, `scoredRunEndMs` as "where the judgment clock reaches the end" —
so the drain slots in beneath them without moving the scorer or the transport again.

## Impact

- Affected specs: `performance-scoring` (Output Offset Applied To The Timing Reference).
  Depends on `fix-output-offset-units` (PR #269): the delta below extends that change's
  requirement text and this branch builds on its commit.
- Affected code: `apps/music/lib/state/performance_scoring_core.dart` (`ScoreClocks`,
  `judgmentClock`, `sustainClock`), `performance_scoring.dart` (entry points take the
  pair; per-note sustain clock), `player_data.dart` (`clocksAt`/`clocks`,
  `scoredRunEndMs`), `player_notifier.dart` (call sites; scored-run finish condition).
- UI readers of `referenceMs` are untouched. The visual playhead already draws the heard
  position, so during the new drain tail it approaches the piece end exactly as the last
  audio plays out.
