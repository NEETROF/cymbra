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
import 'package:music/courses/course_manifest.dart';
import 'package:music/services/course_catalog_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/rating_activity_notifier.dart' show nowFnProvider;
import 'package:music/state/session_notifier.dart';
import 'package:music/widgets/daily_session_block.dart';
import 'package:music/widgets/learning_path_body.dart';

import '../support/daily_fixtures.dart';
import '../support/localized.dart';
import '../support/prefs_fakes.dart';

ProviderContainer _container(
  Map<String, String> prefs, {
  CourseCatalogService catalog = const DailyFakeCatalog(),
}) {
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(
        FakePreferencesService(prefs),
      ),
      nowFnProvider.overrideWithValue(() => DateTime(2026, 10, 17, 9)),
      courseCatalogServiceProvider.overrideWithValue(catalog),
      canUseOnlineServicesProvider.overrideWithValue(false),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

Future<void> _pump(
  WidgetTester tester,
  ProviderContainer c,
  Widget home,
) async {
  tester.view.physicalSize = const Size(1000, 1200);
  tester.view.devicePixelRatio = 1.0;
  addTearDown(tester.view.reset);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: c,
      child: localizedApp(Scaffold(body: SingleChildScrollView(child: home))),
    ),
  );
  for (var i = 0; i < 8; i++) {
    await tester.pump(const Duration(milliseconds: 100));
  }
}

int _filledIn(WidgetTester tester, Key key) => tester
    .widgetList<Icon>(
      find.descendant(of: find.byKey(key), matching: find.byType(Icon)),
    )
    .where((i) => i.icon == Icons.star_rounded)
    .length;

void main() {
  group('stars on the path and in today\'s session', () {
    testWidgets('a piece lesson node shows its best run\'s stars; a lesson '
        'never run shows none', (tester) async {
      final c = _container({
        'courses.best.v1': jsonEncode({'a3': 85}),
        'courses.passed.v1': jsonEncode(['a3']),
      });
      await _pump(tester, c, const LearningPathBody());
      expect(find.byKey(const Key('path-stars-a3')), findsOneWidget);
      expect(_filledIn(tester, const Key('path-stars-a3')), 2);
      expect(find.byKey(const Key('path-stars-a1')), findsNothing);
    });

    testWidgets('today\'s piece item carries the lesson\'s stars — empty '
        'before any run', (tester) async {
      // Mid-path: the piece item is b2's own step, never run.
      final c = _container({
        'courses.completed.v1': jsonEncode(['a1', 'a2', 'b1']),
        'courses.passed.v1': jsonEncode(['a3']),
        'courses.best.v1': jsonEncode({'a3': 91}),
      });
      await _pump(tester, c, const DailySessionBlock());
      expect(find.byKey(const Key('daily-piece-stars')), findsOneWidget);
      expect(_filledIn(tester, const Key('daily-piece-stars')), 0);
    });

    testWidgets('a visited step at 65 % shows one star on today\'s piece', (
      tester,
    ) async {
      final c = _container({
        'courses.completed.v1': jsonEncode(['a1', 'a2', 'b1']),
        'courses.passed.v1': jsonEncode(['a3']),
        'courses.best.v1': jsonEncode({'a3': 91, 'b2': 65}),
      });
      await _pump(tester, c, const DailySessionBlock());
      expect(_filledIn(tester, const Key('daily-piece-stars')), 1);
    });

    testWidgets('a starred node with a long title keeps to its row (it '
        'overflowed while dogfooding)', (tester) async {
      final c = _container(
        {
          'courses.best.v1': jsonEncode({'a3': 85}),
        },
        catalog: DailyFakeCatalog([
          dailyListing('a1', 'u1', 101),
          const CourseListing(
            id: 'a3',
            schemaVersion: 2,
            unit: 'u1',
            unitTitle: {'en': 'Unit u1'},
            sortOrder: 103,
            title: {
              'en':
                  'Your turn: Au clair de la lune, the whole tune with the '
                  'right hand',
            },
          ),
        ]),
      );
      await _pump(tester, c, const LearningPathBody());
      expect(find.byKey(const Key('path-stars-a3')), findsOneWidget);
      expect(tester.takeException(), isNull);
    });
  });
}
