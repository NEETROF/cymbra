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

import 'package:flutter_test/flutter_test.dart';
import 'package:music/state/performance_scoring_core.dart';
import 'package:music/state/session_summary.dart';

// The percussion blend (change: add-drum-scoring): two dimensions, the
// keyboard ratio preserved, and a sustain that is ABSENT rather than zero.

NoteJudgment stroke(
  TimingVerdict verdict, {
  int index = 0,
  int pitch = 38,
  bool waitMode = false,
  double? offsetMs = 0,
}) => NoteJudgment(
  noteIndex: index,
  pitch: pitch,
  startMs: index * 100,
  waitMode: waitMode,
  verdict: verdict,
  timingOffsetMs: waitMode ? null : offsetMs,
  reactionMs: waitMode ? 50 : null,
  sustainRatio: null, // a stroke never carries one
);

SessionResult drumResult(List<NoteJudgment> judgments) =>
    SessionResult.fromJudgments(
      pieceId: 'groove',
      title: 'Groove',
      hands: 'handsAndFeet',
      judgments: judgments,
      bestCombo: 1,
      playedAtMs: 0,
      speed: 1,
      percussion: true,
    );

void main() {
  group('the keyboard constants are untouched', () {
    test('weights and windows are byte-for-byte what they were', () {
      expect(ScoringWeights.timing, 0.5);
      expect(ScoringWeights.correctness, 0.3);
      expect(ScoringWeights.sustain, 0.2);
      expect(
        ScoringWeights.timing +
            ScoringWeights.correctness +
            ScoringWeights.sustain,
        closeTo(1.0, 1e-12),
      );
      expect(ScoringWindows.freePerfectMs, 40);
      expect(ScoringWindows.freeGoodMs, 90);
      expect(ScoringWindows.freeBindMs, 160);
      expect(ScoringWindows.waitPerfectMs, 120);
      expect(ScoringWindows.waitGoodMs, 300);
    });

    test('a keyboard run still blends three dimensions', () {
      // All perfect, held full: 100. Drop the sustain to 0 and exactly the
      // sustain weight is lost — the old arithmetic, unchanged.
      const verdicts = [TimingVerdict.perfect, TimingVerdict.perfect];
      expect(
        syncPercent(
          onsetVerdicts: verdicts,
          sustainRatios: const [1, 1],
          wrongNotes: 0,
        ),
        closeTo(100, 1e-9),
      );
      expect(
        syncPercent(
          onsetVerdicts: verdicts,
          sustainRatios: const [0, 0],
          wrongNotes: 0,
        ),
        closeTo(80, 1e-9),
      );
    });
  });

  group('the percussion weights renormalize the keyboard ratio', () {
    test('they sum to 1 and keep the timing:correctness ratio', () {
      expect(
        PercussionScoringWeights.timing + PercussionScoringWeights.correctness,
        closeTo(1.0, 1e-12),
      );
      expect(
        PercussionScoringWeights.timing / PercussionScoringWeights.correctness,
        closeTo(ScoringWeights.timing / ScoringWeights.correctness, 1e-12),
      );
      // The values the design names, so a silent drift is a failing test.
      expect(PercussionScoringWeights.timing, 0.625);
      expect(PercussionScoringWeights.correctness, 0.375);
    });
  });

  group('percussionSyncPercent', () {
    test('is defined before the first judgment so the gauge renders', () {
      expect(
        percussionSyncPercent(onsetVerdicts: const [], wrongNotes: 0),
        100,
      );
    });

    test('a run of perfect, correct strokes reaches 100 with no sustain', () {
      expect(
        percussionSyncPercent(
          onsetVerdicts: const [TimingVerdict.perfect, TimingVerdict.perfect],
          wrongNotes: 0,
        ),
        closeTo(100, 1e-9),
      );
    });

    test('a do-nothing run scores 0, not the sustain weight', () {
      // Every onset missed, no stroke played: nothing leaks in from the
      // dimension that does not exist.
      expect(
        percussionSyncPercent(
          onsetVerdicts: const [TimingVerdict.missed, TimingVerdict.missed],
          wrongNotes: 0,
        ),
        0,
      );
    });

    test('extra strokes lower the percentage through correctness', () {
      final clean = percussionSyncPercent(
        onsetVerdicts: const [TimingVerdict.perfect],
        wrongNotes: 0,
      );
      final noisy = percussionSyncPercent(
        onsetVerdicts: const [TimingVerdict.perfect],
        wrongNotes: 3,
      );
      expect(noisy, lessThan(clean));
      // Timing is untouched by an extra stroke: only correctness moves.
      expect(
        noisy,
        closeTo(
          PercussionScoringWeights.timing * 100 +
              PercussionScoringWeights.correctness * 25,
          1e-9,
        ),
      );
    });

    test('blends only the two dimensions, at their stated weights', () {
      // One perfect (timing 1.0) and one missed (timing 0.0) ⇒ timing 0.5,
      // correctness 0.5.
      expect(
        percussionSyncPercent(
          onsetVerdicts: const [TimingVerdict.perfect, TimingVerdict.missed],
          wrongNotes: 0,
        ),
        closeTo(50, 1e-9),
      );
    });
  });

  group('capBelowPerfect', () {
    test('a perfect becomes a good, and nothing else moves', () {
      expect(capBelowPerfect(TimingVerdict.perfect), TimingVerdict.good);
      for (final v in const [
        TimingVerdict.good,
        TimingVerdict.early,
        TimingVerdict.late,
        TimingVerdict.missed,
      ]) {
        expect(capBelowPerfect(v), v);
      }
    });
  });

  group('a percussion session result has no sustain dimension', () {
    test('the aggregate is absent, not zero', () {
      final result = drumResult([
        stroke(TimingVerdict.perfect),
        stroke(TimingVerdict.good, index: 1),
      ]);
      expect(result.sustain, isNull);
      expect(result.timing, greaterThan(0));
      expect(result.correctness, 1);
    });

    test('a keyboard result keeps its aggregate', () {
      final result = SessionResult.fromJudgments(
        pieceId: 'p',
        title: 'P',
        hands: 'both',
        judgments: [
          const NoteJudgment(
            noteIndex: 0,
            pitch: 60,
            startMs: 0,
            waitMode: false,
            verdict: TimingVerdict.perfect,
            timingOffsetMs: 0,
            sustainRatio: 1,
          ),
        ],
        bestCombo: 1,
        playedAtMs: 0,
        speed: 1,
      );
      expect(result.sustain, 1);
      expect(result.overallSyncPct, closeTo(100, 1e-9));
    });

    test('the overall percentage uses the two-dimension blend', () {
      final result = drumResult([
        stroke(TimingVerdict.perfect),
        stroke(TimingVerdict.missed, index: 1),
      ]);
      expect(result.overallSyncPct, closeTo(50, 1e-9));
    });

    test('an instant release costs nothing anywhere in the run', () {
      // Nothing in a percussion record can even express a sustain penalty.
      final result = drumResult([stroke(TimingVerdict.perfect)]);
      expect(result.notes.every((j) => j.sustainRatio == null), isTrue);
      expect(result.overallSyncPct, closeTo(100, 1e-9));
    });

    test('serialization round-trips the ABSENCE, never a zero', () {
      final result = drumResult([
        stroke(TimingVerdict.perfect),
        stroke(TimingVerdict.good, index: 1, waitMode: true),
      ]);
      final json = result.toJson();
      expect(json.containsKey('sustain'), isFalse);
      expect(
        (json['notes'] as List).cast<Map<String, dynamic>>().every(
          (n) => !n.containsKey('sustainRatio'),
        ),
        isTrue,
      );

      final restored = SessionResult.fromJson(json);
      expect(restored.sustain, isNull);
      expect(restored.notes.every((j) => j.sustainRatio == null), isTrue);
      expect(restored.overallSyncPct, result.overallSyncPct);
      expect(restored.runMode, result.runMode);
    });

    test('a keyboard record still round-trips its sustain', () {
      final json = SessionResult.fromJudgments(
        pieceId: 'p',
        title: 'P',
        hands: 'right',
        judgments: [
          const NoteJudgment(
            noteIndex: 0,
            pitch: 60,
            startMs: 0,
            waitMode: false,
            verdict: TimingVerdict.good,
            timingOffsetMs: 50,
            sustainRatio: 0.4,
          ),
        ],
        bestCombo: 1,
        playedAtMs: 0,
        speed: 1,
      ).toJson();
      expect(json['sustain'], isNotNull);
      final restored = SessionResult.fromJson(json);
      expect(restored.sustain, closeTo(0.4, 1e-9));
      expect(restored.notes.single.sustainRatio, closeTo(0.4, 1e-9));
    });

    test('the per-mode sub-scores use the percussion blend too', () {
      final result = drumResult([
        stroke(TimingVerdict.perfect),
        stroke(TimingVerdict.missed, index: 1),
        stroke(TimingVerdict.perfect, index: 2, waitMode: true),
      ]);
      expect(result.runMode, RunMode.mixed);
      expect(result.freeSyncPct, closeTo(50, 1e-9));
      expect(result.waitSyncPct, closeTo(100, 1e-9));
      expect(result.freeOnsetCount, 2);
      expect(result.waitOnsetCount, 1);
    });
  });
}
