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
import 'package:music/services/audio_service.dart';
import 'package:music/services/course_catalog_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/src/rust/api/midi.dart' show MidiEvent, MidiEventKind;
import 'package:music/state/course_completion_notifier.dart';

import '../support/fakes.dart';
import '../support/prefs_fakes.dart';

class _FakeService implements CourseCatalogService {
  _FakeService(this.manifests, {this.listings = const []});
  final Map<String, String> manifests;
  final List<CourseListing> listings;
  @override
  Future<List<CourseListing>> listCourses() async => listings;
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
    FakePreferencesService prefs, {
    Map<String, String> manifests = const {'c1': _manifest},
    List<CourseListing> listings = const [],
  }) async {
    final midi = FakeMidiService();
    addTearDown(midi.close);
    final container = ProviderContainer(
      overrides: [
        courseCatalogServiceProvider.overrideWithValue(
          _FakeService(manifests, listings: listings),
        ),
        preferencesServiceProvider.overrideWithValue(prefs),
        audioServiceProvider.overrideWithValue(RecordingAudioService()),
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
    return container;
  }

  testWidgets(
    'steps through blocks, skipping the unsupported one, then finishes',
    (tester) async {
      final prefs = FakePreferencesService();
      final container = await pump(tester, prefs);

      // Step 1: the text block.
      expect(find.text('The staff has five lines.'), findsOneWidget);

      // Step 2: the question — it gates Next until an answer is given.
      await tester.tap(find.byKey(const Key('lesson-next')));
      await tester.pumpAndSettle();
      expect(find.text('Sharp does what?'), findsOneWidget);
      expect(find.byKey(const Key('lesson-next')), findsNothing);

      // Answering shows feedback and reopens Next (no auto-advance: the
      // feedback deserves to be read).
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

      // Finishing marks the course completed and celebrates before returning.
      await tester.tap(find.byKey(const Key('lesson-next')));
      await tester.pumpAndSettle();
      expect(find.text('Lesson complete!'), findsOneWidget);
      // The question counted as this run's one gated exercise — first try.
      expect(find.text('1 of 1 right on the first try'), findsOneWidget);
      expect(
        container.read(courseCompletionProvider).isCompleted('c1'),
        isTrue,
      );
      await tester.tap(find.byKey(const Key('lesson-celebration-close')));
      await tester.pumpAndSettle();
      expect(find.text('open'), findsOneWidget); // back on the launcher
    },
  );

  testWidgets('a gated exercise holds Next and only a late skip escapes', (
    tester,
  ) async {
    const gated = '''
{
  "schemaVersion": 2, "id": "c1", "title": {"en": "Gated"},
  "blocks": [
    {"type": "playKey", "notes": [60], "prompt": {"en": "Play middle C."}},
    {"type": "text", "text": {"en": "All done."}}
  ]
}
''';
    await pump(
      tester,
      FakePreferencesService(),
      manifests: const {'c1': gated},
    );

    // The gate holds: no Next, and no skip yet.
    expect(find.byKey(const Key('lesson-next')), findsNothing);
    expect(find.byKey(const Key('lesson-skip')), findsNothing);

    // The escape hatch appears only after the kindness delay.
    await tester.pump(const Duration(seconds: 13));
    expect(find.byKey(const Key('lesson-skip')), findsOneWidget);
    await tester.tap(find.byKey(const Key('lesson-skip')));
    await tester.pumpAndSettle();
    expect(find.text('All done.'), findsOneWidget);

    // Skipping earned no first-try credit: 0 of 1.
    await tester.tap(find.byKey(const Key('lesson-next')));
    await tester.pumpAndSettle();
    expect(find.text('0 of 1 right on the first try'), findsOneWidget);
    await tester.tap(find.byKey(const Key('lesson-celebration-close')));
    await tester.pumpAndSettle();
  });

  testWidgets(
    'completing the gate advances, earns the stat and chains to the next '
    'lesson',
    (tester) async {
      const gated = '''
{
  "schemaVersion": 2, "id": "c1", "title": {"en": "Gated"},
  "blocks": [
    {"type": "playKey", "notes": [60], "prompt": {"en": "Play middle C."}}
  ]
}
''';
      const next = '''
{"schemaVersion": 2, "id": "c2", "title": {"en": "Next lesson"},
 "blocks": [{"type": "text", "text": {"en": "Second lesson body."}}]}
''';
      final midi = FakeMidiService();
      addTearDown(midi.close);
      final container = ProviderContainer(
        overrides: [
          courseCatalogServiceProvider.overrideWithValue(
            _FakeService(
              const {'c1': gated, 'c2': next},
              listings: const [
                CourseListing(id: 'c1', schemaVersion: 2, sortOrder: 1),
                CourseListing(
                  id: 'c2',
                  schemaVersion: 2,
                  sortOrder: 2,
                  title: {'en': 'Next lesson'},
                ),
              ],
            ),
          ),
          preferencesServiceProvider.overrideWithValue(
            FakePreferencesService(),
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
            localizationsDelegates: AppLocalizations.localizationsDelegates,
            supportedLocales: AppLocalizations.supportedLocales,
            locale: const Locale('en'),
            home: const _Launcher(),
          ),
        ),
      );
      // Let the listing fetch resolve before opening the lesson.
      await tester.pumpAndSettle();
      await tester.tap(find.text('open'));
      await tester.pumpAndSettle();

      // Satisfy the gate through the MIDI seam (same path as the keyboard).
      midi.emit(
        MidiEvent(
          kind: MidiEventKind.noteOn,
          pitch: 60,
          velocity: 100,
          timestampMs: BigInt.zero,
        ),
      );
      await tester.pump(const Duration(milliseconds: 600));
      await tester.pumpAndSettle();

      // Last step satisfied → celebration with full first-try credit.
      expect(find.text('1 of 1 right on the first try'), findsOneWidget);

      // One tap chains straight into the next lesson.
      await tester.tap(find.byKey(const Key('lesson-celebration-next')));
      await tester.pumpAndSettle();
      expect(find.text('Second lesson body.'), findsOneWidget);
    },
  );

  testWidgets('a wrong answer still opens the gate — kindness over drilling', (
    tester,
  ) async {
    await pump(tester, FakePreferencesService());
    await tester.tap(find.byKey(const Key('lesson-next')));
    await tester.pumpAndSettle();
    expect(find.byKey(const Key('lesson-next')), findsNothing);
    await tester.tap(find.byKey(const Key('lesson-option-1'))); // wrong
    await tester.pumpAndSettle();
    expect(find.text('Not quite — keep going.'), findsOneWidget);
    // Answering — even wrongly — is enough to continue.
    expect(
      tester
          .widget<FilledButton>(find.byKey(const Key('lesson-next')))
          .onPressed,
      isNotNull,
    );
  });
}
