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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/src/rust/api/midi.dart' show MidiEcho;
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/performance_scoring.dart';

import '../support/fakes.dart';

/// The instrument-sounds-itself rule (change: add-audio-output-routing) is a
/// **per-source** rule about *sounding only*. These tests pin both halves: what
/// stops being synthesized, and everything that must not change.
void main() {
  late RecordingAudioService audio;
  late FakeMidiService midi;
  late ProviderContainer container;

  Player notifier() => container.read(playerProvider.notifier);
  PlayerData state() => container.read(playerProvider);

  setUp(() async {
    audio = RecordingAudioService();
    midi = FakeMidiService(
      ports: const ['Piano'],
      connected: 'Piano',
      echoTo: audio,
    );
    container = ProviderContainer(
      overrides: [
        midiServiceProvider.overrideWithValue(midi),
        scoreSourceProvider.overrideWithValue(FakeScoreSource()),
        audioServiceProvider.overrideWithValue(audio),
      ],
    );
    addTearDown(container.dispose);
    addTearDown(midi.close);
    container.listen(playerProvider, (_, _) {}, fireImmediately: true);
    // Let the demo score load.
    await Future<void>.delayed(Duration.zero);
    await Future<void>.delayed(Duration.zero);
  });

  group('sounding', () {
    test(
      'a MIDI note is not synthesized when the instrument sounds itself',
      () {
        notifier().setInstrumentSoundsItself(enabled: true);
        audio.calls.clear();

        notifier().noteOn(60, source: NoteSource.midiDevice);
        notifier().noteOff(60, source: NoteSource.midiDevice);

        expect(audio.noteOns, isEmpty);
        expect(audio.noteOffs, isEmpty);
      },
    );

    test('the on-screen keyboard still sounds', () {
      notifier().setInstrumentSoundsItself(enabled: true);

      notifier().noteOn(62, source: NoteSource.onScreen);
      notifier().noteOff(62, source: NoteSource.onScreen);

      expect(audio.noteOns.map((n) => n.pitch), [62]);
      expect(audio.noteOffs, [62]);
    });

    test('the computer keyboard still sounds', () {
      notifier().setInstrumentSoundsItself(enabled: true);

      notifier().noteOn(64, source: NoteSource.computerKeyboard);

      expect(audio.noteOns.map((n) => n.pitch), [64]);
    });

    test('a MIDI note is synthesized while the setting is off', () async {
      // Driven through the ENGINE, which is where a live note is sounded since
      // the echo (change: add-drum-input-mapping, beta fix for input latency):
      // the rule under test is unchanged — the setting being off means the note
      // comes out of the app's synth — but the side that plays it moved off the
      // Dart event loop, so the test has to come in the same door a real
      // instrument does.
      midi.emit(noteOnEvent(60));
      await Future<void>.delayed(Duration.zero);

      expect(audio.noteOns.map((n) => n.pitch), [60]);
      expect(midi.echo, MidiEcho.melodic, reason: 'the engine was armed');
      // Exactly once: whichever side sounds it, the other one does not.
      expect(audio.noteOns, hasLength(1));
      expect(state().activeNotes, contains(60));
    });

    test('MIDI events from the stream carry the instrument source', () async {
      notifier().setInstrumentSoundsItself(enabled: true);
      audio.calls.clear();

      midi.emit(noteOnEvent(67));
      await Future<void>.delayed(Duration.zero);

      // Not sounded (the piano already did), but pressed and tracked.
      expect(audio.noteOns, isEmpty);
      expect(state().activeNotes, contains(67));
    });
  });

  group('everything other than sounding is unchanged', () {
    test('the note is still held, scored and shown as feedback', () {
      notifier().setInstrumentSoundsItself(enabled: true);

      notifier().noteOn(60, source: NoteSource.midiDevice);

      expect(state().activeNotes, contains(60));
      // The scorer received the attack even though nothing was synthesized.
      expect(
        container.read(performanceScorerProvider.notifier),
        isA<PerformanceScorer>(),
      );

      notifier().noteOff(60, source: NoteSource.midiDevice);
      expect(state().activeNotes, isNot(contains(60)));
    });

    test('a MIDI note still satisfies the Wait Mode gate', () {
      notifier()
        ..setInstrumentSoundsItself(enabled: true)
        ..togglePlay();
      final onset = state().onsetPitchesAt(state().elapsedMs);
      expect(onset, isNotEmpty);

      // Frozen until the onset is attacked…
      notifier().advance(16);
      expect(state().blocked, isTrue);

      for (final p in onset) {
        notifier().noteOn(p, source: NoteSource.midiDevice);
      }
      notifier().advance(16);

      expect(state().blocked, isFalse);
      expect(state().elapsedMs, greaterThan(0));
    });

    test('score playback and the metronome are unaffected', () {
      notifier()
        ..setInstrumentSoundsItself(enabled: true)
        ..toggleWaitMode() // free run: no gate to satisfy
        ..toggleMetronome()
        ..togglePlay();
      audio.calls.clear();

      // Cross the demo score's first onset (C4 at 0ms) and a beat boundary.
      for (var i = 0; i < 60; i++) {
        notifier().advance(16);
      }

      // The score's own notes went through the synth as always…
      expect(audio.noteOns.map((n) => n.pitch), contains(60));
      // …and so did the clicks.
      expect(audio.metronomeClicks, isNotEmpty);
    });
  });

  test('turning the rule on silences held voices so none can hang', () {
    notifier().noteOn(60, source: NoteSource.midiDevice);
    audio.calls.clear();

    notifier().setInstrumentSoundsItself(enabled: true);

    expect(audio.allNotesOffCount, 1);
  });

  test('the setting is unavailable while no instrument is connected', () async {
    midi.connected = null;
    midi.ports = const [];
    // Let the 1s status poll notice the disconnection.
    await Future<void>.delayed(const Duration(milliseconds: 1100));

    expect(state().instrumentSoundsItselfAvailable, isFalse);
  });
}
