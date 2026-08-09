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
import 'package:music/widgets/ear_choice_view.dart';
import 'package:music/widgets/lesson_staff.dart';

import '../support/fakes.dart';

// C4 (60) then E4 (64), default gap 700 ms — a major third, asked as a choice.
EarChoiceBlock _block({bool reveal = true}) => EarChoiceBlock(
  notes: [LessonPitch.parse('C4')!, LessonPitch.parse('E4')!],
  choices: const [
    EarOption('third', {'fr': 'Tierce'}),
    EarOption('fifth', {'fr': 'Quinte'}),
  ],
  answerId: 'third',
  reveal: reveal,
  prompt: const {'fr': 'Écoute et choisis'},
);

void main() {
  Future<({RecordingAudioService audio, List<bool> completions})> pumpView(
    WidgetTester tester,
    EarChoiceBlock block,
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
            body: EarChoiceView(
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

  /// Flushes one full playback of the two-note sequence (gap 700 + note 550).
  Future<void> flushSequence(WidgetTester tester) =>
      tester.pump(const Duration(milliseconds: 1400));

  testWidgets('auto-plays the sequence shortly after mount', (tester) async {
    final h = await pumpView(tester, _block());
    expect(find.text('Écoute et choisis'), findsOneWidget);
    expect(find.text('Écouter'), findsOneWidget); // the Listen button label
    expect(find.text('Tierce'), findsOneWidget);
    expect(find.text('Quinte'), findsOneWidget);

    // Nothing sounds before the ~400 ms auto-play delay…
    expect(h.audio.noteOns, isEmpty);
    // …then both notes of the sequence, in order.
    await tester.pump(const Duration(milliseconds: 450));
    expect(h.audio.noteOns.map((e) => e.pitch), [60]);
    await tester.pump(const Duration(milliseconds: 700));
    expect(h.audio.noteOns.map((e) => e.pitch), [60, 64]);
    await flushSequence(tester);
  });

  testWidgets('the Listen button replays, free and unlimited', (tester) async {
    final h = await pumpView(tester, _block());
    await tester.pump(const Duration(milliseconds: 450));
    await flushSequence(tester);
    expect(h.audio.noteOns, hasLength(2)); // the auto-play

    await tester.tap(find.byKey(const Key('earchoice-listen')));
    await flushSequence(tester);
    expect(h.audio.noteOns, hasLength(4));

    await tester.tap(find.byKey(const Key('earchoice-listen')));
    await flushSequence(tester);
    expect(h.audio.noteOns, hasLength(6));
    expect(h.completions, isEmpty); // listening is never an answer
  });

  testWidgets('the correct chip completes flawless and reveals the staff', (
    tester,
  ) async {
    final h = await pumpView(tester, _block());
    await tester.pump(const Duration(milliseconds: 450));
    await flushSequence(tester);
    expect(find.byType(LessonStaff), findsNothing);

    await tester.tap(find.byKey(const Key('earchoice-chip-third')));
    await tester.pump();
    // Completion beat: check icon + the notes the ear just heard, on a staff.
    expect(find.byIcon(Icons.check_circle), findsOneWidget);
    expect(find.byType(LessonStaff), findsOneWidget);
    // The reveal is held a full 5 s so the learner can actually read it.
    await tester.pump(const Duration(seconds: 4));
    expect(h.completions, isEmpty);
    expect(find.byType(LessonStaff), findsOneWidget);
    await tester.pump(const Duration(milliseconds: 1100));
    expect(h.completions, [true]);
  });

  testWidgets('a wrong chip auto-replays the sequence and forfeits flawless', (
    tester,
  ) async {
    final h = await pumpView(tester, _block());
    await tester.pump(const Duration(milliseconds: 450));
    await flushSequence(tester);
    expect(h.audio.noteOns, hasLength(2));

    await tester.tap(find.byKey(const Key('earchoice-chip-fifth')));
    await tester.pump(const Duration(milliseconds: 10));
    // Error fill on the wrong chip — never a success colour, no completion.
    final wrong = tester.widget<OutlinedButton>(
      find.byKey(const Key('earchoice-chip-fifth')),
    );
    expect(
      wrong.style?.backgroundColor?.resolve(const <WidgetState>{}),
      CymbraColors.error.withValues(alpha: 0.12),
    );
    expect(find.byIcon(Icons.check_circle), findsNothing);
    expect(h.completions, isEmpty);

    // The sequence replays once so the learner can listen again.
    await tester.pump(const Duration(milliseconds: 500));
    await flushSequence(tester);
    expect(h.audio.noteOns, hasLength(4));

    await tester.tap(find.byKey(const Key('earchoice-chip-third')));
    await tester.pump(const Duration(seconds: 6)); // reveal hold, then done
    expect(h.completions, [false]);
  });

  testWidgets('no staff reveal when the block says reveal: false', (
    tester,
  ) async {
    final h = await pumpView(tester, _block(reveal: false));
    await tester.pump(const Duration(milliseconds: 450));
    await flushSequence(tester);

    await tester.tap(find.byKey(const Key('earchoice-chip-third')));
    await tester.pump(const Duration(milliseconds: 100));
    expect(find.byIcon(Icons.check_circle), findsOneWidget);
    expect(find.byType(LessonStaff), findsNothing);
    await tester.pump(const Duration(milliseconds: 500));
    expect(h.completions, [true]);
    expect(find.byType(LessonStaff), findsNothing);
  });
}
