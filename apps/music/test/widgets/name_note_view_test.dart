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
import 'package:music/courses/lesson_pitch.dart';
import 'package:music/l10n/gen/app_localizations.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/theme/cymbra_theme.dart';
import 'package:music/widgets/lesson_staff.dart';
import 'package:music/widgets/name_note_view.dart';

import '../support/fakes.dart';

NameNoteBlock _block({List<String> notes = const ['C4']}) => NameNoteBlock(
  notes: [for (final n in notes) LessonPitch.parse(n)!],
  prompt: const {'fr': 'Nomme la note'},
);

void main() {
  // Deterministic chip layout under the French locale (solfège names):
  // C4 target → [Do, Si, Ré] (rotation 0); D4 at queue index 1 → [Do, Mi, Ré]
  // (rotation 1, correct chip last).
  Future<({RecordingAudioService audio, List<bool> completions})> pumpView(
    WidgetTester tester,
    NameNoteBlock block,
  ) async {
    final audio = RecordingAudioService();
    final completions = <bool>[];
    final container = ProviderContainer(
      overrides: [audioServiceProvider.overrideWithValue(audio)],
    );
    addTearDown(container.dispose);
    await tester.pumpWidget(
      UncontrolledProviderScope(
        container: container,
        child: MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          locale: const Locale('fr'),
          home: Scaffold(
            body: NameNoteView(
              block: block,
              onCompleted: ({required bool flawless}) =>
                  completions.add(flawless),
            ),
          ),
        ),
      ),
    );
    return (audio: audio, completions: completions);
  }

  testWidgets(
    'correct names advance the queue and complete flawless, with real '
    'French distractor names',
    (tester) async {
      final h = await pumpView(tester, _block(notes: const ['C4', 'D4']));

      // One note on the staff, localized chips, one progress dot per note.
      expect(find.byType(LessonStaff), findsOneWidget);
      expect(find.text('Nomme la note'), findsOneWidget);
      expect(find.text('Do'), findsOneWidget);
      expect(find.text('Si'), findsOneWidget);
      expect(find.text('Ré'), findsOneWidget);
      expect(find.byKey(const Key('namenote-dot-0')), findsOneWidget);
      expect(find.byKey(const Key('namenote-dot-1')), findsOneWidget);

      // Name C4 correctly → the queue advances to D4 (new distractor set).
      await tester.tap(find.text('Do'));
      await tester.pump(const Duration(milliseconds: 600));
      expect(find.text('Mi'), findsOneWidget);
      expect(h.completions, isEmpty);

      // Name D4 correctly → completion beat (check icon), then the callback.
      await tester.tap(find.text('Ré'));
      await tester.pump(const Duration(milliseconds: 600));
      expect(find.byIcon(Icons.check_circle), findsOneWidget);
      expect(h.completions, isEmpty); // the 450 ms beat is still running
      await tester.pump(const Duration(milliseconds: 500));
      expect(h.completions, [true]);
    },
  );

  testWidgets('every chip sounds its pitch on tap — browsing is ear training', (
    tester,
  ) async {
    final h = await pumpView(tester, _block());
    expect(h.audio.noteOns, isEmpty);

    // Wrong chips still sound: Si = B3 (59), Ré = D4 (62).
    await tester.tap(find.text('Si'));
    expect(h.audio.noteOns.map((e) => e.pitch), contains(59));
    await tester.pump(const Duration(milliseconds: 400));
    await tester.tap(find.text('Ré'));
    expect(h.audio.noteOns.map((e) => e.pitch), contains(62));
    await tester.pump(const Duration(milliseconds: 400));
    expect(h.audio.noteOns, hasLength(2));

    // The correct chip sounds too, and the earlier misses forfeit flawless.
    await tester.tap(find.text('Do'));
    expect(h.audio.noteOns.map((e) => e.pitch), contains(60));
    await tester.pump(const Duration(milliseconds: 600));
    await tester.pump(const Duration(milliseconds: 500));
    expect(h.completions, [false]);
  });

  testWidgets('a wrong chip flashes the error fill and never completes', (
    tester,
  ) async {
    final h = await pumpView(tester, _block());

    await tester.tap(find.text('Ré')); // wrong: chip index 2
    await tester.pump(const Duration(milliseconds: 10));
    final wrong = tester.widget<OutlinedButton>(
      find.byKey(const Key('namenote-chip-2')),
    );
    expect(
      wrong.style?.backgroundColor?.resolve(const <WidgetState>{}),
      CymbraColors.error.withValues(alpha: 0.12),
    );
    // No success signal anywhere on a wrong answer.
    expect(find.byIcon(Icons.check_circle), findsNothing);
    expect(h.completions, isEmpty);

    // The flash is transient.
    await tester.pump(const Duration(milliseconds: 400));
    final after = tester.widget<OutlinedButton>(
      find.byKey(const Key('namenote-chip-2')),
    );
    expect(after.style?.backgroundColor?.resolve(const <WidgetState>{}), null);
    expect(h.completions, isEmpty);
  });

  testWidgets('onCompleted fires exactly once', (tester) async {
    final h = await pumpView(tester, _block());

    await tester.tap(find.text('Do'));
    await tester.pump(const Duration(milliseconds: 600));
    await tester.pump(const Duration(milliseconds: 500));
    expect(h.completions, [true]);

    // Tapping again after completion only sounds — it never re-fires.
    await tester.tap(find.text('Do'));
    await tester.pump(const Duration(milliseconds: 600));
    await tester.pump(const Duration(milliseconds: 500));
    expect(h.completions, [true]);
  });
}
