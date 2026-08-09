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
import 'package:music/screens/lesson_player_screen.dart';
import 'package:music/services/course_catalog_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/widgets/courses_section.dart';

import '../support/prefs_fakes.dart';

class _FakeService implements CourseCatalogService {
  _FakeService(this.listings, this.manifests);
  final List<CourseListing> listings;
  final Map<String, String> manifests;
  @override
  Future<List<CourseListing>> listCourses() async => listings;
  @override
  Future<String?> getCourseManifestJson(String id) async => manifests[id];
}

const _listings = [
  CourseListing(
    id: 'sol-portee',
    schemaVersion: 1,
    track: 'solfege',
    level: 'beginner',
    sortOrder: 1,
    title: {'en': 'Reading the staff'},
  ),
  CourseListing(
    id: 'app-synthesia',
    schemaVersion: 1,
    track: 'app-usage',
    level: 'beginner',
    sortOrder: 1,
    title: {'en': 'Synthesia mode'},
  ),
];

Future<void> _pump(
  WidgetTester tester, {
  List<CourseListing> listings = _listings,
  FakePreferencesService? prefs,
}) async {
  final container = ProviderContainer(
    overrides: [
      courseCatalogServiceProvider.overrideWithValue(
        _FakeService(listings, const {
          'sol-portee':
              '{"schemaVersion":1,"id":"sol-portee","blocks":['
              '{"type":"text","text":{"en":"hello"}}]}',
        }),
      ),
      preferencesServiceProvider.overrideWithValue(
        prefs ?? FakePreferencesService(),
      ),
    ],
  );
  addTearDown(container.dispose);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: MaterialApp(
        localizationsDelegates: AppLocalizations.localizationsDelegates,
        supportedLocales: AppLocalizations.supportedLocales,
        locale: const Locale('en'),
        home: const Scaffold(
          body: Column(
            children: [
              CoursesSection(),
              Expanded(child: Center(child: Text('favorites'))),
            ],
          ),
        ),
      ),
    ),
  );
  await tester.pumpAndSettle();
}

void main() {
  testWidgets('renders a tile per course above the favorites', (tester) async {
    await _pump(tester);
    expect(find.text('Courses'), findsOneWidget);
    expect(find.byKey(const Key('course-tile-sol-portee')), findsOneWidget);
    expect(find.byKey(const Key('course-tile-app-synthesia')), findsOneWidget);
    // The section sits above the favorites body.
    final sectionY = tester
        .getTopLeft(find.byKey(const Key('courses-section')))
        .dy;
    final favY = tester.getTopLeft(find.text('favorites')).dy;
    expect(sectionY, lessThan(favY));
  });

  testWidgets('omits itself entirely when there are no courses', (
    tester,
  ) async {
    await _pump(tester, listings: const []);
    expect(find.byKey(const Key('courses-section')), findsNothing);
    expect(find.text('Courses'), findsNothing);
  });

  testWidgets('a completed course shows the completion indicator', (
    tester,
  ) async {
    await _pump(
      tester,
      prefs: FakePreferencesService({'courses.completed.v1': '["sol-portee"]'}),
    );
    // The completed tile carries the check; the other doesn't.
    final completedTile = find.byKey(const Key('course-tile-sol-portee'));
    expect(
      find.descendant(
        of: completedTile,
        matching: find.byIcon(Icons.check_circle),
      ),
      findsOneWidget,
    );
    final otherTile = find.byKey(const Key('course-tile-app-synthesia'));
    expect(
      find.descendant(of: otherTile, matching: find.byIcon(Icons.check_circle)),
      findsNothing,
    );
  });

  testWidgets('tapping a tile opens the lesson player', (tester) async {
    await _pump(tester);
    await tester.tap(find.byKey(const Key('course-tile-sol-portee')));
    await tester.pumpAndSettle();
    expect(find.byType(LessonPlayerScreen), findsOneWidget);
  });
}
