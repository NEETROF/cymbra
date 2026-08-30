// Copyright 2026 NEETROF
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

import 'dart:math' as math;

/// Pure, host-testable scoring math for the gamified piano practice feature.
///
/// No Flutter imports: every function here is a deterministic transform over
/// primitives so it is covered by fast unit tests. The notifier
/// (`performance_scoring.dart`) collects live judgments and calls into this
/// module; the models (`session_summary.dart`) reuse the enums defined here.

/// The pair of score clocks every scorer entry point receives. [emission] is
/// the transport playhead — the clock that decides when a note is handed to
/// the audio engine — and [heard] is the position the player is actually
/// hearing: the emission clock shifted back by the output offset (see
/// `PlayerData.clocksAt`). Identical at the default offset of 0.
///
/// The scorer takes both rather than one pre-picked value so that *per-note*
/// choices stay possible: an attack is judged on the clock of the mode it is
/// made in, but a sustain must be measured on the clock that *bound* the note
/// — see [judgmentClock] and [sustainClock].
typedef ScoreClocks = ({double emission, double heard});

/// The clock an attack (or a miss window, or a gate stamp) is judged on under
/// [waitMode]: the emission clock in Wait Mode, where the frozen playhead has
/// to match its own onset to the millisecond, and the heard clock in free run,
/// where the player is reacting to sound.
double judgmentClock(ScoreClocks clocks, {required bool waitMode}) =>
    waitMode ? clocks.emission : clocks.heard;

/// The clock a bound note's sustain is measured on: the SAME one that bound it
/// ([boundInWaitMode]), regardless of the mode at release time. A note held
/// across a mid-hold Wait Mode toggle would otherwise have its hold measured
/// on two clocks a whole output offset apart, and come out short (Wait → free)
/// or long (free → Wait) by exactly that.
double sustainClock(ScoreClocks clocks, {required bool boundInWaitMode}) =>
    boundInWaitMode ? clocks.emission : clocks.heard;

/// The engine's nominal pitch-confirmation window for acoustic detection
/// (change: add-acoustic-piano-input): the evidence the presence stage
/// accumulates after an onset before the note event is emitted. MUST mirror
/// `DETECTION_CONFIRM_NOMINAL_MS` in `rust/src/api/audio_input_core.rs` —
/// scoring adds it to the measured input offset so the whole detection chain
/// is compensated, not just the capture round trip.
const double kDetectionConfirmMs = 46;

/// Shifts both clocks earlier by [offsetMs] — how an audio-sourced attack is
/// judged at its true time (delta spec: Measured Input Offset Applied To
/// Audio-Sourced Attacks). Callers pass 0 (or skip the call) for MIDI input,
/// which stays bit-identical by construction.
ScoreClocks shiftClocksForInput(ScoreClocks clocks, double offsetMs) =>
    (emission: clocks.emission - offsetMs, heard: clocks.heard - offsetMs);

/// Ordered timing verdict for one onset, best (`perfect`) to worst (`missed`).
///
/// In free-run (Wait Mode off) the verdict comes from the signed offset of the
/// attack against the note's scheduled onset; in Wait Mode it comes from the
/// reaction time after the gate opens. `missed` only occurs in free-run — the
/// Wait Mode gate blocks until the pitch is pressed.
enum TimingVerdict { perfect, good, early, late, missed }

/// How a whole scored run relates to Wait Mode, derived at finalize time from
/// the per-onset mode stamps. Feeds the two separate leaderboards.
enum RunMode { free, wait, mixed }

/// Timing windows, expressed in **score-clock milliseconds** for free-run and
/// **wall-clock milliseconds** for Wait-Mode reaction. Tunable constants — not
/// spec-locked; validate on-device before tightening.
class ScoringWindows {
  /// Free-run: |offset| ≤ this ⇒ `perfect`.
  static const double freePerfectMs = 40;

  /// Free-run: |offset| ≤ this ⇒ `good`.
  static const double freeGoodMs = 90;

  /// Free-run: |offset| ≤ this ⇒ `early`/`late` and the press still *binds* to
  /// the onset. Beyond it a press does not bind (it is an extra/wrong note).
  static const double freeBindMs = 160;

  /// Wait Mode: reaction ≤ this ⇒ `perfect`.
  static const double waitPerfectMs = 120;

  /// Wait Mode: reaction ≤ this ⇒ `good`; beyond it ⇒ `late`.
  static const double waitGoodMs = 300;

  /// Sustain credit floor: holding at least this fraction of the intended
  /// duration counts as a full-value sustain.
  static const double sustainCreditFloor = 0.85;
}

/// Dimension weights for the synchronization blend (sum to 1).
class ScoringWeights {
  static const double timing = 0.5;
  static const double correctness = 0.3;
  static const double sustain = 0.2;
}

/// Dimension weights for a **percussion** run's blend (change:
/// add-drum-scoring): timing and correctness only, sum to 1.
///
/// A stroke has no sustain dimension — its note-off timing is an artefact of
/// the module's firmware, not of the player — so the third dimension is
/// *absent*, and the two that remain are renormalized to preserve their
/// keyboard ratio: 0.5 : 0.3 ⇒ 0.625 : 0.375 (`percussionWeightsPreserveRatio`
/// pins it). Written as literals rather than derived at compile time so the
/// values are exact doubles, and so the keyboard constants above stay
/// byte-for-byte what they were — the brief's hard constraint.
///
/// Fixing sustain at a constant 1.0 instead would leak 20% of free credit into
/// every percussion run (a do-nothing run would score 20, not 0) and make
/// percussion percentages incomparable upward.
class PercussionScoringWeights {
  static const double timing = 0.625;
  static const double correctness = 0.375;
}

/// Free-run timing verdict for a bound attack whose signed score-clock offset
/// from the onset is [offsetMs] (negative = early, positive = late).
///
/// Assumes the press is within [ScoringWindows.freeBindMs]; callers decide
/// binding vs. wrong-note first. Never returns `missed` (that is assigned by the
/// scorer when an onset window closes unplayed).
TimingVerdict verdictForOffsetMs(double offsetMs) {
  final mag = offsetMs.abs();
  if (mag <= ScoringWindows.freePerfectMs) return TimingVerdict.perfect;
  if (mag <= ScoringWindows.freeGoodMs) return TimingVerdict.good;
  return offsetMs < 0 ? TimingVerdict.early : TimingVerdict.late;
}

/// Whether a free-run attack at signed offset [offsetMs] is close enough to
/// *bind* to the onset (vs. being counted as an extra/wrong note).
bool bindsToOnset(double offsetMs) =>
    offsetMs.abs() <= ScoringWindows.freeBindMs;

/// Wait-Mode timing verdict from the [reactionMs] between the gate opening on an
/// onset and the correct attack. Reaction is non-negative; there is no `missed`.
TimingVerdict verdictForReactionMs(double reactionMs) {
  final r = math.max(0.0, reactionMs);
  if (r <= ScoringWindows.waitPerfectMs) return TimingVerdict.perfect;
  if (r <= ScoringWindows.waitGoodMs) return TimingVerdict.good;
  return TimingVerdict.late;
}

/// Sustain ratio in [0, 1] for a correctly-attacked note held [heldMs] against
/// an intended [intendedMs]. Holding ≥ [ScoringWindows.sustainCreditFloor] of
/// the intended duration scores a full 1.0; over-holding is never penalized; a
/// non-positive intended duration scores 1.0.
double sustainRatioFor(double heldMs, double intendedMs) {
  if (intendedMs <= 0) return 1;
  final ratio = heldMs / intendedMs;
  if (ratio >= ScoringWindows.sustainCreditFloor) return 1;
  return ratio.clamp(0.0, 1.0).toDouble();
}

/// Whether a verdict counts as a landed (non-missed) onset.
bool isHit(TimingVerdict v) => v != TimingVerdict.missed;

/// Timing points in [0, 1] awarded for a single onset verdict.
double timingPointOf(TimingVerdict v) => switch (v) {
  TimingVerdict.perfect => 1.0,
  TimingVerdict.good => 0.8,
  TimingVerdict.early || TimingVerdict.late => 0.5,
  TimingVerdict.missed => 0.0,
};

double _mean(Iterable<double> xs) {
  var sum = 0.0;
  var n = 0;
  for (final x in xs) {
    sum += x;
    n++;
  }
  return n == 0 ? 1.0 : sum / n;
}

/// Timing dimension in [0, 1] over the judged onset [verdicts] (1.0 if none).
double timingScore(Iterable<TimingVerdict> verdicts) =>
    _mean(verdicts.map(timingPointOf));

/// Correctness dimension in [0, 1]: landed onsets over all judged onsets plus
/// extra/wrong notes. 1.0 when nothing has been judged yet.
double correctnessScore(Iterable<TimingVerdict> verdicts, int wrongNotes) {
  final list = verdicts.toList(growable: false);
  final denom = list.length + wrongNotes;
  if (denom == 0) return 1;
  final hits = list.where(isHit).length;
  return hits / denom;
}

/// Sustain dimension in [0, 1] over the per-hit [sustainRatios].
///
/// Empty ratios are ambiguous: either the run is **pristine** (no onset judged
/// yet) or **every judged onset was missed** (nothing to sustain). Pass
/// [anyOnsetJudged] to disambiguate — pristine scores a full 1.0 so the gauge
/// renders before the first judgment, whereas a run with judged-but-all-missed
/// onsets scores 0. Without this gate, a do-nothing run leaks the full sustain
/// weight (0.2 ⇒ 20%) into the blend even though nothing was held.
double sustainScore(
  Iterable<double> sustainRatios, {
  bool anyOnsetJudged = false,
}) {
  var sum = 0.0;
  var n = 0;
  for (final x in sustainRatios) {
    sum += x;
    n++;
  }
  if (n == 0) return anyOnsetJudged ? 0.0 : 1.0;
  return sum / n;
}

/// Rolling/overall synchronization percentage in [0, 100] blended from the
/// three dimensions. Defined before the first judgment (empty inputs ⇒ 100), so
/// the gauge always renders.
double syncPercent({
  required Iterable<TimingVerdict> onsetVerdicts,
  required Iterable<double> sustainRatios,
  required int wrongNotes,
}) {
  final verdicts = onsetVerdicts.toList(growable: false);
  final blend =
      ScoringWeights.timing * timingScore(verdicts) +
      ScoringWeights.correctness * correctnessScore(verdicts, wrongNotes) +
      ScoringWeights.sustain *
          sustainScore(sustainRatios, anyOnsetJudged: verdicts.isNotEmpty);
  return (blend * 100).clamp(0.0, 100.0).toDouble();
}

/// Rolling/overall synchronization percentage in [0, 100] for a **percussion**
/// run (change: add-drum-scoring): the timing and correctness dimensions only,
/// blended at [PercussionScoringWeights].
///
/// Defined before the first judgment (empty inputs ⇒ 100), like [syncPercent],
/// so the gauge always renders — and 0 for a run whose every onset was missed
/// with no stroke played, because no absent dimension leaks credit into the
/// blend.
double percussionSyncPercent({
  required Iterable<TimingVerdict> onsetVerdicts,
  required int wrongNotes,
}) {
  final verdicts = onsetVerdicts.toList(growable: false);
  final blend =
      PercussionScoringWeights.timing * timingScore(verdicts) +
      PercussionScoringWeights.correctness *
          correctnessScore(verdicts, wrongNotes);
  return (blend * 100).clamp(0.0, 100.0).toDouble();
}

/// The verdict of a stroke that bound with the **wrong hi-hat articulation**
/// (change: add-drum-scoring): capped below `perfect`, never below what its
/// timing already earned.
///
/// Open versus closed is the one distinction the cascade draws inside a lane,
/// so erasing it would void a drawn promise — but producing it needs hardware
/// many kits lack, so it must never gate. Shading resolves both: the run always
/// completes, the difference always costs.
TimingVerdict capBelowPerfect(TimingVerdict v) =>
    v == TimingVerdict.perfect ? TimingVerdict.good : v;

/// Feedback tier 0–4 for a synchronization [percent], one per 20% band. Pure so
/// the gauge/effects escalation is testable without rendering.
int feedbackTier(double percent) => (percent / 20).floor().clamp(0, 4);

/// Run classification from the per-mode onset counts (see [RunMode]).
RunMode classifyRun({required int freeOnsets, required int waitOnsets}) {
  if (waitOnsets == 0) return RunMode.free;
  if (freeOnsets == 0) return RunMode.wait;
  return RunMode.mixed;
}
