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
import 'package:music/services/midi_service.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';

import 'support/fakes.dart';

/// Lets the async score load and the broadcast MIDI stream settle.
Future<void> _flush() => Future<void>.delayed(Duration.zero);

void main() {
  late FakeMidiService midi;
  late ProviderContainer container;

  Player notifier() => container.read(playerProvider.notifier);
  PlayerData read() => container.read(playerProvider);

  Future<void> build({FakeMidiService? service}) async {
    midi = service ?? FakeMidiService();
    container = ProviderContainer(
      overrides: [
        midiServiceProvider.overrideWithValue(midi),
        scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      ],
    );
    addTearDown(container.dispose);
    // Keep the auto-dispose provider alive for the duration of the test.
    container.listen(playerProvider, (_, _) {}, fireImmediately: true);
    await _flush(); // let _loadScore resolve
  }

  tearDown(() async => midi.close());

  group('init / score flattening', () {
    test('loads the score and flattens notes sorted by start', () async {
      await build();
      expect(read().score, isNotNull);
      expect(read().notes.map((n) => n.pitch).toList(), [60, 62]);
      expect(read().notes.first.startMs, 0);
      expect(read().songEndMs, 1000);
    });
  });

  group('note input', () {
    test('noteOn / noteOff update activeNotes', () async {
      await build();
      notifier().noteOn(60);
      expect(read().activeNotes, contains(60));
      notifier().noteOff(60);
      expect(read().activeNotes, isNot(contains(60)));
    });

    test('MIDI stream events drive activeNotes', () async {
      await build();
      midi.emit(noteOnEvent(67));
      await _flush();
      expect(read().activeNotes, contains(67));
      midi.emit(noteOffEvent(67));
      await _flush();
      expect(read().activeNotes, isNot(contains(67)));
    });
  });

  group('MIDI status', () {
    test('reflects detected ports and connection', () async {
      await build(
        service: FakeMidiService(ports: ['Piano'], connected: 'Piano'),
      );
      expect(read().midiPorts, ['Piano']);
      expect(read().connectedDevice, 'Piano');
      expect(read().midiConnected, isTrue);
    });

    test('selectMidiPort forwards to the engine and refreshes', () async {
      await build(service: FakeMidiService(ports: ['Piano', 'Synth']));
      notifier().selectMidiPort('Synth');
      expect(midi.selectPortCalls, ['Synth']);
      expect(read().connectedDevice, 'Synth');
    });
  });

  group('playback controls', () {
    test('togglePlay, setMode, toggleWaitMode, setSpeed, restart', () async {
      await build();
      expect(read().isPlaying, isFalse);
      notifier().togglePlay();
      expect(read().isPlaying, isTrue);

      notifier().setMode(RenderMode.staff);
      expect(read().mode, RenderMode.staff);

      final wait = read().waitMode;
      notifier().toggleWaitMode();
      expect(read().waitMode, !wait);

      notifier().setSpeed(5.0); // clamped to 2.0
      expect(read().speed, 2.0);
      notifier().setSpeed(0.0); // clamped to 0.25
      expect(read().speed, 0.25);

      // Advance then restart (wait-mode is already off from the toggle above).
      notifier().advance(120);
      expect(read().elapsedMs, greaterThan(0));
      notifier().restart();
      expect(read().elapsedMs, 0);
    });
  });

  group('time advance + wait mode', () {
    test('requiredNotesAt returns the note under the playhead', () async {
      await build();
      expect(read().requiredNotesAt(0), {60});
      expect(read().requiredNotesAt(500), {62});
      expect(read().requiredNotesAt(1500), isEmpty);
    });

    test('does nothing while paused', () async {
      await build();
      notifier().advance(100);
      expect(read().elapsedMs, 0);
    });

    test('wait mode freezes until the required note is held', () async {
      await build();
      notifier().togglePlay(); // play
      expect(read().waitMode, isTrue);

      notifier().advance(100);
      expect(read().blocked, isTrue);
      expect(read().elapsedMs, 0);

      notifier().noteOn(60);
      notifier().advance(100);
      expect(read().blocked, isFalse);
      expect(read().elapsedMs, greaterThan(0));
    });

    test('without wait mode the playhead advances freely', () async {
      await build();
      notifier().toggleWaitMode(); // disable
      notifier().togglePlay();
      notifier().advance(120);
      expect(read().elapsedMs, 120);
    });

    test('loops back to start at the end of the song', () async {
      await build();
      notifier().toggleWaitMode(); // disable wait
      notifier().togglePlay();
      notifier().advance(500);
      expect(read().elapsedMs, 500);
      notifier().advance(600); // crosses songEndMs (1000)
      expect(read().elapsedMs, 0);
    });
  });
}
