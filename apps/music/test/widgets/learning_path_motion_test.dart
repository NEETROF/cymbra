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
import 'package:music/courses/course_manifest.dart';
import 'package:music/l10n/gen/app_localizations.dart';
import 'package:music/services/course_catalog_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/course_completion_notifier.dart';
import 'package:music/theme/motion.dart';
import 'package:music/widgets/learning_path_body.dart';

import '../support/prefs_fakes.dart';

class _FakeService implements CourseCatalogService {
  _FakeService(this.listings);
  final List<CourseListing> listings;
  @override
  Future<List<CourseListing>> listCourses() async => listings;
  @override
  Future<String?> getCourseManifestJson(String id) async => null;
}

CourseListing _lesson(String id, String unit, int order) => CourseListing(
  id: id,
  schemaVersion: 2,
  unit: unit,
  unitTitle: {'en': unit},
  sortOrder: order,
  title: {'en': 'Lesson $id'},
);

final _listings = [
  _lesson('c1', 'u1', 101),
  _lesson('c2', 'u1', 102),
  _lesson('c3', 'u2', 201),
];

Future<ProviderContainer> _pump(
  WidgetTester tester, {
  bool reduced = false,
}) async {
  final container = ProviderContainer(
    overrides: [
      courseCatalogServiceProvider.overrideWithValue(_FakeService(_listings)),
      preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
    ],
  );
  addTearDown(container.dispose);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: MediaQuery(
        data: MediaQueryData(disableAnimations: reduced),
        child: MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          locale: const Locale('en'),
          home: const Scaffold(
            body: SingleChildScrollView(child: LearningPathBody()),
          ),
        ),
      ),
    ),
  );
  await tester.pump();
  await tester.pump();
  return container;
}

/// The scale of the icon inside node [id]'s circle.
double _checkScale(WidgetTester tester, String id) => tester
    .widget<ScaleTransition>(
      find.descendant(
        of: find.byKey(Key('path-node-$id')),
        matching: find.byType(ScaleTransition),
      ),
    )
    .scale
    .value;

void main() {
  group('learning path motion', () {
    testWidgets('node rows keep the fixed pitch the curve is drawn from', (
      tester,
    ) async {
      await _pump(tester);
      double top(String id) =>
          tester.getTopLeft(find.byKey(Key('path-node-$id'))).dy;
      expect(top('c2') - top('c1'), kPathNodeRowHeight);
      expect(find.byKey(const Key('path-curve-u1')), findsOneWidget);
      expect(find.byKey(const Key('path-curve-u2')), findsOneWidget);
    });

    testWidgets('the check pops in when a node becomes done', (tester) async {
      final c = await _pump(tester);
      expect(_checkScale(tester, 'c1'), 1.0, reason: 'settled while not done');

      await c.read(courseCompletionProvider.notifier).markCompleted('c1');
      await tester.pump();
      expect(_checkScale(tester, 'c1'), lessThan(0.5), reason: 'popping in');
      await tester.pump(Motion.quick);
      await tester.pump();
      expect(_checkScale(tester, 'c1'), closeTo(1.0, 0.01));
    });

    testWidgets('reduced motion: the check is simply there', (tester) async {
      final c = await _pump(tester, reduced: true);
      await c.read(courseCompletionProvider.notifier).markCompleted('c1');
      await tester.pump();
      await tester.pump();
      expect(_checkScale(tester, 'c1'), 1.0);
    });

    testWidgets('a finished unit lights its header once per app session', (
      tester,
    ) async {
      final c = await _pump(tester);
      expect(c.read(litUnitsProvider), isEmpty);

      await c.read(courseCompletionProvider.notifier).markCompleted('c1');
      await tester.pump();
      expect(
        c.read(litUnitsProvider),
        isEmpty,
        reason: 'one lesson of two: the unit is not finished',
      );

      await c.read(courseCompletionProvider.notifier).markCompleted('c2');
      await tester.pump();
      await tester.pump();
      expect(c.read(litUnitsProvider), {'u1'});
      await tester.pump(const Duration(milliseconds: 700));
      await tester.pump();

      // Rebuilding the path does not light it again.
      await tester.pumpWidget(const SizedBox());
      await tester.pump();
      await tester.pumpWidget(
        UncontrolledProviderScope(
          container: c,
          child: MaterialApp(
            localizationsDelegates: AppLocalizations.localizationsDelegates,
            supportedLocales: AppLocalizations.supportedLocales,
            locale: const Locale('en'),
            home: const Scaffold(
              body: SingleChildScrollView(child: LearningPathBody()),
            ),
          ),
        ),
      );
      await tester.pump();
      await tester.pump();
      expect(c.read(litUnitsProvider), {'u1'});
      expect(tester.takeException(), isNull);
    });
  });
}
