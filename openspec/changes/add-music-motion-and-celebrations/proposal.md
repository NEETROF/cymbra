# Proposal

## Why

Cymbra Music rewards a lot and shows almost none of it moving. Finishing a lesson, passing a
piece, earning a badge, closing a daily session or hitting a streak milestone each open a
static dialog with an icon. Points change silently in a pill. The learning path is a list
with one pulsing dot. The play surface itself already has the right instincts — a 3-2-1
countdown, hit sparks, tier colours — but nothing outside the player does. This change
gives the app one motion system and spends it where it pays: on the moments that make a
learner come back tomorrow.

## What Changes

- **One motion system.** A small set of duration and curve tokens, a reduced-motion rule
  honoured everywhere (the platform's setting zeroes every duration), and a budget: no new
  dependency, every effect drawn with Flutter's own animation and painting, disposed when
  its widget leaves.
- **Celebrations that move.** Lesson completed, piece passed, badge earned, level up, daily
  session complete and streak milestone share one animated celebration surface: an icon
  that scales in, a burst of drawn confetti in the reward accent, a title that settles, a
  counter that counts up when points are involved. The existing lesson and reward dialogs
  become this surface; their content and actions are unchanged.
- **Counters count.** The standing pill, the summary's "+N points" cue and the daily ring
  animate from the previous value to the new one instead of jumping.
- **The path is drawn.** Lesson nodes sit on a continuous curve; a completed node animates
  its check and fills the segment to the next node; the next node keeps its pulse; a unit
  that completes lights its header once.
- **Haptics on the moments that matter.** A light haptic on a pass or a completion where
  the platform supports it, following the system haptics setting; never on a miss.
- **Learning-safe stays the law.** Nothing here touches the play surface's rules: no
  celebration for a missed note, no reward in success green, nothing over the notes.

## Capabilities

### New Capabilities
- `music-motion`: the motion tokens and reduced-motion rule, the shared animated
  celebration surface and the moments that use it, animated counters, the drawn learning
  path, haptics, and the performance and learning-safe constraints.

### Modified Capabilities
- None. `gamified-feedback`'s learning-safe constraints are consumed as they stand; the
  celebration dialogs' content and actions keep their existing requirements.

## Impact

Products: **Music only**. Consumed, not new: the lesson celebration, reward celebration,
session summary and standing pill, the learning path body (shared with
`add-music-learn-play-home`), the daily-session completion cue, the reward-cue listeners.
New: `theme/motion.dart` tokens, `widgets/celebration_surface.dart` and its confetti
painter, `widgets/animated_counter.dart`, `painters/path_curve_painter.dart`, a haptics seam.

- Flutter only; no backend, no proto, no new package.
- Tests: widget tests pumping frames for every celebration moment with and without reduced
  motion, counter interpolation, path painter goldens, a frame-budget test on the confetti
  painter (particles count and allocation), haptics seam mock calls.
