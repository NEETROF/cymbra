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
import 'package:music/screens/library_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/course_catalog_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/state/drums_access.dart';
import 'package:music/state/home_door.dart';
import 'package:music/state/instrument_context.dart';
import 'package:music/state/learn_door_access.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';
import '../support/prefs_fakes.dart';

/// Three lessons in two units — enough for a continue card, a path and a
/// "lesson in progress" chip.
class _FakeCourses implements CourseCatalogService {
  @override
  Future<List<CourseListing>> listCourses() async => const [
    CourseListing(
      id: 'c1',
      schemaVersion: 2,
      unit: 'u1',
      unitTitle: {'en': 'First notes'},
      sortOrder: 101,
      title: {'en': 'The staff'},
    ),
    CourseListing(
      id: 'c2',
      schemaVersion: 2,
      unit: 'u1',
      unitTitle: {'en': 'First notes'},
      sortOrder: 102,
      title: {'en': 'The treble clef'},
    ),
    CourseListing(
      id: 'c3',
      schemaVersion: 2,
      unit: 'u2',
      unitTitle: {'en': 'Rhythm'},
      sortOrder: 201,
      title: {'en': 'The pulse'},
    ),
  ];

  @override
  Future<String?> getCourseManifestJson(String id) async =>
      '{"schemaVersion":1,"id":"$id","title":{"en":"The staff"},"blocks":['
      '{"type":"text","text":{"en":"hello"}}]}';
}

/// No courses at all: the Learn door's quiet empty state.
class _NoCourses implements CourseCatalogService {
  @override
  Future<List<CourseListing>> listCourses() async => const [];
  @override
  Future<String?> getCourseManifestJson(String id) async => null;
}

/// Flippable visibility, like the instrument-context tests: models the
/// `drums.enabled` snapshot resolving live.
final _drumsVisible = StateProvider((ref) => false);

ProviderContainer _container({
  FakePreferencesService? prefs,
  bool learnDoor = true,
  bool drumsVisible = false,
  CourseCatalogService? courses,
}) {
  final c = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(
        prefs ?? FakePreferencesService(),
      ),
      learnDoorEnabledProvider.overrideWithValue(learnDoor),
      drumsEnabledProvider.overrideWith((ref) => ref.watch(_drumsVisible)),
      courseCatalogServiceProvider.overrideWithValue(courses ?? _FakeCourses()),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(FakeNotationEngine()),
      midiServiceProvider.overrideWithValue(FakeMidiService()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
    ],
  );
  c.read(_drumsVisible.notifier).state = drumsVisible;
  addTearDown(c.dispose);
  return c;
}

Future<void> _pump(
  WidgetTester tester,
  ProviderContainer container, {
  Size size = const Size(1400, 900),
  bool reduceMotion = false,
}) async {
  // The view size is what MediaQuery (and so the device class) reads.
  tester.view.physicalSize = size;
  tester.view.devicePixelRatio = 1.0;
  addTearDown(tester.view.reset);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(
        reduceMotion
            ? Builder(
                builder: (context) => MediaQuery(
                  data: MediaQuery.of(
                    context,
                  ).copyWith(disableAnimations: true),
                  child: const LibraryScreen(),
                ),
              )
            : const LibraryScreen(),
        locale: const Locale('en'),
      ),
    ),
  );
  await _frames(tester);
}

/// Bounded frames: the path's next node pulses forever, so pumpAndSettle
/// never settles.
Future<void> _frames(WidgetTester tester, [int n = 10]) async {
  for (var i = 0; i < n; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

Future<void> _teardown(WidgetTester tester) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
}

final _switcher = find.byKey(const Key('home-door-switcher'));
final _learnBody = find.byKey(const Key('learn-door-body'));
final _playBody = find.byKey(const Key('play-door-body'));
final _continueCard = find.byKey(const Key('courses-continue-card'));
final _chip = find.byKey(const Key('home-continue-learning-chip'));

String _drumsStored() =>
    jsonEncode({'context': 'drums', 'choiceOffered': true});

void main() {
  testWidgets('opens on the Play door: the library as before, no courses '
      'card, the switcher in the header', (tester) async {
    final c = _container();
    await _pump(tester, c);

    expect(_switcher, findsOneWidget);
    expect(_playBody, findsOneWidget);
    expect(_learnBody, findsNothing);
    expect(find.text('Ode to Joy (theme)'), findsOneWidget);
    expect(find.byKey(const Key('courses-section')), findsNothing);
    expect(_continueCard, findsNothing);
    // Nothing started yet → no way-back chip either.
    expect(_chip, findsNothing);
    await _teardown(tester);
  });

  testWidgets('Learn shows the continue card and the path, and the choice '
      'is persisted', (tester) async {
    final prefs = FakePreferencesService();
    final c = _container(prefs: prefs);
    await _pump(tester, c);

    await tester.tap(find.text('Learn'));
    await _frames(tester);

    expect(_learnBody, findsOneWidget);
    expect(_playBody, findsNothing);
    expect(find.text('Ode to Joy (theme)'), findsNothing);
    // Zero-extent until the daily session lands, so the viewport reports it
    // offstage — present in the tree all the same.
    expect(
      find.byKey(const Key('daily-session-slot'), skipOffstage: false),
      findsOneWidget,
    );
    expect(_continueCard, findsOneWidget);
    expect(find.byKey(const Key('path-unit-u1')), findsOneWidget);
    expect(find.byKey(const Key('path-node-c3')), findsOneWidget);
    // The door's own card drops the "see the path" link: the path is below.
    expect(find.byKey(const Key('courses-see-path')), findsNothing);
    expect(prefs.store[HomeDoorChoice.prefsKey], 'learn');
    await _teardown(tester);
  });

  testWidgets('a relaunch reopens on the remembered door', (tester) async {
    final c = _container(
      prefs: FakePreferencesService({HomeDoorChoice.prefsKey: 'learn'}),
    );
    await _pump(tester, c);
    expect(_learnBody, findsOneWidget);
    expect(_playBody, findsNothing);
    await _teardown(tester);
  });

  testWidgets('a guest\'s lesson opens from the Learn door and its title '
      'flies into the lesson player', (tester) async {
    final c = _container(
      prefs: FakePreferencesService({HomeDoorChoice.prefsKey: 'learn'}),
    );
    await _pump(tester, c);
    await tester.tap(_continueCard);
    // Let the route transition (and the Hero flight) run to completion.
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));
    await _frames(tester);
    expect(find.byType(LessonPlayerScreen), findsOneWidget);
    expect(find.byType(Hero), findsWidgets);
    expect(find.text('The staff'), findsOneWidget);
    await _teardown(tester);
  });

  testWidgets('a lesson in progress puts a way-back chip on the Play door; '
      'tapping it switches the door', (tester) async {
    final c = _container(
      prefs: FakePreferencesService({'courses.completed.v1': '["c1"]'}),
    );
    await _pump(tester, c);
    expect(_playBody, findsOneWidget);
    expect(_chip, findsOneWidget);
    expect(find.text('Continue learning · The treble clef'), findsOneWidget);

    await tester.tap(_chip);
    await _frames(tester);
    expect(_learnBody, findsOneWidget);
    expect(c.read(homeDoorChoiceProvider).door, HomeDoor.learn);
    await _teardown(tester);
  });

  testWidgets('switching doors animates: both bodies coexist mid-transition', (
    tester,
  ) async {
    final c = _container();
    await _pump(tester, c);
    await tester.tap(find.text('Learn'));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 100));
    expect(_playBody, findsOneWidget);
    expect(_learnBody, findsOneWidget);
    await _frames(tester);
    expect(_playBody, findsNothing);
    await _teardown(tester);
  });

  testWidgets('reduced motion swaps the body at once', (tester) async {
    final c = _container();
    await _pump(tester, c, reduceMotion: true);
    await tester.tap(find.text('Learn'));
    await tester.pump();
    expect(_learnBody, findsOneWidget);
    expect(_playBody, findsNothing);
    await _teardown(tester);
  });

  testWidgets('the drums context offers Play only and hides the learning '
      'content; the keyboard context brings Learn back', (tester) async {
    final c = _container(
      drumsVisible: true,
      prefs: FakePreferencesService({
        HomeDoorChoice.prefsKey: 'learn',
        InstrumentContext.prefsKey: _drumsStored(),
      }),
    );
    await _pump(tester, c);
    expect(_switcher, findsOneWidget);
    expect(find.text('Learn'), findsNothing);
    expect(_learnBody, findsNothing);
    expect(_continueCard, findsNothing);

    await tester.tap(
      find.descendant(
        of: find.byKey(const Key('instrument-switcher')),
        matching: find.text('Piano'),
      ),
    );
    await _frames(tester);
    expect(find.text('Learn'), findsOneWidget);
    expect(_learnBody, findsOneWidget);
    await _teardown(tester);
  });

  testWidgets('with the flag off the home is the single body it always was', (
    tester,
  ) async {
    final c = _container(
      learnDoor: false,
      prefs: FakePreferencesService({HomeDoorChoice.prefsKey: 'learn'}),
    );
    await _pump(tester, c);
    expect(_switcher, findsNothing);
    expect(_learnBody, findsNothing);
    expect(find.text('Ode to Joy (theme)'), findsOneWidget);
    expect(_chip, findsNothing);
    await _teardown(tester);
  });

  testWidgets('a Learn door with no courses states it quietly', (tester) async {
    final c = _container(
      courses: _NoCourses(),
      prefs: FakePreferencesService({HomeDoorChoice.prefsKey: 'learn'}),
    );
    await _pump(tester, c);
    expect(find.byKey(const Key('learn-door-empty')), findsOneWidget);
    expect(_continueCard, findsNothing);
    await _teardown(tester);
  });

  testWidgets('the pushed path screen still works as a deep link', (
    tester,
  ) async {
    final c = _container(learnDoor: false);
    await _pump(tester, c);
    // Not reachable from the single guest home; push it directly.
    final context = tester.element(find.byType(LibraryScreen));
    openLearningPath(context);
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));
    expect(find.byKey(const Key('path-screen')), findsOneWidget);
    expect(find.byKey(const Key('path-node-c1')), findsOneWidget);
    await _teardown(tester);
  });
}
