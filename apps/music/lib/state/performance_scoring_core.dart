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

/// Sustain dimension in [0, 1] over the per-hit [sustainRatios] (1.0 if none).
double sustainScore(Iterable<double> sustainRatios) => _mean(sustainRatios);

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
      ScoringWeights.sustain * sustainScore(sustainRatios);
  return (blend * 100).clamp(0.0, 100.0).toDouble();
}

/// Feedback tier 0–4 for a synchronization [percent], one per 20% band. Pure so
/// the gauge/effects escalation is testable without rendering.
int feedbackTier(double percent) =>
    (percent / 20).floor().clamp(0, 4);

/// Run classification from the per-mode onset counts (see [RunMode]).
RunMode classifyRun({required int freeOnsets, required int waitOnsets}) {
  if (waitOnsets == 0) return RunMode.free;
  if (freeOnsets == 0) return RunMode.wait;
  return RunMode.mixed;
}
