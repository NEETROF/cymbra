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
import 'package:music/courses/lesson_rhythm.dart';
import 'package:music/l10n/gen/app_localizations.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/state/note_label.dart' show NoteFigure;
import 'package:music/widgets/rhythm_tap_view.dart';

import '../support/fakes.dart';

const _q = RhythmFigure(NoteFigure.quarter);
const _qRest = RhythmFigure(NoteFigure.quarter, rest: true);

/// Two quarters in a 2/4 bar at 60 bpm: beat = 1000 ms, count-in = 2000 ms,
/// onsets at 0 and 1000, pattern end 2000 + 500 ms grace. Slow and short so a
/// widget test can pump through a whole pass in a sane wall time.
const _twoQuarters = RhythmTapBlock(
  pattern: [_q, _q],
  beats: 2,
  beatType: 4,
  bpm: 60,
  prompt: {'fr': 'Tape le rythme'},
);

void main() {
  Future<void> pumpView(
    WidgetTester tester, {
    required RecordingAudioService audio,
    required List<bool> completions,
    RhythmTapBlock block = _twoQuarters,
    FakeMidiService? midi,
  }) async {
    final fakeMidi = midi ?? FakeMidiService();
    addTearDown(fakeMidi.close);
    final container = ProviderContainer(
      overrides: [
        audioServiceProvider.overrideWithValue(audio),
        midiServiceProvider.overrideWithValue(fakeMidi),
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
            body: RhythmTapView(
              block: block,
              onCompleted: ({required bool flawless}) =>
                  completions.add(flawless),
            ),
          ),
        ),
      ),
    );
  }

  /// Advances the ticker in 100 ms frames ([ms] must be a multiple of 100).
  Future<void> advance(WidgetTester tester, int ms) async {
    for (var t = 0; t < ms; t += 100) {
      await tester.pump(const Duration(milliseconds: 100));
    }
  }

  /// Taps Commencer and pumps through the one-bar count-in into tapping.
  Future<void> startIntoTapping(WidgetTester tester) async {
    await tester.tap(find.byKey(const Key('rhythm-start')));
    await tester.pump();
    await advance(tester, 2000);
    expect(find.byKey(const Key('rhythm-pad')), findsOneWidget);
  }

  testWidgets('renders one glyph per pattern element, including a rest', (
    tester,
  ) async {
    await pumpView(
      tester,
      audio: RecordingAudioService(),
      completions: [],
      block: const RhythmTapBlock(
        pattern: [_q, _qRest, _q],
        beats: 3,
        beatType: 4,
        bpm: 60,
        prompt: {'fr': 'Tape le rythme'},
      ),
    );
    expect(find.byKey(const Key('rhythm-glyph-0')), findsOneWidget);
    expect(find.byKey(const Key('rhythm-glyph-1')), findsOneWidget);
    expect(find.byKey(const Key('rhythm-glyph-2')), findsOneWidget);
    expect(find.byKey(const Key('rhythm-glyph-3')), findsNothing);
    expect(find.text('Tape le rythme'), findsOneWidget);
    expect(find.text('Écouter'), findsOneWidget);
    expect(find.text('Commencer'), findsOneWidget);
  });

  testWidgets('count-in fires one bar of clicks, first accented, then the '
      'pattern window follows on the same grid', (tester) async {
    final audio = RecordingAudioService();
    await pumpView(tester, audio: audio, completions: []);
    await tester.tap(find.byKey(const Key('rhythm-start')));
    await tester.pump();
    await advance(tester, 1900);
    // Exactly the bar's two count-in beats so far, downbeat accented.
    expect(audio.metronomeClicks, [true, false]);
    expect(find.byKey(const Key('rhythm-pad')), findsNothing);
    await advance(tester, 200);
    // The pattern window opened and its downbeat continued the same grid.
    expect(find.byKey(const Key('rhythm-pad')), findsOneWidget);
    expect(audio.metronomeClicks, [true, false, true]);
  });

  testWidgets('tapping on the onsets passes: chime, sounding pad taps and '
      'onCompleted(flawless: true) exactly once', (tester) async {
    final audio = RecordingAudioService();
    final completions = <bool>[];
    await pumpView(tester, audio: audio, completions: completions);
    await startIntoTapping(tester);

    await tester.tap(find.byKey(const Key('rhythm-pad'))); // onset 0
    await advance(tester, 1000);
    await tester.tap(find.byKey(const Key('rhythm-pad'))); // onset 1000
    // Each tap is heard: the pad sounds the fixed tap pitch.
    expect(audio.noteOns.where((n) => n.pitch == 76).length, 2);

    await advance(tester, 1500); // pattern end (2000) + half-beat grace (500)
    expect(find.text('Bravo !'), findsOneWidget);
    await advance(tester, 500); // success chime + the 450 ms completion delay
    expect(completions, [true]);
    // The chime's rising arpeggio was scheduled.
    expect(audio.noteOns.where((n) => n.pitch == 84).length, 1);
  });

  testWidgets('a garbage pass shows the kind retry, and retrying then '
      'tapping correctly completes with flawless: false', (tester) async {
    final audio = RecordingAudioService();
    final completions = <bool>[];
    await pumpView(tester, audio: audio, completions: completions);
    await startIntoTapping(tester);

    await advance(tester, 400); // 400 ms is outside the ±250 ms window
    await tester.tap(find.byKey(const Key('rhythm-pad')));
    await advance(tester, 2100); // to the end of the window
    expect(find.byKey(const Key('rhythm-retry')), findsOneWidget);
    expect(find.text('Bravo !'), findsNothing);
    expect(completions, isEmpty);

    // Retry replays the demo; starting over interrupts it cleanly.
    await tester.tap(find.byKey(const Key('rhythm-retry')));
    await tester.pump();
    await startIntoTapping(tester);
    await tester.tap(find.byKey(const Key('rhythm-pad')));
    await advance(tester, 1000);
    await tester.tap(find.byKey(const Key('rhythm-pad')));
    await advance(tester, 1500);
    await advance(tester, 500);
    // A perfect second pass still completes — but never as flawless.
    expect(completions, [false]);
  });

  testWidgets('Écouter plays the demo (taps + clicks) without completing', (
    tester,
  ) async {
    final audio = RecordingAudioService();
    final completions = <bool>[];
    await pumpView(tester, audio: audio, completions: completions);
    await tester.tap(find.byKey(const Key('rhythm-listen')));
    await tester.pump();
    await advance(tester, 500);
    expect(audio.noteOns.where((n) => n.pitch == 76).length, 1);
    expect(audio.metronomeClicks, isNotEmpty);
    await advance(tester, 2200); // demo runs out (2000 + half-beat tail)
    expect(audio.noteOns.where((n) => n.pitch == 76).length, 2);
    expect(completions, isEmpty);
    // Still on the intro: the exercise never armed.
    expect(find.byKey(const Key('rhythm-start')), findsOneWidget);
    expect(find.byKey(const Key('rhythm-pad')), findsNothing);
  });

  testWidgets('disposing mid-tapping stops the metronome and leaks nothing', (
    tester,
  ) async {
    final audio = RecordingAudioService();
    await pumpView(tester, audio: audio, completions: []);
    await startIntoTapping(tester);
    await tester.tap(find.byKey(const Key('rhythm-pad')));
    await advance(tester, 100);

    final clicksBefore = audio.metronomeClicks.length;
    final onsBefore = audio.noteOns.length;
    await tester.pumpWidget(const SizedBox());
    expect(tester.takeException(), isNull);
    await tester.pump(const Duration(milliseconds: 300));
    expect(audio.metronomeClicks.length, clicksBefore);
    expect(audio.noteOns.length, onsBefore);
  });

  testWidgets('a connected piano taps the rhythm on any key', (tester) async {
    final audio = RecordingAudioService();
    final completions = <bool>[];
    final midi = FakeMidiService();
    await pumpView(tester, audio: audio, completions: completions, midi: midi);
    await startIntoTapping(tester);

    // Play the two quarters from the instrument — any key counts as the pad.
    midi.emit(noteOnEvent(48));
    await advance(tester, 1000);
    midi.emit(noteOnEvent(52));
    await advance(tester, 2000); // grace, result, chime beat

    expect(completions, [true]);
    // The instrument already sounds: MIDI taps never trigger the synth tick
    // (the only noteOns are the completion chime's).
    expect(audio.noteOns.where((n) => n.pitch == 76), isEmpty);
  });
}
