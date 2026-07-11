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

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/clock_service.dart';
import 'package:music/state/performance_scoring.dart';
import 'package:music/state/player_data.dart';
import 'package:music/widgets/scoring_gauge.dart';

import '../support/localized.dart';

class _FakeClock implements Clock {
  int now = 0;
  @override
  int nowMs() => now;
}

void main() {
  testWidgets(
    'gauge is hidden until a scored run is active, then shows the %',
    (tester) async {
      final container = ProviderContainer(
        overrides: [clockProvider.overrideWithValue(_FakeClock())],
      );
      addTearDown(container.dispose);

      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: container,
          child: localizedApp(const Scaffold(body: ScoringGauge())),
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

      // Active run → the gauge shows the baseline 100%.
      expect(find.textContaining('%'), findsOneWidget);
      expect(find.text('100%'), findsOneWidget);

      container.read(performanceScorerProvider.notifier).cancelRun();
      await tester.pump();

      // Cancelled → hidden again.
      expect(find.textContaining('%'), findsNothing);
    },
  );
}
