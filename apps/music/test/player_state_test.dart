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

import 'package:flutter_test/flutter_test.dart';
import 'package:music/state/player_state.dart';

import 'support/fakes.dart';

/// Lets the broadcast MIDI stream deliver queued events.
Future<void> _flush() => Future<void>.delayed(Duration.zero);

void main() {
  late FakeMidiService midi;
  late PlayerState state;

  Future<void> build({FakeMidiService? service}) async {
    midi = service ?? FakeMidiService();
    state = PlayerState(midi: midi, scores: FakeScoreSource());
    await state.init();
  }

  tearDown(() async {
    state.dispose();
    await midi.close();
  });

  group('init / score flattening', () {
    test('loads the score and flattens notes sorted by start', () async {
      await build();
      expect(state.score, isNotNull);
      expect(state.notes.map((n) => n.pitch).toList(), [60, 62]);
      expect(state.notes.first.startMs, 0);
      expect(state.songEndMs, 1000);
    });
  });

  group('note input', () {
    test('noteOn / noteOff update activeNotes', () async {
      await build();
      state.noteOn(60);
      expect(state.activeNotes, contains(60));
      state.noteOff(60);
      expect(state.activeNotes, isNot(contains(60)));
    });

    test('MIDI stream events drive activeNotes', () async {
      await build();
      midi.emit(noteOnEvent(67));
      await _flush();
      expect(state.activeNotes, contains(67));
      midi.emit(noteOffEvent(67));
      await _flush();
      expect(state.activeNotes, isNot(contains(67)));
    });
  });

  group('MIDI status', () {
    test('reflects detected ports and connection', () async {
      await build(service: FakeMidiService(ports: ['Piano'], connected: 'Piano'));
      expect(state.midiPorts, ['Piano']);
      expect(state.connectedDevice, 'Piano');
      expect(state.midiConnected, isTrue);
    });

    test('selectMidiPort forwards to the engine and refreshes', () async {
      await build(service: FakeMidiService(ports: ['Piano', 'Synth']));
      state.selectMidiPort('Synth');
      expect(midi.selectPortCalls, ['Synth']);
      expect(state.connectedDevice, 'Synth');
    });
  });

  group('playback controls', () {
    test('togglePlay, setMode, toggleWaitMode, setSpeed, restart', () async {
      await build();
      expect(state.isPlaying, isFalse);
      state.togglePlay();
      expect(state.isPlaying, isTrue);

      state.setMode(RenderMode.staff);
      expect(state.mode, RenderMode.staff);

      final wait = state.waitMode;
      state.toggleWaitMode();
      expect(state.waitMode, !wait);

      state.setSpeed(5.0); // clamped to 2.0
      expect(state.speed, 2.0);
      state.setSpeed(0.0); // clamped to 0.25
      expect(state.speed, 0.25);

      state.elapsedMs = 500;
      state.restart();
      expect(state.elapsedMs, 0);
    });
  });

  group('time advance + wait mode', () {
    test('requiredNotesAt returns the note under the playhead', () async {
      await build();
      expect(state.requiredNotesAt(0), {60});
      expect(state.requiredNotesAt(500), {62});
      expect(state.requiredNotesAt(1500), isEmpty);
    });

    test('does nothing while paused', () async {
      await build();
      state.advance(100);
      expect(state.elapsedMs, 0);
    });

    test('wait mode freezes until the required note is held', () async {
      await build();
      state.togglePlay(); // play
      expect(state.waitMode, isTrue);

      // Required note (60) not held → blocked, no progress.
      state.advance(100);
      expect(state.blocked, isTrue);
      expect(state.elapsedMs, 0);

      // Hold the note → unblocks and advances (clamped to next note start).
      state.noteOn(60);
      state.advance(100);
      expect(state.blocked, isFalse);
      expect(state.elapsedMs, greaterThan(0));
    });

    test('without wait mode the playhead advances freely', () async {
      await build();
      state.toggleWaitMode(); // disable
      state.togglePlay();
      state.advance(120);
      expect(state.elapsedMs, 120);
    });

    test('loops back to start at the end of the song', () async {
      await build();
      state.toggleWaitMode(); // disable wait so it can run to the end
      state.togglePlay();
      state.elapsedMs = 999;
      state.advance(50); // crosses songEndMs (1000)
      expect(state.elapsedMs, 0);
    });
  });
}
