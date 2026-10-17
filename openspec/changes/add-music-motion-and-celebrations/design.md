# Design

## Context

Existing motion: `AnimationController`s in the player (countdown, hit effects, metronome
pulse, mistake replay), the path's pulsing next node, the swipe deck, snackbars. Nothing
shared: durations and curves are literals per widget. Celebrations are `Dialog`s
(`lesson_celebration.dart`, `reward_celebration.dart`) with a static icon; the colour
discipline is already right (reward moments wear the lilac accent, never success green).
Reduced motion is not read anywhere. See proposal.md — Why.

## Goals / Non-Goals

**Goals:**
- Delight on six moments; consistency everywhere else.
- Accessible by default: reduced motion, haptics follow the system.
- Cheap: no dependency, no assets, bounded particle counts, measured on low-end Android.

**Non-Goals:**
- No redesign of the play surface's gamified feedback (countdown, sparks, gauge) beyond
  reading the shared tokens.
- No sound effects: the synth owns audio and a lesson may run with the piano muted.
- No Rive/Lottie: an asset pipeline for a handful of effects is heavier than drawing them.
- No new celebration triggers: the moments listed are the ones whose state already exists.

## Decisions

### D1 — Tokens in `theme/motion.dart`, read through `MotionScale`
`Motion.quick = 160 ms`, `standard = 280 ms`, `celebrate = 900 ms`, curves `easeOutCubic`
and `easeOutBack`. A `MotionScale` inherited widget multiplies every duration by 1 or 0
from `MediaQuery.disableAnimations`; widgets never read the platform flag themselves.
The player's existing controllers adopt the tokens where their literal already matches.

### D2 — One `CelebrationSurface`, six callers
Alternatives: animate each dialog separately.
Chosen: a single widget taking an icon, title, subtitle, optional counter (from → to), and
actions, used by the lesson celebration, reward celebration, level-up, daily-session
complete and streak milestone; the session summary embeds only the counter and the stars
row (stars from `add-music-piece-stars`). Sequence: icon scales in with `easeOutBack`,
confetti burst fires once, title fades up, counter counts for `celebrate`, actions fade in
last so nothing can be tapped mid-flight. Under reduced motion the whole thing paints its
final frame.

### D3 — Confetti is a `CustomPainter` with a fixed particle pool
Forty particles, positions precomputed from a seeded random so goldens are stable, driven
by one controller, no per-frame allocation (positions in a `Float32List`), painted in the
reward accent plus two tints. Over the dialog only, never over a play surface.

### D4 — Counters interpolate integers, not doubles
`AnimatedCounter(from, to, duration)` tweens an int and formats through the existing
localized formatter; the standing pill wraps it, the summary's "+N" cue and the daily ring
use it. The pill animates only when the value grew (a debit after a streak buy-back shows
instantly, which reads as honest).

### D5 — The path curve is painted behind the nodes
`PathCurvePainter` draws a cubic through the node centres (reusing the meander indents),
filled up to the last completed node in the tertiary colour and dashed beyond; completing
a node animates the check (`quick`) then the fill to the next node (`standard`). Unit
headers flash once when their count reaches total, remembered per session so a rebuild
does not replay it.

### D6 — Haptics behind a seam
`hapticsProvider` wraps `HapticFeedback.lightImpact`, mocked in tests, called from the
celebration surface and the pass outcome only; it checks the platform's haptics setting
through the seam and never fires on a miss or an error.

## Risks / Trade-offs

- [Frame drops on low-end Android during confetti] → fixed pool, `RepaintBoundary` on the
  surface, measured on the project's reference device before release; the particle count
  is a single constant to lower.
- [A celebration over a short landscape phone] → the surface keeps the dialogs' scroll and
  pinned-actions rules; the burst is clipped to the dialog.
- [Replayed celebrations on rebuild] → every celebration is driven by an explicit cue from
  a listener widget, never by `build`.
