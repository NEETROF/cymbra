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
import 'package:music/widgets/read_play_view.dart';

import '../support/fakes.dart';

ReadPlayBlock block(
  List<String> notes, {
  ReadPlayMode mode = ReadPlayMode.drill,
  LessonLabelMode labels = LessonLabelMode.afterMiss,
}) => ReadPlayBlock(
  notes: [for (final n in notes) LessonPitch.parse(n)!],
  mode: mode,
  labels: labels,
  prompt: const {'en': 'Play what you read', 'fr': 'Jouez ce qui est écrit'},
);

PianoKeyboardPainter painterOf(WidgetTester tester) =>
    tester
            .widget<CustomPaint>(find.byKey(const Key('readplay-keyboard')))
            .painter!
        as PianoKeyboardPainter;

/// Taps the on-screen key for [pitch], resolving its x through the same
/// [PianoLayout] the widget painted with (read back from the painter). White
/// keys are hit below the black-key band, black keys inside it.
Future<void> tapKey(WidgetTester tester, int pitch) async {
  final finder = find.byKey(const Key('readplay-keyboard'));
  final layout = painterOf(tester).layout;
  final y = PianoLayout.isBlack(pitch) ? 40.0 : 110.0;
  await tester.tapAt(
    tester.getTopLeft(finder) + Offset(layout.centerX(pitch), y),
  );
  await tester.pump();
}

void main() {
  Future<({FakeMidiService midi, RecordingAudioService audio, List<bool> done})>
  pumpView(WidgetTester tester, ReadPlayBlock b) async {
    final midi = FakeMidiService();
    addTearDown(midi.close);
    final audio = RecordingAudioService();
    final done = <bool>[];
    final container = ProviderContainer(
      overrides: [
        audioServiceProvider.overrideWithValue(audio),
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
          locale: const Locale('fr'),
          home: Scaffold(
            body: ReadPlayView(
              block: b,
              onCompleted: ({required bool flawless}) => done.add(flawless),
            ),
          ),
        ),
      ),
    );
    return (midi: midi, audio: audio, done: done);
  }

  /// Lets every short timer (press flash, drill flash, chime, completion)
  /// elapse so no test ends with a pending timer.
  Future<void> settle(WidgetTester tester) async {
    await tester.pump(const Duration(seconds: 2));
  }

  testWidgets('drill advances on correct taps and completes flawless', (
    tester,
  ) async {
    final h = await pumpView(tester, block(['C4', 'D4']));
    expect(find.text('Jouez ce qui est écrit'), findsOneWidget);
    expect(find.byKey(const Key('readplay-dot-0')), findsOneWidget);
    expect(find.byKey(const Key('readplay-dot-1')), findsOneWidget);
    expect(painterOf(tester).requiredNotes, {60});

    await tapKey(tester, 60);
    // The tap itself sounded through the lesson sounder.
    expect(h.audio.noteOns.map((n) => n.pitch), contains(60));
    // Validation flash, then the queue advances to the next note.
    await tester.pump(const Duration(milliseconds: 400));
    expect(painterOf(tester).requiredNotes, {62});

    await tapKey(tester, 62);
    await tester.pump(const Duration(milliseconds: 400));
    expect(find.byIcon(Icons.check_circle), findsOneWidget);
    await tester.pump(const Duration(milliseconds: 500));
    expect(h.done, [true]);
    // The success chime sounded (rising arpeggio on 84).
    expect(h.audio.noteOns.map((n) => n.pitch), containsAll([84, 88, 91]));
    await settle(tester);
  });

  testWidgets('two misses reveal the note label and cost flawless', (
    tester,
  ) async {
    final h = await pumpView(tester, block(['C4']));
    await tapKey(tester, 62); // wrong
    expect(painterOf(tester).noteLabels, isEmpty); // one miss is not enough
    await tapKey(tester, 64); // wrong again, same target
    expect(painterOf(tester).noteLabels, {60: 'Do'}); // fr → solfège

    await tapKey(tester, 60);
    await tester.pump(const Duration(milliseconds: 400));
    await tester.pump(const Duration(milliseconds: 500));
    expect(h.done, [false]);
    await settle(tester);
  });

  testWidgets('melody mode enforces order without resetting progress', (
    tester,
  ) async {
    final h = await pumpView(
      tester,
      block(['C4', 'E4'], mode: ReadPlayMode.melody),
    );
    await tapKey(tester, 64); // out of order: judged wrong, no advance
    expect(painterOf(tester).requiredNotes, {60});

    await tapKey(tester, 60);
    expect(painterOf(tester).requiredNotes, {64});
    await tapKey(tester, 64);
    await tester.pump(const Duration(milliseconds: 500));
    // The stray tap counted as a miss, but never undid the progress.
    expect(h.done, [false]);
    await settle(tester);
  });

  testWidgets('set mode completes in any order', (tester) async {
    final h = await pumpView(
      tester,
      block(['C4', 'E4', 'G4'], mode: ReadPlayMode.set),
    );
    expect(painterOf(tester).requiredNotes, {60, 64, 67});
    await tapKey(tester, 67);
    expect(painterOf(tester).requiredNotes, {60, 64});
    await tapKey(tester, 60);
    await tapKey(tester, 64);
    await tester.pump(const Duration(milliseconds: 500));
    expect(h.done, [true]);
    await settle(tester);
  });

  testWidgets('MIDI input advances but is not echoed by the sounder', (
    tester,
  ) async {
    final h = await pumpView(tester, block(['C4']));
    h.midi.emit(noteOnEvent(60));
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 400));
    await tester.pump(const Duration(milliseconds: 500));
    expect(h.done, [true]);
    // Only the chime sounded — the instrument itself already made the note.
    expect(h.audio.noteOns.map((n) => n.pitch).toSet(), {84, 88, 91});
    await settle(tester);
  });

  testWidgets('onCompleted fires exactly once, then input is ignored', (
    tester,
  ) async {
    final h = await pumpView(tester, block(['C4']));
    await tapKey(tester, 60);
    await tester.pump(const Duration(milliseconds: 400));
    await tester.pump(const Duration(milliseconds: 500));
    expect(h.done, [true]);

    final soundsAfter = h.audio.noteOns.length;
    await tapKey(tester, 60);
    h.midi.emit(noteOnEvent(60));
    await settle(tester);
    expect(h.done, [true]); // still exactly one call
    expect(h.audio.noteOns.length, soundsAfter); // done: taps no longer sound
  });
}
