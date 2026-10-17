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
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/haptics_service.dart';
import 'package:music/theme/motion.dart';
import 'package:music/widgets/celebration_surface.dart';

import 'celebration_surface_test.mocks.dart';

@GenerateNiceMocks([MockSpec<HapticsService>()])
Future<MockHapticsService> _pump(
  WidgetTester tester, {
  required VoidCallback onAction,
  bool reduced = false,
  bool haptic = true,
  int? counterTo,
}) async {
  final haptics = MockHapticsService();
  when(haptics.light()).thenAnswer((_) async {});
  await tester.pumpWidget(
    ProviderScope(
      overrides: [hapticsServiceProvider.overrideWithValue(haptics)],
      child: MediaQuery(
        data: MediaQueryData(disableAnimations: reduced),
        child: MaterialApp(
          home: Scaffold(
            body: Center(
              child: CelebrationSurface(
                icon: Icons.school,
                title: 'Lesson complete!',
                subtitle: 'The staff',
                haptic: haptic,
                counterTo: counterTo,
                counterFrom: counterTo == null ? null : 0,
                counterFormat: (v) => '$v pts',
                actions: [
                  FilledButton(
                    key: const Key('act'),
                    onPressed: onAction,
                    child: const Text('Go'),
                  ),
                ],
              ),
            ),
          ),
        ),
      ),
    ),
  );
  return haptics;
}

void main() {
  group('celebration surface', () {
    testWidgets('actions are inert mid-sequence and live once it is over', (
      tester,
    ) async {
      var taps = 0;
      await _pump(tester, onAction: () => taps++);
      await tester.pump(Motion.celebrate ~/ 3);
      await tester.tap(find.byKey(const Key('act')), warnIfMissed: false);
      await tester.pump();
      expect(taps, 0, reason: 'nothing can be tapped mid-flight');

      await tester.pumpAndSettle();
      await tester.tap(find.byKey(const Key('act')));
      expect(taps, 1);
      expect(find.text('Lesson complete!'), findsOneWidget);
      expect(find.text('The staff'), findsOneWidget);
    });

    testWidgets('one light haptic as the sequence starts, none when off', (
      tester,
    ) async {
      final h = await _pump(tester, onAction: () {});
      await tester.pumpAndSettle();
      verify(h.light()).called(1);

      final quiet = await _pump(tester, onAction: () {}, haptic: false);
      await tester.pumpAndSettle();
      verifyNever(quiet.light());
    });

    testWidgets('reduced motion paints the final frame at once', (
      tester,
    ) async {
      var taps = 0;
      await _pump(
        tester,
        onAction: () => taps++,
        reduced: true,
        counterTo: 120,
      );
      await tester.pump();
      // Already at the end: the counter reads its final value and the action
      // answers on the very first frame.
      expect(find.text('120 pts'), findsOneWidget);
      await tester.tap(find.byKey(const Key('act')));
      expect(taps, 1);
    });

    testWidgets('the counter counts up to its value by the end', (
      tester,
    ) async {
      await _pump(tester, onAction: () {}, counterTo: 120);
      expect(find.byKey(const Key('celebration-counter')), findsOneWidget);
      await tester.pumpAndSettle();
      expect(find.text('120 pts'), findsOneWidget);
    });

    testWidgets('never overflows on a short landscape-phone viewport', (
      tester,
    ) async {
      tester.view.physicalSize = const Size(852, 393);
      tester.view.devicePixelRatio = 1.0;
      addTearDown(tester.view.reset);
      await _pump(tester, onAction: () {}, counterTo: 42);
      await tester.pumpAndSettle();
      expect(tester.takeException(), isNull);
    });
  });
}
