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
import 'package:music/state/course_completion_notifier.dart';

import '../support/prefs_fakes.dart';

class _FakeService implements CourseCatalogService {
  _FakeService(this.manifests);
  final Map<String, String> manifests;
  @override
  Future<List<CourseListing>> listCourses() async => const [];
  @override
  Future<String?> getCourseManifestJson(String id) async => manifests[id];
}

/// A text step, a 2-option question (answer 0), an UNSUPPORTED block (must be
/// skipped), and a diagram — so the player shows 3 steps, not 4.
const _manifest = '''
{
  "schemaVersion": 1, "id": "c1",
  "title": {"en": "Reading the staff"},
  "blocks": [
    {"type": "text", "text": {"en": "The staff has five lines."}},
    {"type": "question", "prompt": {"en": "Sharp does what?"},
     "options": [{"en": "Raises a semitone"}, {"en": "Lowers it"}],
     "answerIndex": 0, "feedback": {"en": "Up a semitone."}},
    {"type": "hologram", "x": 1},
    {"type": "diagram", "id": "treble-clef"}
  ]
}
''';

class _Launcher extends StatelessWidget {
  const _Launcher();
  @override
  Widget build(BuildContext context) => Scaffold(
    body: Center(
      child: ElevatedButton(
        onPressed: () => openLessonPlayer(context, 'c1'),
        child: const Text('open'),
      ),
    ),
  );
}

void main() {
  Future<ProviderContainer> pump(
    WidgetTester tester,
    FakePreferencesService prefs,
  ) async {
    final container = ProviderContainer(
      overrides: [
        courseCatalogServiceProvider.overrideWithValue(
          _FakeService(const {'c1': _manifest}),
        ),
        preferencesServiceProvider.overrideWithValue(prefs),
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
          home: const _Launcher(),
        ),
      ),
    );
    await tester.tap(find.text('open'));
    await tester.pumpAndSettle();
    return container;
  }

  testWidgets(
    'steps through blocks, skipping the unsupported one, then finishes',
    (tester) async {
      final prefs = FakePreferencesService();
      final container = await pump(tester, prefs);

      // Step 1: the text block.
      expect(find.text('The staff has five lines.'), findsOneWidget);

      // Step 2: the question.
      await tester.tap(find.byKey(const Key('lesson-next')));
      await tester.pumpAndSettle();
      expect(find.text('Sharp does what?'), findsOneWidget);

      // Answering shows feedback but never blocks continuing.
      await tester.tap(find.byKey(const Key('lesson-option-0')));
      await tester.pumpAndSettle();
      expect(find.text('Correct!'), findsOneWidget);
      expect(
        tester
            .widget<FilledButton>(find.byKey(const Key('lesson-next')))
            .onPressed,
        isNotNull,
      );

      // Step 3: the diagram (last) — the 'hologram' block was skipped, so Next
      // now finishes.
      await tester.tap(find.byKey(const Key('lesson-next')));
      await tester.pumpAndSettle();
      expect(find.byType(CustomPaint), findsWidgets);
      expect(find.text('Finish'), findsOneWidget);

      // Finishing marks the course completed and returns.
      await tester.tap(find.byKey(const Key('lesson-next')));
      await tester.pumpAndSettle();
      expect(find.text('open'), findsOneWidget); // back on the launcher
      expect(
        container.read(courseCompletionProvider).isCompleted('c1'),
        isTrue,
      );
    },
  );

  testWidgets('a wrong answer gives non-blocking feedback', (tester) async {
    await pump(tester, FakePreferencesService());
    await tester.tap(find.byKey(const Key('lesson-next')));
    await tester.pumpAndSettle();
    await tester.tap(find.byKey(const Key('lesson-option-1'))); // wrong
    await tester.pumpAndSettle();
    expect(find.text('Not quite — keep going.'), findsOneWidget);
    // Still advanceable.
    expect(
      tester
          .widget<FilledButton>(find.byKey(const Key('lesson-next')))
          .onPressed,
      isNotNull,
    );
  });
}
