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

import '../support/fakes.dart';
import '../support/prefs_fakes.dart';

class _FakeService implements CourseCatalogService {
  _FakeService(this.listings);
  final List<CourseListing> listings;
  @override
  Future<List<CourseListing>> listCourses() async => listings;
  @override
  Future<String?> getCourseManifestJson(String id) async =>
      '{"schemaVersion":1,"id":"$id","blocks":['
      '{"type":"text","text":{"en":"hello"}}]}';
}

CourseListing _lesson(String id, String unit, int order, String title) =>
    CourseListing(
      id: id,
      schemaVersion: 2,
      unit: unit,
      unitTitle: {'en': unit == 'u1' ? 'First notes' : 'Rhythm'},
      sortOrder: order,
      title: {'en': title},
    );

final _listings = [
  _lesson('c1', 'u1', 101, 'The staff'),
  _lesson('c2', 'u1', 102, 'The treble clef'),
  _lesson('c3', 'u2', 201, 'The pulse'),
];

Future<void> _pump(WidgetTester tester, {FakePreferencesService? prefs}) async {
  final midi = FakeMidiService();
  addTearDown(midi.close);
  final container = ProviderContainer(
    overrides: [
      midiServiceProvider.overrideWithValue(midi),
      courseCatalogServiceProvider.overrideWithValue(_FakeService(_listings)),
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
        home: const LearningPathScreen(),
      ),
    ),
  );
  await tester.pump();
  await tester.pump();
}

void main() {
  testWidgets('groups lessons under their unit headers, in order', (
    tester,
  ) async {
    await _pump(tester);
    expect(find.byKey(const Key('path-unit-u1')), findsOneWidget);
    expect(find.byKey(const Key('path-unit-u2')), findsOneWidget);
    expect(find.text('First notes'), findsOneWidget);
    expect(find.text('Rhythm'), findsOneWidget);
    expect(find.byKey(const Key('path-node-c1')), findsOneWidget);
    expect(find.byKey(const Key('path-node-c3')), findsOneWidget);
    // u1 sits above u2.
    expect(
      tester.getTopLeft(find.byKey(const Key('path-unit-u1'))).dy,
      lessThan(tester.getTopLeft(find.byKey(const Key('path-unit-u2'))).dy),
    );
  });

  testWidgets('marks completed lessons and highlights the next one', (
    tester,
  ) async {
    await _pump(
      tester,
      prefs: FakePreferencesService({'courses.completed.v1': '["c1"]'}),
    );
    // c1 done → check icon; c2 is next → play affordance.
    expect(
      find.descendant(
        of: find.byKey(const Key('path-node-c1')),
        matching: find.byIcon(Icons.check),
      ),
      findsOneWidget,
    );
    expect(
      find.descendant(
        of: find.byKey(const Key('path-node-c2')),
        matching: find.byIcon(Icons.play_arrow),
      ),
      findsOneWidget,
    );
    // Unit progress reads 1/2.
    expect(find.text('1/2'), findsOneWidget);
  });

  testWidgets('every node stays tappable — order guides, never locks', (
    tester,
  ) async {
    await _pump(tester);
    // Jump straight to a later unit's lesson.
    await tester.tap(find.byKey(const Key('path-node-c3')));
    await tester.pump();
    await tester.pump();
    expect(find.byType(LessonPlayerScreen), findsOneWidget);
  });
}
