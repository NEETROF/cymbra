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
import 'package:music/services/notation_engine.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/widgets/score_block_view.dart';

import '../support/notation_fakes.dart';
import '../support/fakes.dart';
import '../support/prefs_fakes.dart';

class _FakeCourses implements CourseCatalogService {
  _FakeCourses(this.manifest);
  final String manifest;
  @override
  Future<List<CourseListing>> listCourses() async => const [];
  @override
  Future<String?> getCourseManifestJson(String id) async => manifest;
}

const _manifest = '''
{
  "schemaVersion": 1, "id": "sc", "title": {"en": "SC"},
  "blocks": [
    {"type": "score", "musicXml": "<score/>", "playable": false,
     "prompt": {"en": "Look at this excerpt"}},
    {"type": "text", "text": {"en": "done"}}
  ]
}
''';

class _Launcher extends StatelessWidget {
  const _Launcher();
  @override
  Widget build(BuildContext context) => Scaffold(
    body: Center(
      child: ElevatedButton(
        onPressed: () => openLessonPlayer(context, 'sc'),
        child: const Text('open'),
      ),
    ),
  );
}

void main() {
  Future<void> pump(WidgetTester tester) async {
    final midi = FakeMidiService();
    addTearDown(midi.close);
    final container = ProviderContainer(
      overrides: [
        midiServiceProvider.overrideWithValue(midi),
        courseCatalogServiceProvider.overrideWithValue(_FakeCourses(_manifest)),
        preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
        // Parse is faked → a hand-built document, no native library needed.
        notationEngineProvider.overrideWithValue(FakeNotationEngine()),
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
  }

  testWidgets('a score block engraves the excerpt from its MusicXML', (
    tester,
  ) async {
    await pump(tester);
    expect(find.byType(ScoreBlockView), findsOneWidget);
    expect(find.text('Look at this excerpt'), findsOneWidget);
    // The excerpt is engraved (a CustomPaint from PartitionPainter), not a
    // spinner or the failure icon.
    expect(find.byType(CustomPaint), findsWidgets);
    expect(find.byType(CircularProgressIndicator), findsNothing);
  });

  testWidgets('the score block is skippable', (tester) async {
    await pump(tester);
    await tester.tap(find.byKey(const Key('lesson-next')));
    await tester.pumpAndSettle();
    expect(find.text('done'), findsOneWidget);
  });
}
