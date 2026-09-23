## Why

A user reported that the play screen's get-ready countdown stretches with the transport
tempo: at the 25% speed setting the visible `3 … 2 … 1` ran for ~12 s instead of ~3 s.

The countdown is a **real-world** beat — the player is getting their hands onto the keys,
which takes the same time whatever tempo the piece is about to be played at. It was being
spent in **musical** time: the play screen multiplied the frame delta by the transport
speed *before* handing it to the notifier, and the notifier subtracted that already-scaled
delta from `countdownMs`. Its duration was therefore `kCountdownStartMs / speed` — 14.8 s
at the 0.25x floor (12.0 s of it on the digits, matching the report) and 1.85 s at the 2x
ceiling. Every lap of a measure-range practice loop re-arms the countdown, so a slow
drilling loop paid the stall on every repetition.

The existing spec is silent on this: it never says the countdown is wall-clock, so the
behaviour at speeds other than 1x was undefined rather than wrong-per-spec. It also still
describes a `5 → 4 → 3 → 2 → 1 → GO` countdown, which has not matched the implementation
(`_digitsMs = 3000`, three digits) for some time.

## What Changes

- **The countdown SHALL run on wall-clock time** — the same duration at every transport
  speed. Fixed by moving the speed multiplication out of the play screen's ticker and into
  `Player.advance`, which now takes the **real** frame delta and scales only the part of
  the frame that is musical time (the playhead). This also covers the per-lap re-arm inside
  a selective practice loop, including after a tempo ramp.
- **The spec is corrected** to describe the `3 → 2 → 1 → GO` countdown that ships, and
  gains the wall-clock requirement plus a scenario for it.

**Behaviour change beyond the reported bug (deliberate, worth calling out at review):**
at speeds *above* 1x the countdown now lasts **longer** than before (3.70 s instead of
1.85 s at 2x), and the unscored warm-up window it opens shrinks accordingly at slow tempos
(3.70 s instead of 14.8 s at 0.25x). Both follow from the countdown being a fixed
wall-clock duration; neither was reported, and neither was specified before.

## Impact

- Affected specs: `gamified-feedback` (Pre-Start Countdown).
- Affected code: `apps/music/lib/screens/player_screen.dart` (ticker),
  `apps/music/lib/state/player_notifier.dart` (`advance` contract),
  doc comments in `apps/music/lib/state/player_data.dart` and
  `apps/music/lib/widgets/countdown_overlay.dart`.
- `Player.advance(dtMs)` changes contract from "musical ms, speed already applied by the
  caller" to "real ms, speed applied inside". It has exactly one production caller.
