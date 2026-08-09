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
import 'package:music/painters/piano_keyboard_painter.dart';
import 'package:music/painters/piano_layout.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/widgets/build_chord_view.dart';

import '../support/fakes.dart';

/// C major (C4-E4-G4 = 60-64-67). The keyboard range is the chord ±2 snapped
/// to white keys: A3 (57) … A4 (69).
BuildChordBlock block() =>
    CourseBlock.buildChord(
          notes: [
            for (final t in ['C4', 'E4', 'G4']) LessonPitch.parse(t)!,
          ],
          prompt: const {'fr': 'Construis do majeur'},
        )
        as BuildChordBlock;

void main() {
  late RecordingAudioService audio;
  late FakeMidiService midi;
  late List<bool> completions;

  Future<void> pump(WidgetTester tester) async {
    audio = RecordingAudioService();
    midi = FakeMidiService();
    addTearDown(midi.close);
    completions = [];
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
                width: 400,
                child: BuildChordView(
                  block: block(),
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

  PianoKeyboardPainter painter(WidgetTester tester) =>
      tester
              .widget<CustomPaint>(find.byKey(const Key('buildchord-keyboard')))
              .painter!
          as PianoKeyboardPainter;

  /// Taps [pitch] on the on-screen keyboard, rebuilding the widget's own
  /// layout (range A3..A4) to find the key's centre; y=120 is the white-only
  /// band of the 132 px keyboard.
  Future<void> tapKey(WidgetTester tester, int pitch) async {
    final rect = tester.getRect(find.byKey(const Key('buildchord-keyboard')));
    final layout = PianoLayout(lowPitch: 57, highPitch: 69, width: rect.width);
    await tester.tapAt(rect.topLeft + Offset(layout.centerX(pitch), 120));
    await tester.pump();
  }

  List<int> noteOnPitches() => [for (final n in audio.noteOns) n.pitch];

  testWidgets('tapping a key sounds it and toggles the selection', (
    tester,
  ) async {
    await pump(tester);

    // The prompt and the chord's note names (fr solfège) are shown.
    expect(find.text('Construis do majeur'), findsOneWidget);
    expect(find.text('Do – Mi – Sol'), findsOneWidget);

    await tapKey(tester, 60);
    expect(noteOnPitches(), [60]);
    expect(painter(tester).selectedNotes, {60});

    // Tapping again toggles it off — and still sounds.
    await tapKey(tester, 60);
    expect(noteOnPitches(), [60, 60]);
    expect(painter(tester).selectedNotes, isEmpty);

    await tester.pump(const Duration(milliseconds: 350));
    expect(completions, isEmpty);
  });

  testWidgets('building the exact chord strums it and completes flawlessly', (
    tester,
  ) async {
    await pump(tester);

    await tapKey(tester, 60);
    await tapKey(tester, 64);
    await tapKey(tester, 67);

    // Validated: the keys show the painter's "correct" state (required ∩
    // active) and the check mark appears.
    expect(painter(tester).requiredNotes, {60, 64, 67});
    expect(painter(tester).activeNotes, {60, 64, 67});
    expect(find.byIcon(Icons.check_circle), findsOneWidget);

    // The chord is strummed low-to-high after the three tap sounds.
    await tester.pump(const Duration(milliseconds: 250));
    expect(noteOnPitches().sublist(3), [60, 64, 67]);

    // Strum ends (~720 ms), then chime + 450 ms → onCompleted, exactly once.
    expect(completions, isEmpty);
    await tester.pump(const Duration(milliseconds: 500));
    await tester.pump(const Duration(milliseconds: 500));
    expect(completions, [true]);
    await tester.pump(const Duration(seconds: 1));
    expect(completions, [true]);
  });

  testWidgets('a wrong full set strums, keeps the selection, and a later fix '
      'completes without flawless', (tester) async {
    await pump(tester);

    await tapKey(tester, 60);
    await tapKey(tester, 64);
    await tapKey(tester, 65); // F4 — wrong

    // The wrong attempt keeps the selection, flashes only the stray key, and
    // never shows a success state.
    expect(painter(tester).selectedNotes, {60, 64, 65});
    expect(painter(tester).activeNotes, {65});
    expect(painter(tester).requiredNotes, isEmpty);
    expect(find.byIcon(Icons.check_circle), findsNothing);

    // What was built is strummed, so the error is heard.
    await tester.pump(const Duration(milliseconds: 250));
    expect(noteOnPitches().sublist(3), [60, 64, 65]);

    // Flash over, selection still editable and intact.
    await tester.pump(const Duration(milliseconds: 600));
    expect(painter(tester).activeNotes, isEmpty);
    expect(painter(tester).selectedNotes, {60, 64, 65});

    // Fix it: F4 off, G4 on → completes, but the miss cost flawless.
    await tapKey(tester, 65);
    expect(painter(tester).selectedNotes, {60, 64});
    await tapKey(tester, 67);
    await tester.pump(const Duration(milliseconds: 1300));
    expect(completions, [false]);
  });

  testWidgets('a MIDI note toggles the selection without an app-synth tap', (
    tester,
  ) async {
    await pump(tester);

    // The instrument already sounds, so no synth tap is layered on top.
    // (Two pumps: one delivers the stream event, one renders the toggle.)
    midi.emit(noteOnEvent(60));
    await tester.pump();
    await tester.pump();
    expect(painter(tester).selectedNotes, {60});
    expect(audio.noteOns, isEmpty);

    midi.emit(noteOnEvent(60));
    await tester.pump();
    await tester.pump();
    expect(painter(tester).selectedNotes, isEmpty);
    expect(audio.noteOns, isEmpty);
  });
}
