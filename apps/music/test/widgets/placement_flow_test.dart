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
import 'package:music/screens/learning_path_screen.dart';
import 'package:music/screens/lesson_player_screen.dart';
import 'package:music/screens/placement_result_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/course_catalog_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/course_completion_notifier.dart';
import 'package:music/state/placement_notifier.dart';
import 'package:music/state/rating_activity_notifier.dart' show nowFnProvider;
import 'package:music/widgets/courses_section.dart';
import 'package:music/widgets/learn_door_body.dart';

import '../support/daily_fixtures.dart';
import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/prefs_fakes.dart';

const _placementListing = CourseListing(
  id: 'placement-piano',
  schemaVersion: 2,
  track: kPlacementTrack,
  sortOrder: 1,
  title: {'en': 'Find your starting point'},
);

Map<String, Object> _q(String unit, String prompt) => {
  'type': 'question',
  'placementUnit': unit,
  'prompt': {'en': prompt},
  'options': [
    {'en': 'right'},
    {'en': 'wrong'},
  ],
  'answerIndex': 0,
};

/// The fixture path plus a placement of three questions, one per unit.
class _Catalog implements CourseCatalogService {
  const _Catalog({this.withPlacement = true});

  final bool withPlacement;

  @override
  Future<List<CourseListing>> listCourses() async => [
    ...dailyListings,
    if (withPlacement) _placementListing,
  ];

  @override
  Future<String?> getCourseManifestJson(String id) async {
    if (id != 'placement-piano') return dailyManifestJson(id);
    return jsonEncode({
      'schemaVersion': 2,
      'id': id,
      'track': 'placement',
      'title': {'en': 'Find your starting point'},
      'blocks': [
        {
          'type': 'text',
          'text': {'en': 'Three quick questions.'},
        },
        _q('u1', 'Unit one?'),
        _q('u2', 'Unit two?'),
        _q('u3', 'Unit three?'),
        {
          'type': 'text',
          'text': {'en': 'Done.'},
        },
      ],
    });
  }
}

ProviderContainer _container({
  Map<String, String> prefs = const {},
  bool withPlacement = true,
}) {
  final midi = FakeMidiService();
  addTearDown(midi.close);
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(
        FakePreferencesService(prefs),
      ),
      nowFnProvider.overrideWithValue(() => DateTime(2026, 10, 17, 9)),
      courseCatalogServiceProvider.overrideWithValue(
        _Catalog(withPlacement: withPlacement),
      ),
      midiServiceProvider.overrideWithValue(midi),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

/// Bounded frames: the path's "next up" node pulses forever, so pumpAndSettle
/// never settles on the Learn door.
Future<void> _settle(WidgetTester tester, [int n = 10]) async {
  for (var i = 0; i < n; i++) {
    await tester.pump(const Duration(milliseconds: 100));
  }
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
    UncontrolledProviderScope(container: c, child: localizedApp(home)),
  );
  await _settle(tester);
}

/// Answers the three placement questions: [right] per question, then Next.
Future<void> _answer(WidgetTester tester, List<bool> right) async {
  for (final r in right) {
    await tester.tap(find.byKey(Key('lesson-option-${r ? 0 : 1}')));
    await _settle(tester);
    await tester.tap(find.byKey(const Key('lesson-next')));
    await _settle(tester);
  }
}

void main() {
  group('the offer (D5)', () {
    testWidgets('first visit: offered above the path, with the header entry', (
      tester,
    ) async {
      await _pump(tester, _container(), const Scaffold(body: LearnDoorBody()));
      expect(find.byKey(const Key('placement-offer')), findsOneWidget);
      expect(find.text('Already play a little?'), findsOneWidget);
      expect(find.byKey(const Key('placement-entry')), findsOneWidget);
    });

    testWidgets('gone once a lesson was completed; the entry stays', (
      tester,
    ) async {
      await _pump(
        tester,
        _container(
          prefs: {
            'courses.completed.v1': jsonEncode(['a1']),
          },
        ),
        const Scaffold(body: LearnDoorBody()),
      );
      expect(find.byKey(const Key('placement-offer')), findsNothing);
      expect(find.byKey(const Key('placement-entry')), findsOneWidget);
    });

    testWidgets('gone once the placement was taken, even cleared', (
      tester,
    ) async {
      await _pump(
        tester,
        _container(
          prefs: {
            'placement.v1': jsonEncode({
              'startUnit': null,
              'takenAtMs': DateTime(2026, 10, 1).millisecondsSinceEpoch,
            }),
          },
        ),
        const Scaffold(body: LearnDoorBody()),
      );
      expect(find.byKey(const Key('placement-offer')), findsNothing);
    });

    testWidgets('"Later" hides it for the session', (tester) async {
      await _pump(tester, _container(), const Scaffold(body: LearnDoorBody()));
      await tester.tap(find.byKey(const Key('placement-offer-later')));
      await _settle(tester);
      expect(find.byKey(const Key('placement-offer')), findsNothing);
    });

    testWidgets('no placement published → no offer, no entry', (tester) async {
      await _pump(
        tester,
        _container(withPlacement: false),
        const Scaffold(body: LearnDoorBody()),
      );
      expect(find.byKey(const Key('placement-offer')), findsNothing);
      expect(find.byKey(const Key('placement-entry')), findsNothing);
    });
  });

  group('the run (D2)', () {
    testWidgets('a miss on unit 3 proposes unit 3; confirming skips 1 and 2 '
        'and records no completion', (tester) async {
      final c = _container();
      await _pump(tester, c, const Scaffold(body: LearnDoorBody()));
      await tester.tap(find.byKey(const Key('placement-offer-start')));
      await _settle(tester);
      expect(find.byType(LessonPlayerScreen), findsOneWidget);
      await tester.tap(find.byKey(const Key('lesson-next'))); // the hook
      await _settle(tester);
      await _answer(tester, [true, true]);
      // The first miss ends the run at once: no remaining probe, no recap,
      // the result instead of a celebration.
      await tester.tap(find.byKey(const Key('lesson-option-1')));
      await _settle(tester);
      expect(find.byType(PlacementResultScreen), findsOneWidget);
      expect(find.text('Unit 3'), findsOneWidget);
      expect(find.textContaining('2 exercises solved'), findsOneWidget);
      expect(find.textContaining('missed at “Unit three?”'), findsOneWidget);
      await tester.tap(find.byKey(const Key('placement-confirm')));
      await _settle(tester);

      expect(c.read(placementProvider).startUnit, 'u3');
      expect(c.read(courseCompletionProvider).completed, isEmpty);
      // Back on the door: units 1 and 2 collapsed but open, the offer gone.
      expect(find.byKey(const Key('path-unit-skipped-u1')), findsOneWidget);
      expect(find.byKey(const Key('path-unit-skipped-u2')), findsOneWidget);
      expect(find.byKey(const Key('path-unit-skipped-u3')), findsNothing);
      expect(find.byKey(const Key('path-node-a1')), findsNothing);
      expect(find.byKey(const Key('placement-offer')), findsNothing);
      await tester.tap(find.byKey(const Key('path-unit-open-u1')));
      await _settle(tester);
      expect(find.byKey(const Key('path-node-a1')), findsOneWidget);
    });

    testWidgets('everything solved proposes the unit after the last probed, '
        'clamped to the last one', (tester) async {
      final c = _container();
      await _pump(tester, c, const Scaffold(body: LearnDoorBody()));
      await tester.tap(find.byKey(const Key('placement-offer-start')));
      await _settle(tester);
      await tester.tap(find.byKey(const Key('lesson-next')));
      await _settle(tester);
      await _answer(tester, [true, true, true]);
      await tester.tap(find.byKey(const Key('lesson-next')));
      await _settle(tester);
      expect(find.text('Unit 3'), findsOneWidget);
      expect(find.textContaining('everything solved'), findsOneWidget);
    });

    testWidgets('"I don\'t know yet" is offered at once on a probe, and '
        'skipping ends the run on that unit', (tester) async {
      final c = _container();
      await _pump(tester, c, const Scaffold(body: LearnDoorBody()));
      await tester.tap(find.byKey(const Key('placement-offer-start')));
      await _settle(tester);
      // The hook is not a probe: no skip there.
      expect(find.byKey(const Key('lesson-skip')), findsNothing);
      await tester.tap(find.byKey(const Key('lesson-next')));
      await _settle(tester);
      await _answer(tester, [true]);
      // Unit 2's probe: the skip is there without any wait.
      expect(find.byKey(const Key('lesson-skip')), findsOneWidget);
      expect(find.text("I don't know yet"), findsOneWidget);
      await tester.tap(find.byKey(const Key('lesson-skip')));
      await _settle(tester);
      expect(find.byType(PlacementResultScreen), findsOneWidget);
      expect(find.text('Unit 2'), findsOneWidget);
      expect(find.textContaining('1 exercise solved'), findsOneWidget);
      expect(c.read(courseCompletionProvider).completed, isEmpty);
    });
  });

  group('the path starts at the starting unit (D3)', () {
    testWidgets('the continue card points to the starting unit\'s first '
        'lesson; clearing restores the first lesson', (tester) async {
      final c = _container(
        prefs: {
          'placement.v1': jsonEncode({
            'startUnit': 'u2',
            'takenAtMs': DateTime(2026, 10, 1).millisecondsSinceEpoch,
          }),
        },
      );
      await _pump(
        tester,
        c,
        const Scaffold(body: SingleChildScrollView(child: CoursesSection())),
      );
      expect(
        find.descendant(
          of: find.byKey(const Key('courses-continue-card')),
          matching: find.text('Lesson b1'),
        ),
        findsOneWidget,
      );
      await c.read(placementProvider.notifier).clear();
      await _settle(tester);
      expect(
        find.descendant(
          of: find.byKey(const Key('courses-continue-card')),
          matching: find.text('Lesson a1'),
        ),
        findsOneWidget,
      );
    });

    testWidgets('the path screen keeps the entry in its app bar', (
      tester,
    ) async {
      await _pump(tester, _container(), const LearningPathScreen());
      expect(find.byKey(const Key('placement-entry')), findsOneWidget);
    });
  });
}
