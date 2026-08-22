## Why

The "Audio delay compensation" setting (`outputOffsetMs`) shifts the score position the
scorer judges against, so a player following delayed audio is not penalized. Two
pre-existing defects were found around it, both verified against the shipping code.

**1. Any offset >= 2 ms silently zeroes Wait Mode scoring.** In Wait Mode the cascade
freezes with the playhead exactly on the onset, and the scorer identifies the gating onset
by matching that position to within a millisecond. Handing it the shifted clock made it
look ~200 score-ms behind the frozen playhead, so it matched nothing: the press was
recorded **nowhere** — not as a hit, not even as wrong — and every note ended `missed` at
`finishRun`. Play *feels* normal throughout, because the visual gate and the cascade read
the unshifted playhead. Measured: offset 200 -> 0 notes judged; offset 0 -> 1 note,
`perfect`.

The current spec is what asked for this: "Wait Mode inherits the shift ... the gate opens
against the same shifted reference". The reasoning behind it ("one number, so what is
highlighted and what is judged cannot drift apart") is sound for free run and wrong for
Wait Mode, where the verdict is a reaction time taken from the wall clock and the score
clock only answers *which onset is gating*. Nothing has sounded during a freeze.

**2. The compensation is in the wrong unit — NOT fixed here, see below.**
`outputOffsetMs` is a wall-clock latency, subtracted straight from a score-time playhead.
While the sound is in flight the playhead only covers `outputOffsetMs * speed` ms of
score, so at any transport speed other than 1x the shift is off by a factor of `speed`.
Measured with a 200 ms offset, a player attacking exactly when the sound reaches their ear
is judged `early` at 0.25x, and at 2x is judged `missed` **plus** a second `wrong`
judgment for the same keypress.

**3. The regression test that should have caught defect 1** asserted
`isNot(contains(missed))` over an **empty** hit list, which passes for free.

## What Changes

- **Wait Mode SHALL judge on the emission clock.** The gate is identified by where the
  playhead is actually frozen; reaction time already comes from the wall clock and is
  unaffected. Free run keeps the heard clock — that is what the setting is for. Both come
  from one accessor (`PlayerData.judgmentClockAt`), so the five scorer call sites cannot
  drift apart, and the hand-duplicated `next - outputOffsetMs` at the `finishRun` site is
  gone.
- The vacuous assertion is repaired, `isNot(perfect)` is tightened to the exact verdict,
  and the file gains tempo-invariance coverage.

At the default offset of 0 nothing changes, at any speed, in either mode.

## Deliberately NOT changed: the unit error (defect 2)

The obvious fix — `referenceMs = elapsedMs - outputOffsetMs * speed` — was implemented,
reviewed, measured, and **reverted**. It makes the judgment clock a function of `speed`,
which the player can tap at any moment on the transport bar, and the scorer measures
*durations* against positions stamped earlier on that clock. Measured on the implemented
version:

- Free run, offset 200, a pending onset at the playhead: tapping 1x -> 0.25x moves the
  judgment clock forward by 150 ms in one call, against a 160 ms bind window. The next
  frame's miss sweep marks the onset `missed` and resets the combo, and the player's
  subsequent press is then logged as a second, *wrong* note. Sync 0% where it was 100%.
- A backward jump instead clamps `held = playheadMs - note.startMs` to zero, finalizing a
  correctly-played note's sustain at 0. Measured sync 84% -> 80% for one note.

Neither is reachable today: the raw offset does not depend on `speed`, so nothing can move
the clock mid-run. Trading a wrong-by-a-constant-factor judgment for a clock that teleports
on a common gesture is a bad trade.

The real fix is not a rescale but a **drain**: the lag in score time is the integral of
`speed` over the last `outputOffsetMs` of *real* time, so the heard position has to become
a delayed copy of the playhead (continuous by construction, transitioning over
`outputOffsetMs` after a tempo change) rather than an instantaneous multiplication. That
needs state and a design, and belongs in its own change. Two tests added here — the
`judgmentClockMs` tempo-invariance unit test and the mid-run tempo tap behavioural test —
must stay green through it.

## Impact

- Affected specs: `performance-scoring` (Output Offset Applied To The Timing Reference).
- Affected code: `apps/music/lib/state/player_data.dart` (new `judgmentClockAt`,
  `judgmentClockMs`), `apps/music/lib/state/player_notifier.dart` (five scorer call sites).
- UI readers of `referenceMs` are untouched, in call shape and in value.

## Known, still open after this change

- The unit error above (defect 2).
- **A held note whose Wait Mode is toggled mid-hold** has its sustain measured on two
  clocks and comes out short. Strictly better than what it replaces (before, that press
  bound to nothing and the note ended `missed` with no sustain at all), and it disappears
  with the same drain design.
- **The tail of the piece is unreachable in free run under an offset**: the transport
  finalizes when the *emission* clock reaches the end, by which point the judgment clock is
  `outputOffsetMs` short, so the last notes can be `missed` for a player following the
  sound. Pre-existing at every speed and unchanged here.
