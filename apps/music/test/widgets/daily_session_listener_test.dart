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

import 'dart:convert';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/course_catalog_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/daily_session_core.dart';
import 'package:music/state/daily_session_notifier.dart';
import 'package:music/state/home_door.dart';
import 'package:music/state/rating_activity_notifier.dart' show nowFnProvider;
import 'package:music/state/today_ledger.dart';
import 'package:music/widgets/daily_session_listener.dart';

import '../support/daily_fixtures.dart';
import '../support/localized.dart';
import '../support/prefs_fakes.dart';

/// Unit 1 done (a1, a2 completed): the session is a3 (lesson), a3's own piece
/// step, and a review from unit 1 — three items against the normal goal.
ProviderContainer _container() {
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(
        FakePreferencesService({
          'courses.completed.v1': jsonEncode(['a1', 'a2']),
        }),
      ),
      nowFnProvider.overrideWithValue(() => DateTime(2026, 10, 17, 9)),
      courseCatalogServiceProvider.overrideWithValue(const DailyFakeCatalog()),
      effectiveHomeDoorProvider.overrideWithValue(HomeDoor.learn),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

Future<void> _pump(WidgetTester tester, ProviderContainer c) async {
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: c,
      child: localizedApp(
        DailySessionListener(
          child: Scaffold(
            body: Builder(
              builder: (context) => TextButton(
                key: const Key('go'),
                onPressed: () => Navigator.of(context).push(
                  MaterialPageRoute<void>(
                    builder: (_) => const Scaffold(body: Text('top')),
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
  await tester.pumpAndSettle();
}

String _reviewKey(ProviderContainer c) => c
    .read(dailySessionProvider)
    .requireValue
    .items
    .firstWhere((i) => i.kind == DailyItemKind.review)
    .key;

void main() {
  group('daily session listener', () {
    testWidgets('reaching the goal raises the completion moment once; a '
        'third item does not repeat it', (tester) async {
      final c = _container();
      await _pump(tester, c);
      final ledger = c.read(dailyLedgerProvider.notifier);
      await ledger.markLesson('a3');
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('reward-celebration')), findsNothing);

      await ledger.markPiece('a3');
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('reward-celebration')), findsOneWidget);
      expect(find.text('Session complete!'), findsOneWidget);
      expect(find.textContaining('2 items done today'), findsOneWidget);
      expect(c.read(dailyLedgerProvider).cueRaised, isTrue);
      await tester.tap(find.text('Nice!'));
      await tester.pumpAndSettle();

      await ledger.markReview(_reviewKey(c));
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('reward-celebration')), findsNothing);
      expect(c.read(dailySessionProvider).requireValue.doneCount, 3);
    });

    testWidgets('the moment waits until this route is current again', (
      tester,
    ) async {
      final c = _container();
      await _pump(tester, c);
      final ledger = c.read(dailyLedgerProvider.notifier);
      await ledger.markLesson('a3');
      await tester.pumpAndSettle();
      // Another route on top (the lesson player, in the app).
      await tester.tap(find.byKey(const Key('go')));
      await tester.pumpAndSettle();
      await ledger.markPiece('a3');
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('reward-celebration')), findsNothing);
      expect(find.text('top'), findsOneWidget);

      Navigator.of(tester.element(find.text('top'))).pop();
      await tester.pumpAndSettle();
      expect(find.byKey(const Key('reward-celebration')), findsOneWidget);
    });

    testWidgets('a cue already raised today is not raised again on launch', (
      tester,
    ) async {
      final c = ProviderContainer(
        overrides: [
          preferencesServiceProvider.overrideWithValue(
            FakePreferencesService({
              'courses.completed.v1': jsonEncode(['a1', 'a2']),
              'daily.ledger.v1': jsonEncode({
                'localDay': '2026-10-17',
                'lessonIds': ['a3'],
                'pieceStepKeys': ['a3'],
                'cueRaised': true,
              }),
            }),
          ),
          nowFnProvider.overrideWithValue(() => DateTime(2026, 10, 17, 9)),
          courseCatalogServiceProvider.overrideWithValue(
            const DailyFakeCatalog(),
          ),
          effectiveHomeDoorProvider.overrideWithValue(HomeDoor.learn),
        ],
      );
      addTearDown(c.dispose);
      await _pump(tester, c);
      expect(c.read(dailySessionProvider).requireValue.completedToday, isTrue);
      expect(find.byKey(const Key('reward-celebration')), findsNothing);
    });
  });
}
