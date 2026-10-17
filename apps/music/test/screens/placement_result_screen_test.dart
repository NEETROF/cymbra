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
import 'package:music/screens/placement_result_screen.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/placement_core.dart';
import 'package:music/state/placement_notifier.dart';
import 'package:music/state/rating_activity_notifier.dart' show nowFnProvider;

import '../support/localized.dart';
import '../support/prefs_fakes.dart';

const _units = ['u1', 'u2', 'u3', 'u4'];
const _titles = {
  'u1': {'en': 'First notes'},
  'u2': {'en': 'Rhythm'},
  'u3': {'en': 'The whole staff'},
  'u4': {'en': 'Bass clef'},
};

Future<ProviderContainer> _pump(
  WidgetTester tester, {
  PlacementOutcome outcome = const PlacementOutcome(
    startUnit: 'u3',
    solved: 5,
    missedStep: 3,
  ),
  String? missedTitle = 'Lines and spaces',
}) async {
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
      nowFnProvider.overrideWithValue(() => DateTime(2026, 10, 17, 9)),
    ],
  );
  addTearDown(c.dispose);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: c,
      child: localizedApp(
        Builder(
          builder: (context) => Scaffold(
            body: Center(
              child: TextButton(
                key: const Key('go'),
                onPressed: () => Navigator.of(context).push(
                  MaterialPageRoute<void>(
                    builder: (_) => PlacementResultScreen(
                      outcome: outcome,
                      units: _units,
                      unitTitles: _titles,
                      missedTitle: missedTitle,
                    ),
                  ),
                ),
                child: const Text('home'),
              ),
            ),
          ),
        ),
      ),
    ),
  );
  await tester.tap(find.byKey(const Key('go')));
  await tester.pumpAndSettle();
  return c;
}

void main() {
  group('placement result screen', () {
    testWidgets('proposes the unit, names the miss, confirms the pointer', (
      tester,
    ) async {
      final c = await _pump(tester);
      expect(find.text('Unit 3'), findsOneWidget);
      expect(find.text('The whole staff'), findsOneWidget);
      expect(find.textContaining('5 exercises solved'), findsOneWidget);
      expect(
        find.textContaining('missed at “Lines and spaces”'),
        findsOneWidget,
      );
      await tester.tap(find.byKey(const Key('placement-confirm')));
      await tester.pumpAndSettle();
      expect(c.read(placementProvider).startUnit, 'u3');
      expect(find.byKey(const Key('placement-result')), findsNothing);
      expect(find.text('home'), findsOneWidget);
    });

    testWidgets('one down, one up, or any chip — before confirming', (
      tester,
    ) async {
      final c = await _pump(tester);
      await tester.tap(find.byKey(const Key('placement-minus')));
      await tester.pump();
      expect(find.text('Unit 2'), findsOneWidget);
      expect(find.text('Start at unit 2'), findsOneWidget);
      await tester.tap(find.byKey(const Key('placement-plus')));
      await tester.pump();
      expect(find.text('Unit 3'), findsOneWidget);
      await tester.tap(find.byKey(const Key('placement-unit-u1')));
      await tester.pump();
      expect(find.text('Unit 1'), findsOneWidget);
      expect(
        tester
            .widget<IconButton>(find.byKey(const Key('placement-minus')))
            .onPressed,
        isNull,
        reason: 'nothing below unit 1',
      );
      await tester.tap(find.byKey(const Key('placement-confirm')));
      await tester.pumpAndSettle();
      expect(c.read(placementProvider).startUnit, 'u1');
    });

    testWidgets('everything solved reads so, and the plus stops at the end', (
      tester,
    ) async {
      await _pump(
        tester,
        outcome: const PlacementOutcome(
          startUnit: 'u4',
          solved: 7,
          missedStep: null,
        ),
        missedTitle: null,
      );
      expect(find.text('Unit 4'), findsOneWidget);
      expect(find.textContaining('everything solved'), findsOneWidget);
      expect(
        tester
            .widget<IconButton>(find.byKey(const Key('placement-plus')))
            .onPressed,
        isNull,
      );
    });

    testWidgets('start from the beginning clears the pointer', (tester) async {
      final c = await _pump(tester);
      await c.read(placementProvider.notifier).set('u2');
      await tester.tap(find.byKey(const Key('placement-restart')));
      await tester.pumpAndSettle();
      expect(c.read(placementProvider).startUnit, isNull);
      expect(c.read(placementProvider).taken, isTrue);
      expect(find.text('home'), findsOneWidget);
    });
  });
}
