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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/clock_service.dart';
import 'package:music/state/input_calibration_notifier.dart';
import 'package:music/state/performance_scoring.dart';
import 'package:music/state/performance_scoring_core.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/session_summary.dart';

/// Audio-sourced scoring (change: add-acoustic-piano-input): the measured
/// input offset shifts judgments, the sustain dimension is excluded (the
/// percussion precedent), the record is stamped — and MIDI runs stay
/// bit-identical.
class _FakeClock implements Clock {
  int now = 0;
  @override
  int nowMs() => now;
}

ScoreClocks c(double ms) => (emission: ms, heard: ms);

void main() {
  group('shiftClocksForInput', () {
    test('shifts both clocks earlier by the offset', () {
      final shifted = shiftClocksForInput((
        emission: 1000.0,
        heard: 900.0,
      ), 130);
      expect(shifted.emission, 870.0);
      expect(shifted.heard, 770.0);
    });

    test('a zero offset is the identity', () {
      const clocks = (emission: 1000.0, heard: 900.0);
      expect(shiftClocksForInput(clocks, 0), clocks);
    });
  });

  group('micFreeRunGate', () {
    MicFreeRunGate gateFor(double? measured) {
      final container = ProviderContainer(
        overrides: [measuredInputOffsetMsProvider.overrideWithValue(measured)],
      );
      addTearDown(container.dispose);
      return container.read(micFreeRunGateProvider);
    }

    test('no measurement gates to calibration', () {
      expect(gateFor(null), MicFreeRunGate.needsCalibration);
    });

    test('a fit measurement opens free-run', () {
      // 80 + the 46 ms confirmation window = 126 ≤ the 160 ms bind window.
      expect(gateFor(80), MicFreeRunGate.ok);
    });

    test('an unfit measurement stays gated', () {
      // 130 + 46 = 176 > 160: honest devices stay on Wait Mode.
      expect(gateFor(130), MicFreeRunGate.latencyTooHigh);
    });
  });

  group('audio-sourced runs in the scorer', () {
    late ProviderContainer container;

    PerformanceScorer scorer() =>
        container.read(performanceScorerProvider.notifier);
    ScoringData read() => container.read(performanceScorerProvider);

    setUp(() {
      container = ProviderContainer(
        overrides: [clockProvider.overrideWithValue(_FakeClock())],
      );
      addTearDown(container.dispose);
      container.listen(
        performanceScorerProvider,
        (_, _) {},
        fireImmediately: true,
      );
    });

    final notes = [const TimedNote(pitch: 60, startMs: 0, durationMs: 500)];

    void start({required bool acoustic}) => scorer().startRun(
      pieceId: 'p',
      title: 'Piece',
      hands: 'both',
      speed: 1,
      notes: notes,
      acousticInput: acoustic,
    );

    test('an acoustic run carries no sustain and stamps its source', () {
      start(acoustic: true);
      scorer().noteOn(60, c(0), waitMode: false);
      // Released absurdly early — would crater a keyboard run's sustain.
      scorer().noteOff(60, c(10));
      scorer().finishRun(c(600), waitMode: false);

      final result = read().lastResult!;
      expect(result.inputSource, 'microphone');
      expect(result.sustain, isNull);
      for (final j in result.notes) {
        expect(j.sustainRatio, isNull);
      }
      // Attack-only blend: a perfect attack scores full despite the release.
      expect(result.overallSyncPct, 100.0);
    });

    test('a MIDI run keeps its sustain dimension and stamp', () {
      start(acoustic: false);
      scorer().noteOn(60, c(0), waitMode: false);
      scorer().noteOff(60, c(10));
      scorer().finishRun(c(600), waitMode: false);

      final result = read().lastResult!;
      expect(result.inputSource, 'midi');
      expect(result.sustain, isNotNull);
      // The early release costs sustain — the dimension exists and judged it.
      expect(result.overallSyncPct, lessThan(100.0));
    });
  });

  group('SessionResult json roundtrip', () {
    test('the input source survives, and legacy records read as MIDI', () {
      final result = SessionResult.fromJudgments(
        pieceId: 'p',
        title: 'T',
        hands: 'both',
        judgments: const [],
        bestCombo: 0,
        playedAtMs: 1,
        speed: 1,
        acousticInput: true,
      );
      final restored = SessionResult.fromJson(result.toJson());
      expect(restored.inputSource, 'microphone');

      final legacy = Map<String, dynamic>.from(result.toJson())
        ..remove('inputSource');
      expect(SessionResult.fromJson(legacy).inputSource, 'midi');
    });
  });
}
