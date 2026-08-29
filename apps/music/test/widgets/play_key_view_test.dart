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
import 'package:music/services/midi_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/src/rust/api/midi.dart' show MidiEvent, MidiEventKind;
import 'package:music/widgets/play_key_view.dart';

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

// A playKey step asking for middle C, then a text step so satisfying advances.
const _manifest = '''
{
  "schemaVersion": 1, "id": "pk", "title": {"en": "PK"},
  "blocks": [
    {"type": "playKey", "notes": [60], "prompt": {"en": "Play middle C"}},
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
        onPressed: () => openLessonPlayer(context, 'pk'),
        child: const Text('open'),
      ),
    ),
  );
}

MidiEvent _noteOn(int pitch) => MidiEvent(
  kind: MidiEventKind.noteOn,
  pitch: pitch,
  velocity: 100,
  channel: 0,
  timestampMs: BigInt.zero,
);

void main() {
  Future<FakeMidiService> pump(WidgetTester tester) async {
    final midi = FakeMidiService();
    addTearDown(midi.close);
    final container = ProviderContainer(
      overrides: [
        courseCatalogServiceProvider.overrideWithValue(_FakeCourses(_manifest)),
        preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
        midiServiceProvider.overrideWithValue(midi),
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
    return midi;
  }

  testWidgets('the on-screen keyboard is shown for a playKey step', (
    tester,
  ) async {
    await pump(tester);
    expect(find.byType(PlayKeyView), findsOneWidget);
    expect(find.text('Play middle C'), findsOneWidget);
    // Not yet advanced to the next step.
    expect(find.text('done'), findsNothing);
  });

  testWidgets('playing the target note (via MIDI) advances the lesson', (
    tester,
  ) async {
    final midi = await pump(tester);
    midi.emit(_noteOn(60)); // middle C — the target
    // The gate shows a brief confirmation, then auto-advances (~450 ms).
    await tester.pump(const Duration(milliseconds: 500));
    await tester.pumpAndSettle();
    // The gate was met, so the lesson moved on to the text step.
    expect(find.text('done'), findsOneWidget);
    expect(find.byType(PlayKeyView), findsNothing);
  });

  testWidgets('a wrong note does not advance', (tester) async {
    final midi = await pump(tester);
    midi.emit(_noteOn(62)); // D — not the target
    await tester.pumpAndSettle();
    expect(find.text('done'), findsNothing);
    expect(find.byType(PlayKeyView), findsOneWidget);
  });

  testWidgets('a stuck learner still escapes through the late skip', (
    tester,
  ) async {
    await pump(tester);
    // The gate holds Next back…
    expect(find.byKey(const Key('lesson-next')), findsNothing);
    // …but the discreet escape hatch appears after the kindness delay, so
    // nobody is ever trapped on an exercise.
    await tester.pump(const Duration(seconds: 13));
    await tester.tap(find.byKey(const Key('lesson-skip')));
    await tester.pumpAndSettle();
    expect(find.text('done'), findsOneWidget);
  });
}
