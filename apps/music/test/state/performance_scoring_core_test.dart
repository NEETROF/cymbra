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

void main() {
  group('free-run timing verdict (signed offset)', () {
    test('within the perfect window scores perfect either side', () {
      expect(verdictForOffsetMs(0), TimingVerdict.perfect);
      expect(verdictForOffsetMs(-40), TimingVerdict.perfect);
      expect(verdictForOffsetMs(40), TimingVerdict.perfect);
    });

    test('inside the good window scores good', () {
      expect(verdictForOffsetMs(41), TimingVerdict.good);
      expect(verdictForOffsetMs(-90), TimingVerdict.good);
    });

    test('outside good but within bind scores early/late by sign', () {
      expect(verdictForOffsetMs(-91), TimingVerdict.early);
      expect(verdictForOffsetMs(91), TimingVerdict.late);
      expect(verdictForOffsetMs(160), TimingVerdict.late);
    });

    test('binds only within the bind window', () {
      expect(bindsToOnset(160), isTrue);
      expect(bindsToOnset(-160), isTrue);
      expect(bindsToOnset(161), isFalse);
    });
  });

  group('wait-mode reaction verdict', () {
    test('fast reaction scores perfect', () {
      expect(verdictForReactionMs(0), TimingVerdict.perfect);
      expect(verdictForReactionMs(120), TimingVerdict.perfect);
    });

    test('slower reaction scores good then late', () {
      expect(verdictForReactionMs(121), TimingVerdict.good);
      expect(verdictForReactionMs(300), TimingVerdict.good);
      expect(verdictForReactionMs(301), TimingVerdict.late);
    });

    test('negative reaction is clamped to perfect, never missed', () {
      expect(verdictForReactionMs(-50), TimingVerdict.perfect);
    });
  });

  group('sustain ratio', () {
    test('full or over-hold scores 1.0', () {
      expect(sustainRatioFor(1000, 1000), 1.0);
      expect(sustainRatioFor(2000, 1000), 1.0);
    });

    test('at the credit floor scores 1.0', () {
      expect(sustainRatioFor(850, 1000), 1.0);
    });

    test('released far too early scores low', () {
      expect(sustainRatioFor(100, 1000), closeTo(0.1, 1e-9));
      expect(sustainRatioFor(0, 1000), 0.0);
    });

    test('non-positive intended duration scores 1.0', () {
      expect(sustainRatioFor(0, 0), 1.0);
    });
  });

  group('dimension scores', () {
    test('timing score is the mean of per-verdict points', () {
      expect(timingScore(const []), 1.0);
      expect(
        timingScore(const [TimingVerdict.perfect, TimingVerdict.missed]),
        closeTo(0.5, 1e-9),
      );
    });

    test('correctness counts hits over onsets plus wrong notes', () {
      expect(correctnessScore(const [], 0), 1.0);
      expect(
        correctnessScore(const [
          TimingVerdict.perfect,
          TimingVerdict.missed,
        ], 0),
        closeTo(0.5, 1e-9),
      );
      // 1 hit, 1 onset, 1 wrong note → 1/2.
      expect(
        correctnessScore(const [TimingVerdict.good], 1),
        closeTo(0.5, 1e-9),
      );
    });

    test('sustain score is the mean of ratios (1.0 if none)', () {
      expect(sustainScore(const []), 1.0);
      expect(sustainScore(const [1.0, 0.0]), closeTo(0.5, 1e-9));
    });
  });

  group('synchronization percentage', () {
    test('defined at 100 before any judgment', () {
      expect(
        syncPercent(
          onsetVerdicts: const [],
          sustainRatios: const [],
          wrongNotes: 0,
        ),
        100.0,
      );
    });

    test('perfect, well-sustained play trends to 100', () {
      final pct = syncPercent(
        onsetVerdicts: const [TimingVerdict.perfect, TimingVerdict.perfect],
        sustainRatios: const [1.0, 1.0],
        wrongNotes: 0,
      );
      expect(pct, 100.0);
    });

    test('misses and wrong notes drag the percentage down', () {
      final good = syncPercent(
        onsetVerdicts: const [TimingVerdict.perfect],
        sustainRatios: const [1.0],
        wrongNotes: 0,
      );
      final bad = syncPercent(
        onsetVerdicts: const [TimingVerdict.perfect, TimingVerdict.missed],
        sustainRatios: const [1.0],
        wrongNotes: 2,
      );
      expect(bad, lessThan(good));
      expect(bad, inInclusiveRange(0, 100));
    });
  });

  group('feedback tier', () {
    test('maps 20% bands to 0..4 and clamps 100 to 4', () {
      expect(feedbackTier(0), 0);
      expect(feedbackTier(19.9), 0);
      expect(feedbackTier(20), 1);
      expect(feedbackTier(59), 2);
      expect(feedbackTier(80), 4);
      expect(feedbackTier(100), 4);
    });

    test('rises and falls with the percentage', () {
      expect(feedbackTier(41) > feedbackTier(39), isTrue);
      expect(feedbackTier(39) < feedbackTier(61), isTrue);
    });
  });

  group('run classification', () {
    test('pure free / pure wait / mixed', () {
      expect(classifyRun(freeOnsets: 5, waitOnsets: 0), RunMode.free);
      expect(classifyRun(freeOnsets: 0, waitOnsets: 5), RunMode.wait);
      expect(classifyRun(freeOnsets: 3, waitOnsets: 2), RunMode.mixed);
      expect(classifyRun(freeOnsets: 0, waitOnsets: 0), RunMode.free);
    });
  });
}
