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

/// Screenshot harness for the interactive solfège courses: renders the REAL
/// corpus lessons (backend/content/courses) through the real widgets and
/// captures them as goldens. Tagged `golden` so the cross-platform gate skips
/// it; refresh with:
///   flutter test --tags golden test/screenshots/course_screens_test.dart \
///     --update-goldens
@Tags(['golden'])
library;

import 'dart:convert';
import 'dart:io';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/courses/course_manifest.dart';
import 'package:music/l10n/gen/app_localizations.dart';
import 'package:music/screens/learning_path_screen.dart';
import 'package:music/screens/lesson_player_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/course_catalog_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/theme/cymbra_theme.dart';
import 'package:music/widgets/build_chord_view.dart';
import 'package:music/widgets/courses_section.dart';
import 'package:music/widgets/ear_choice_view.dart';
import 'package:music/widgets/lesson_celebration.dart';
import 'package:music/widgets/name_note_view.dart';
import 'package:music/widgets/read_play_view.dart';
import 'package:music/widgets/rhythm_tap_view.dart';

import '../support/fakes.dart';
import '../support/prefs_fakes.dart';

const _corpusDir = '../../backend/content/courses';

Map<String, dynamic> _course(String id) =>
    jsonDecode(File('$_corpusDir/$id.json').readAsStringSync())
        as Map<String, dynamic>;

CourseManifest _manifest(String id) =>
    parseCourseManifest(jsonEncode(_course(id)['content']))!;

T _block<T extends CourseBlock>(String id, {int skip = 0}) =>
    _manifest(id).blocks.whereType<T>().skip(skip).first;

CourseListing _listing(String id) {
  final doc = _course(id);
  return CourseListing(
    id: doc['id'] as String,
    schemaVersion: doc['schemaVersion'] as int,
    track: doc['track'] as String,
    level: doc['level'] as String,
    unit: doc['unit'] as String,
    unitTitle: (doc['unitTitle'] as Map).cast<String, String>(),
    sortOrder: doc['sortOrder'] as int,
    title: (doc['title'] as Map).cast<String, String>(),
  );
}

class _FakeCatalog implements CourseCatalogService {
  _FakeCatalog(this.listings);
  final List<CourseListing> listings;
  @override
  Future<List<CourseListing>> listCourses() async => listings;
  @override
  Future<String?> getCourseManifestJson(String id) async =>
      jsonEncode(_course(id)['content']);
}

void main() {
  final ids =
      Directory(_corpusDir)
          .listSync()
          .whereType<File>()
          .where((f) => f.path.endsWith('.json'))
          .map((f) => f.uri.pathSegments.last.replaceAll('.json', ''))
          .toList()
        ..sort();

  Future<void> shoot(
    WidgetTester tester,
    Widget home,
    String name, {
    Set<String> completed = const {},
    Future<void> Function(WidgetTester)? act,
  }) async {
    tester.view.physicalSize = const Size(1080, 2100);
    tester.view.devicePixelRatio = 2.5;
    addTearDown(tester.view.reset);
    final midi = FakeMidiService();
    addTearDown(midi.close);
    final container = ProviderContainer(
      overrides: [
        courseCatalogServiceProvider.overrideWithValue(
          _FakeCatalog([for (final id in ids) _listing(id)]),
        ),
        preferencesServiceProvider.overrideWithValue(
          FakePreferencesService({
            'courses.completed.v1': jsonEncode(completed.toList()),
          }),
        ),
        audioServiceProvider.overrideWithValue(RecordingAudioService()),
        midiServiceProvider.overrideWithValue(midi),
      ],
    );
    addTearDown(container.dispose);
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: MaterialApp(
          theme: buildCymbraTheme(),
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          locale: const Locale('fr'),
          home: RepaintBoundary(child: home),
        ),
      ),
    );
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 350));
    if (act != null) await act(tester);
    await expectLater(
      find.byType(RepaintBoundary).first,
      matchesGoldenFile('shots/$name.png'),
    );
  }

  Widget exercise(Widget child) => Scaffold(
    backgroundColor: CymbraColors.background,
    body: SafeArea(
      child: Padding(padding: const EdgeInsets.all(20), child: child),
    ),
  );

  testWidgets('home continue card', (tester) async {
    await shoot(
      tester,
      const Scaffold(
        backgroundColor: CymbraColors.background,
        body: SafeArea(child: Column(children: [CoursesSection()])),
      ),
      'home-continue-card',
      completed: {'sol-u1-01-le-piano-et-le-do-central'},
    );
  });

  testWidgets('learning path', (tester) async {
    await shoot(
      tester,
      const LearningPathScreen(),
      'learning-path',
      completed: {
        'sol-u1-01-le-piano-et-le-do-central',
        'sol-u1-02-la-portee',
        'sol-u1-03-la-cle-de-sol',
      },
    );
  });

  testWidgets('lesson player on a staff block', (tester) async {
    await shoot(
      tester,
      const LessonPlayerScreen(courseId: 'sol-u1-04-do-re-mi'),
      'lesson-staff-block',
      act: (t) async {
        // Step past the hook to the staff illustration.
        await t.tap(find.byKey(const Key('lesson-next')));
        await t.pump(const Duration(milliseconds: 200));
      },
    );
  });

  testWidgets('readPlay drill', (tester) async {
    await shoot(
      tester,
      exercise(
        ReadPlayView(
          block: _block<ReadPlayBlock>('sol-u1-04-do-re-mi'),
          onCompleted: ({required bool flawless}) {},
        ),
      ),
      'exercise-readplay',
    );
  });

  testWidgets('nameNote chips', (tester) async {
    await shoot(
      tester,
      exercise(
        NameNoteView(
          block: _block<NameNoteBlock>('sol-u1-04-do-re-mi'),
          onCompleted: ({required bool flawless}) {},
        ),
      ),
      'exercise-namenote',
    );
  });

  testWidgets('earChoice listening', (tester) async {
    await shoot(
      tester,
      exercise(
        EarChoiceView(
          block: _block<EarChoiceBlock>('sol-u1-04-do-re-mi'),
          onCompleted: ({required bool flawless}) {},
        ),
      ),
      'exercise-earchoice',
    );
  });

  testWidgets('rhythmTap intro', (tester) async {
    await shoot(
      tester,
      exercise(
        RhythmTapView(
          block: _block<RhythmTapBlock>('sol-u2-02-noire-et-blanche'),
          onCompleted: ({required bool flawless}) {},
        ),
      ),
      'exercise-rhythmtap',
    );
  });

  testWidgets('buildChord with a started selection', (tester) async {
    await shoot(
      tester,
      exercise(
        BuildChordView(
          block: _block<BuildChordBlock>('sol-u7-03-la-triade-majeure'),
          onCompleted: ({required bool flawless}) {},
        ),
      ),
      'exercise-buildchord',
    );
  });

  testWidgets('celebration', (tester) async {
    await shoot(
      tester,
      Builder(
        builder: (context) => Scaffold(
          backgroundColor: CymbraColors.background,
          body: Center(
            child: FilledButton(
              key: const Key('go'),
              onPressed: () => showLessonCelebration(
                context,
                lessonTitle: 'Do, ré, mi',
                flawless: 4,
                gated: 4,
                nextLessonTitle: 'Fa et sol',
              ),
              child: const Text('go'),
            ),
          ),
        ),
      ),
      'lesson-celebration',
      act: (t) async {
        await t.tap(find.byKey(const Key('go')));
        await t.pump(const Duration(milliseconds: 400));
      },
    );
  });
}
