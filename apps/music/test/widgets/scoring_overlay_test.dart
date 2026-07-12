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
import 'package:music/painters/hit_effects_painter.dart';
import 'package:music/painters/piano_layout.dart';
import 'package:music/services/clock_service.dart';
import 'package:music/state/performance_scoring.dart';
import 'package:music/state/player_data.dart';
import 'package:music/widgets/scoring_overlay.dart';

import '../support/localized.dart';

class _FakeClock implements Clock {
  @override
  int nowMs() => 0;
}

final _hitEffects = find.byWidgetPredicate(
  (w) => w is CustomPaint && w.painter is HitEffectsPainter,
);

void main() {
  const layout = PianoLayout(width: 300, lowPitch: 21, highPitch: 108);

  Future<ProviderContainer> pumpOverlay(
    WidgetTester tester, {
    required bool showEffects,
    bool withLayout = true,
  }) async {
    final container = ProviderContainer(
      overrides: [clockProvider.overrideWithValue(_FakeClock())],
    );
    addTearDown(container.dispose);
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: localizedApp(
          Scaffold(
            body: ScoringOverlay(
              layout: withLayout ? layout : null,
              showEffects: showEffects,
            ),
          ),
        ),
      ),
    );
    // Start the run once the widget is mounted (keeps the auto-dispose scorer
    // alive, so no stray disposal timer is left pending).
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
    return container;
  }

  testWidgets('hides the hit sparks but keeps the gauge when effects are off', (
    tester,
  ) async {
    await pumpOverlay(tester, showEffects: false);
    // No spark layer (keyboard hidden), but the gauge still reads the sync %.
    expect(_hitEffects, findsNothing);
    expect(find.textContaining('%'), findsOneWidget);
  });

  testWidgets('with no layout (Partition) shows the gauge but no sparks', (
    tester,
  ) async {
    await pumpOverlay(tester, showEffects: true, withLayout: false);
    expect(_hitEffects, findsNothing);
    expect(find.textContaining('%'), findsOneWidget);
  });
}
