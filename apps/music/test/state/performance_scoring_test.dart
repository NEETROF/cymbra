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
import 'package:music/state/performance_scoring.dart';
import 'package:music/state/performance_scoring_core.dart';
import 'package:music/state/player_data.dart';

/// Deterministic clock the tests advance by hand (no flaky sleeps).
class FakeClock implements Clock {
  int now = 0;
  @override
  int nowMs() => now;
}

void main() {
  late FakeClock clock;
  late ProviderContainer container;

  PerformanceScorer scorer() =>
      container.read(performanceScorerProvider.notifier);
  ScoringData read() => container.read(performanceScorerProvider);

  setUp(() {
    clock = FakeClock();
    container = ProviderContainer(
      overrides: [clockProvider.overrideWithValue(clock)],
    );
    addTearDown(container.dispose);
    container.listen(performanceScorerProvider, (_, _) {}, fireImmediately: true);
  });

  // C4 @ [0,500), D4 @ [500,1000).
  final notes = [
    const TimedNote(pitch: 60, startMs: 0, durationMs: 500),
    const TimedNote(pitch: 62, startMs: 500, durationMs: 500),
  ];

  void start() => scorer().startRun(
        pieceId: 'p',
        title: 'Piece',
        hands: 'both',
        speed: 1,
        notes: notes,
      );

  group('activation / gating', () {
    test('events are no-ops before a run starts', () {
      scorer().noteOn(60, 0, waitMode: false);
      expect(read().active, isFalse);
      expect(read().lastResult, isNull);
    });

    test('startRun activates with a defined baseline', () {
      start();
      expect(read().active, isTrue);
      expect(read().syncPercent, 100);
      expect(read().tier, 4);
    });

    test('cancelRun deactivates without a result', () {
      start();
      scorer().cancelRun();
      expect(read().active, isFalse);
      expect(read().lastResult, isNull);
    });
  });

  group('free-run judging', () {
    test('on-time press binds as perfect and grows the combo', () {
      start();
      scorer().noteOn(60, 0, waitMode: false);
      expect(read().combo, 1);
      expect(read().syncPercent, 100);
      expect(read().recentHits.last.verdict, TimingVerdict.perfect);
    });

    test('unplayed onset is missed once its window passes', () {
      start();
      scorer().noteOn(60, 0, waitMode: false); // play C4 on time
      scorer().noteOff(60, 500);
      // Push the playhead well past D4's bind window without playing it.
      scorer().tick(700, waitMode: false);
      scorer().finishRun(1000, waitMode: false);
      final r = read().lastResult!;
      expect(r.verdictCounts[TimingVerdict.missed], 1);
      final d4 = r.notes.firstWhere((n) => n.pitch == 62);
      expect(d4.verdict, TimingVerdict.missed);
    });

    test('a wrong note is recorded and resets the combo', () {
      start();
      scorer().noteOn(60, 0, waitMode: false); // good, combo 1
      scorer().noteOn(99, 10, waitMode: false); // no such onset → wrong
      expect(read().combo, 0);
      scorer().finishRun(1000, waitMode: false);
      expect(read().lastResult!.wrongNotes, 1);
    });

    test('sustain is finalized from the release position', () {
      start();
      scorer().noteOn(60, 0, waitMode: false);
      scorer().noteOff(60, 60); // released after ~12% of the 500ms note
      scorer().finishRun(1000, waitMode: false);
      final hit = read().lastResult!.notes.firstWhere((n) => n.noteIndex == 0);
      expect(hit.sustainRatio, lessThan(0.5));
    });
  });

  group('wait-mode reaction timing', () {
    test('reaction time is measured from gate-open to attack', () {
      start();
      clock.now = 1000;
      scorer().tick(0, waitMode: true); // gate opens on C4 @ 0
      clock.now = 1080; // 80ms reaction → perfect
      scorer().noteOn(60, 0, waitMode: true);
      final hit = read().recentHits.last;
      expect(hit.verdict, TimingVerdict.perfect);
    });

    test('slow reaction scores lower', () {
      start();
      clock.now = 1000;
      scorer().tick(0, waitMode: true);
      clock.now = 1400; // 400ms → late
      scorer().noteOn(60, 0, waitMode: true);
      expect(read().recentHits.last.verdict, TimingVerdict.late);
    });

    test('exploratory press away from a gate is ignored, not wrong', () {
      start();
      // No gate open at playhead 250 (between onsets); a stray press is ignored.
      scorer().noteOn(64, 250, waitMode: true);
      scorer().finishRun(1000, waitMode: true);
      expect(read().lastResult!.wrongNotes, 0);
    });
  });

  group('mode stamping across a mid-run toggle', () {
    test('run mixes free and wait onsets and yields both sub-scores', () {
      start();
      // C4 played in free run.
      scorer().noteOn(60, 0, waitMode: false);
      scorer().noteOff(60, 500);
      // D4 played under Wait Mode (toggled on mid-run).
      clock.now = 2000;
      scorer().tick(500, waitMode: true); // gate opens on D4 @ 500
      clock.now = 2050;
      scorer().noteOn(62, 500, waitMode: true);
      scorer().finishRun(1000, waitMode: true);

      final r = read().lastResult!;
      expect(r.runMode, RunMode.mixed);
      expect(r.freeSyncPct, isNotNull);
      expect(r.waitSyncPct, isNotNull);
      expect(r.freeOnsetCount, 1);
      expect(r.waitOnsetCount, 1);
      // The two onsets are stamped with the mode active when each was judged.
      final c4 = r.notes.firstWhere((n) => n.pitch == 60);
      final d4 = r.notes.firstWhere((n) => n.pitch == 62);
      expect(c4.waitMode, isFalse);
      expect(d4.waitMode, isTrue);
    });

    test('pure free run classifies free with no wait sub-score', () {
      start();
      scorer().noteOn(60, 0, waitMode: false);
      scorer().noteOn(62, 500, waitMode: false);
      scorer().finishRun(1000, waitMode: false);
      final r = read().lastResult!;
      expect(r.runMode, RunMode.free);
      expect(r.waitSyncPct, isNull);
    });
  });

  group('finish', () {
    test('finishRun produces a result and deactivates', () {
      start();
      scorer().noteOn(60, 0, waitMode: false);
      scorer().finishRun(1000, waitMode: false);
      expect(read().active, isFalse);
      expect(read().lastResult, isNotNull);
      expect(read().bestCombo, greaterThanOrEqualTo(1));
    });

    test('clearLastResult drops the stored result', () {
      start();
      scorer().finishRun(1000, waitMode: false);
      expect(read().lastResult, isNotNull);
      scorer().clearLastResult();
      expect(read().lastResult, isNull);
    });
  });
}
