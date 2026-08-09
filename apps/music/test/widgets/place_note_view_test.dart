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
import 'package:music/services/midi_service.dart';
import 'package:music/theme/cymbra_theme.dart';
import 'package:music/widgets/lesson_staff.dart';
import 'package:music/widgets/place_note_view.dart';

import '../support/fakes.dart';

/// E4 sits on the treble bottom line (step 0), G4 one space+line above (step 2).
PlaceNoteBlock block({List<String> targets = const ['E4', 'G4']}) =>
    CourseBlock.placeNote(
          targets: [for (final t in targets) LessonPitch.parse(t)!],
          clef: LessonClef.treble,
          prompt: const {'fr': 'Place la note'},
        )
        as PlaceNoteBlock;

void main() {
  late RecordingAudioService audio;
  late List<bool> completions;

  Future<void> pump(WidgetTester tester, {PlaceNoteBlock? b}) async {
    audio = RecordingAudioService();
    completions = [];
    final midi = FakeMidiService();
    addTearDown(midi.close);
    await tester.pumpWidget(
      ProviderScope(
        overrides: [
          audioServiceProvider.overrideWithValue(audio),
          midiServiceProvider.overrideWithValue(midi),
        ],
        child: MaterialApp(
          localizationsDelegates: AppLocalizations.localizationsDelegates,
          supportedLocales: AppLocalizations.supportedLocales,
          locale: const Locale('fr'),
          home: Scaffold(
            body: Center(
              child: SizedBox(
                width: 360,
                child: PlaceNoteView(
                  block: b ?? block(),
                  onCompleted: ({required bool flawless}) =>
                      completions.add(flawless),
                ),
              ),
            ),
          ),
        ),
      ),
    );
  }

  LessonStaff staff(WidgetTester tester) =>
      tester.widget<LessonStaff>(find.byType(LessonStaff));

  /// Taps the staff exactly on [step] (half staff-spaces above the bottom
  /// line), using the same geometry the widget hit-tests with.
  Future<void> tapStep(WidgetTester tester, int step) async {
    final g = LessonStaff.geometryFor(staff(tester).height);
    final origin = tester.getTopLeft(find.byKey(const Key('lesson-staff-tap')));
    await tester.tapAt(origin + Offset(180, g.bottom - step * (g.s / 2)));
    await tester.pump();
  }

  List<int> noteOnPitches() => [for (final n in audio.noteOns) n.pitch];

  testWidgets('correct placements sound, accumulate and complete flawlessly', (
    tester,
  ) async {
    await pump(tester);

    // The chip names the first target (fr → solfège: E = Mi).
    expect(
      find.descendant(
        of: find.byKey(const Key('placenote-target')),
        matching: find.text('Mi'),
      ),
      findsOneWidget,
    );
    expect(find.byKey(const Key('placenote-dot-0')), findsOneWidget);
    expect(find.byKey(const Key('placenote-dot-1')), findsOneWidget);

    // Place E4 on the bottom line: it sounds and lands on the staff.
    await tapStep(tester, 0);
    expect(noteOnPitches(), contains(64));
    expect(staff(tester).elements.length, 1);

    // The queue advances (~350 ms) to G4 (Sol); the placed note stays.
    await tester.pump(const Duration(milliseconds: 400));
    expect(
      find.descendant(
        of: find.byKey(const Key('placenote-target')),
        matching: find.text('Sol'),
      ),
      findsOneWidget,
    );

    await tapStep(tester, 2);
    expect(noteOnPitches(), contains(67));
    expect(staff(tester).elements.length, 2);

    // Advance → completion convention: check mark, chime, then onCompleted.
    await tester.pump(const Duration(milliseconds: 400));
    expect(find.byIcon(Icons.check_circle), findsOneWidget);
    expect(completions, isEmpty);
    await tester.pump(const Duration(milliseconds: 500));
    expect(completions, [true]);

    // Exactly once, and a tap after completion is inert.
    final before = audio.noteOns.length;
    await tapStep(tester, 0);
    expect(audio.noteOns.length, before);
    await tester.pump(const Duration(seconds: 1));
    expect(completions, [true]);
  });

  testWidgets(
    'a wrong step is heard, ghosts in the error tint and does not advance',
    (tester) async {
      await pump(tester);

      // Tap B4 (step 4) while E4 is awaited: the wrong pitch sounds…
      await tapStep(tester, 4);
      expect(noteOnPitches(), [71]);
      // …nothing is placed, the chip still asks for Mi…
      expect(staff(tester).elements, isEmpty);
      expect(
        find.descendant(
          of: find.byKey(const Key('placenote-target')),
          matching: find.text('Mi'),
        ),
        findsOneWidget,
      );
      // …and the tapped natural ghosts in the error colour, then fades.
      expect(staff(tester).ghost, LessonPitch.parse('B4'));
      expect(
        staff(tester).ghostColor,
        CymbraColors.error.withValues(alpha: 0.7),
      );
      await tester.pump(const Duration(milliseconds: 650));
      expect(staff(tester).ghost, isNull);

      // Completing after the miss reports a non-flawless run.
      await tapStep(tester, 0);
      await tester.pump(const Duration(milliseconds: 400));
      await tapStep(tester, 2);
      await tester.pump(const Duration(milliseconds: 900));
      expect(completions, [false]);
    },
  );

  testWidgets('the listen button sounds the current target', (tester) async {
    await pump(tester);

    expect(audio.noteOns, isEmpty);
    await tester.tap(find.byKey(const Key('placenote-listen')));
    expect(noteOnPitches(), [64]);
    await tester.pump(const Duration(milliseconds: 350));
    expect(completions, isEmpty);
  });
}
