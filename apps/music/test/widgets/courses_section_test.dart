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
import 'package:music/screens/learning_path_screen.dart';
import 'package:music/screens/lesson_player_screen.dart';
import 'package:music/services/course_catalog_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/widgets/courses_section.dart';

import '../support/fakes.dart';
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
    id: 'sol-u1-01',
    schemaVersion: 2,
    track: 'solfege',
    level: 'beginner',
    unit: 'u1',
    unitTitle: {'en': 'First notes'},
    sortOrder: 101,
    title: {'en': 'Reading the staff'},
  ),
  CourseListing(
    id: 'sol-u1-02',
    schemaVersion: 2,
    track: 'solfege',
    level: 'beginner',
    unit: 'u1',
    unitTitle: {'en': 'First notes'},
    sortOrder: 102,
    title: {'en': 'The treble clef'},
  ),
];

Future<void> _pump(
  WidgetTester tester, {
  List<CourseListing> listings = _listings,
  FakePreferencesService? prefs,
}) async {
  final midi = FakeMidiService();
  addTearDown(midi.close);
  final container = ProviderContainer(
    overrides: [
      midiServiceProvider.overrideWithValue(midi),
      courseCatalogServiceProvider.overrideWithValue(
        _FakeService(listings, const {
          'sol-u1-01':
              '{"schemaVersion":1,"id":"sol-u1-01","blocks":['
              '{"type":"text","text":{"en":"hello"}}]}',
          'sol-u1-02':
              '{"schemaVersion":1,"id":"sol-u1-02","blocks":['
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
  testWidgets('shows the continue card above the favorites', (tester) async {
    await _pump(tester);
    expect(find.text('Courses'), findsOneWidget);
    // Nothing completed yet → the first lesson is up next, with its unit.
    final card = find.byKey(const Key('courses-continue-card'));
    expect(card, findsOneWidget);
    expect(
      find.descendant(of: card, matching: find.text('Reading the staff')),
      findsOneWidget,
    );
    expect(
      find.descendant(of: card, matching: find.text('First notes')),
      findsOneWidget,
    );
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

  testWidgets('the card carries the next uncompleted lesson', (tester) async {
    await _pump(
      tester,
      prefs: FakePreferencesService({'courses.completed.v1': '["sol-u1-01"]'}),
    );
    final card = find.byKey(const Key('courses-continue-card'));
    expect(
      find.descendant(of: card, matching: find.text('The treble clef')),
      findsOneWidget,
    );
    expect(
      find.descendant(of: card, matching: find.text('Reading the staff')),
      findsNothing,
    );
  });

  testWidgets(
    'a fully completed path falls back to the first lesson (replayable)',
    (tester) async {
      await _pump(
        tester,
        prefs: FakePreferencesService({
          'courses.completed.v1': '["sol-u1-01","sol-u1-02"]',
        }),
      );
      final card = find.byKey(const Key('courses-continue-card'));
      expect(
        find.descendant(of: card, matching: find.text('Reading the staff')),
        findsOneWidget,
      );
    },
  );

  testWidgets('tapping the card opens the lesson player', (tester) async {
    await _pump(tester);
    await tester.tap(find.byKey(const Key('courses-continue-card')));
    await tester.pumpAndSettle();
    expect(find.byType(LessonPlayerScreen), findsOneWidget);
  });

  testWidgets('the header link opens the learning path', (tester) async {
    await _pump(tester);
    await tester.tap(find.byKey(const Key('courses-see-path')));
    // The path's next-up node pulses forever, so settle with bounded pumps.
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));
    expect(find.byType(LearningPathScreen), findsOneWidget);
  });

  testWidgets('a v3 course never renders a dead tile', (tester) async {
    await _pump(
      tester,
      listings: [
        ..._listings,
        const CourseListing(
          id: 'sol-future',
          schemaVersion: 3,
          sortOrder: 1,
          title: {'en': 'From the future'},
        ),
      ],
    );
    // The unsupported listing is filtered before the section ever sees it.
    expect(find.text('From the future'), findsNothing);
    expect(find.byKey(const Key('courses-continue-card')), findsOneWidget);
  });
}
