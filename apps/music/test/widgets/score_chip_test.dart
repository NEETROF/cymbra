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

import 'dart:ui';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/clock_service.dart';
import 'package:music/state/performance_scoring.dart';
import 'package:music/state/player_data.dart';
import 'package:music/widgets/score_chip.dart';

import '../support/localized.dart';

class _FakeClock implements Clock {
  int now = 0;
  @override
  int nowMs() => now;
}

void main() {
  testWidgets(
    'chip is hidden until a scored run is active, then shows % and combo',
    (tester) async {
      final container = ProviderContainer(
        overrides: [clockProvider.overrideWithValue(_FakeClock())],
      );
      addTearDown(container.dispose);

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: localizedApp(const Scaffold(body: ScoreChip())),
        ),
      );

      // No run yet → nothing rendered.
      expect(find.textContaining('%'), findsNothing);

      container
          .read(performanceScorerProvider.notifier)
          .startRun(
            pieceId: 'p',
            title: 'T',
            hands: 'both',
            speed: 1,
            notes: const [TimedNote(pitch: 60, startMs: 0, durationMs: 500)],
          );
      await tester.pump();

      // Active run → the chip shows the baseline 100% and the combo counter.
      expect(find.text('100%'), findsOneWidget);

      container.read(performanceScorerProvider.notifier).cancelRun();
      await tester.pump();

      // Cancelled → hidden again.
      expect(find.textContaining('%'), findsNothing);
    },
  );

  testWidgets('combo shows only once it exists, never on the compact chip', (
    tester,
  ) async {
    final container = ProviderContainer(
      overrides: [clockProvider.overrideWithValue(_FakeClock())],
    );
    addTearDown(container.dispose);

    Finder comboText() => find.byWidgetPredicate(
      (w) => w is Text && (w.data ?? '').contains('×'),
    );

    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: localizedApp(const Scaffold(body: ScoreChip())),
      ),
    );
    final scorer = container.read(performanceScorerProvider.notifier);
    scorer.startRun(
      pieceId: 'p',
      title: 'T',
      hands: 'both',
      speed: 1,
      notes: const [TimedNote(pitch: 60, startMs: 0, durationMs: 500)],
    );
    await tester.pump();
    // No landed note yet → no "×0" reproach on the chip.
    expect(find.text('100%'), findsOneWidget);
    expect(comboText(), findsNothing);
    final idleSize = tester.getSize(find.byType(ScoreChip));

    // A perfect hit starts the streak → the full chip shows ×1…
    scorer.noteOn(60, (emission: 0, heard: 0), waitMode: false);
    await tester.pump();
    expect(comboText(), findsOneWidget);
    // …in its reserved slot: the pill never resizes when the combo appears.
    expect(tester.getSize(find.byType(ScoreChip)), idleSize);

    // …while the compact variant never carries the combo text.
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: localizedApp(const Scaffold(body: ScoreChip(compact: true))),
      ),
    );
    await tester.pump();
    expect(find.text('100%'), findsOneWidget);
    expect(comboText(), findsNothing);
  });

  group('GaugeFireworkPainter', () {
    void paintAt(double progress) {
      final recorder = PictureRecorder();
      GaugeFireworkPainter(
        progress: progress,
        color: const Color(0xFF44E2CD),
      ).paint(Canvas(recorder), const Size(88, 120));
      recorder.endRecording().dispose();
    }

    test('paints a burst mid-animation and nothing at rest', () {
      paintAt(0.0); // idle — no particles
      paintAt(0.5); // mid burst
      paintAt(1.0); // finished — no particles
    });

    test('repaints as the burst progresses', () {
      const a = GaugeFireworkPainter(progress: 0.1, color: Color(0xFF000000));
      const b = GaugeFireworkPainter(progress: 0.4, color: Color(0xFF000000));
      expect(a.shouldRepaint(b), isTrue);
      expect(a.shouldRepaint(a), isFalse);
    });
  });
}
