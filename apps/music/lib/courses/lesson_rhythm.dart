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

/// The rhythm vocabulary of a course exercise (change: add-notation-courses,
/// schema v2) and the pure grading of tapped rhythms.
///
/// A manifest writes a rhythm as figure tokens (`"quarter"`, `"eighth"`, dotted,
/// rests); everything timed is derived here — onset instants at a tempo, and how
/// a learner's taps compare to them. Pure and host-testable, like
/// `metronomeBeatsCrossed` which drives the metronome the learner taps against.
library;

import '../state/note_label.dart' show NoteFigure, dotMultiplier;

/// Fraction of a whole note per figure — the subset a course rhythm may use.
const Map<NoteFigure, double> _fractions = {
  NoteFigure.whole: 1.0,
  NoteFigure.half: 0.5,
  NoteFigure.quarter: 0.25,
  NoteFigure.eighth: 0.125,
  NoteFigure.sixteenth: 0.0625,
};

/// Manifest tokens for the figures a course rhythm may carry (MusicXML-style,
/// matching `note_label.dart`'s naming).
const Map<String, NoteFigure> _tokens = {
  'whole': NoteFigure.whole,
  'half': NoteFigure.half,
  'quarter': NoteFigure.quarter,
  'eighth': NoteFigure.eighth,
  '16th': NoteFigure.sixteenth,
};

/// One figure of a written rhythm: a note or a rest, with augmentation dots.
class RhythmFigure {
  final NoteFigure figure;
  final int dots;
  final bool rest;

  const RhythmFigure(this.figure, {this.dots = 0, this.rest = false});

  /// Parses a manifest element (`{"fig": "quarter", "dots": 1, "rest": true}`).
  /// Returns null on an unknown figure or malformed shape, so the enclosing
  /// block degrades instead of throwing.
  static RhythmFigure? parse(Object? raw) {
    if (raw is! Map) return null;
    final figure = raw['fig'] is String ? _tokens[raw['fig']] : null;
    if (figure == null) return null;
    final dots = raw['dots'];
    return RhythmFigure(
      figure,
      dots: dots is int ? dots.clamp(0, 2) : 0,
      rest: raw['rest'] == true,
    );
  }

  /// How many beats of a `1/[beatType]` meter this figure lasts.
  double beats(int beatType) =>
      _fractions[figure]! * dotMultiplier(dots) * beatType;

  @override
  bool operator ==(Object other) =>
      other is RhythmFigure &&
      other.figure == figure &&
      other.dots == dots &&
      other.rest == rest;

  @override
  int get hashCode => Object.hash(figure, dots, rest);

  @override
  String toString() => 'RhythmFigure($figure, dots: $dots, rest: $rest)';
}

/// A rhythm pattern resolved at a tempo: when each **note** attack falls (rests
/// consume time but produce no onset) and how long the whole pattern lasts.
({List<double> onsetsMs, double totalMs}) rhythmOnsets({
  required List<RhythmFigure> pattern,
  required int bpm,
  required int beatType,
}) {
  final beatMs = 60000.0 / bpm;
  final onsets = <double>[];
  var t = 0.0;
  for (final f in pattern) {
    if (!f.rest) onsets.add(t);
    t += f.beats(beatType) * beatMs;
  }
  return (onsetsMs: onsets, totalMs: t);
}

/// How one learner pass compares to the written onsets: per-onset hit flags
/// (a tap landed within [windowMs] of it) and the count of stray taps that
/// matched nothing. Each tap satisfies at most one onset (nearest first), so a
/// single tap can never validate two attacks.
({List<bool> hit, int extras}) gradeRhythmTaps({
  required List<double> onsetsMs,
  required List<double> tapsMs,
  required double windowMs,
}) {
  final hit = List<bool>.filled(onsetsMs.length, false);
  final used = List<bool>.filled(tapsMs.length, false);
  for (var i = 0; i < onsetsMs.length; i++) {
    var best = -1;
    var bestDist = double.infinity;
    for (var j = 0; j < tapsMs.length; j++) {
      if (used[j]) continue;
      final d = (tapsMs[j] - onsetsMs[i]).abs();
      if (d <= windowMs && d < bestDist) {
        best = j;
        bestDist = d;
      }
    }
    if (best >= 0) {
      hit[i] = true;
      used[best] = true;
    }
  }
  return (hit: hit, extras: used.where((u) => !u).length);
}
