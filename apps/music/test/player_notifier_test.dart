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
import 'package:music/src/rust/api/score.dart';
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

  Future<void> build({FakeMidiService? service, Score? score}) async {
    midi = service ?? FakeMidiService();
    container = ProviderContainer(
      overrides: [
        midiServiceProvider.overrideWithValue(midi),
        scoreSourceProvider.overrideWithValue(FakeScoreSource(score)),
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

    test('setKeyboardRange updates mode and keyboardBounds', () async {
      await build();
      // Defaults to the full 88-key piano.
      expect(read().keyboardRange, KeyboardRangeMode.keys88);
      expect(read().keyboardBounds.low, 21);
      expect(read().keyboardBounds.high, 108);

      // Switching to auto fits the fake score (pitches 60 & 62).
      notifier().setKeyboardRange(KeyboardRangeMode.auto);
      expect(read().keyboardRange, KeyboardRangeMode.auto);
      final auto = read().keyboardBounds;
      expect(auto.low, lessThanOrEqualTo(60));
      expect(auto.high, greaterThanOrEqualTo(62));
    });
  });

  group('hand selection', () {
    test('defaults to both', () async {
      await build();
      expect(read().selectedHands, Hand.both);
    });

    test('setSelectedHands updates state immutably', () async {
      await build();
      final before = read();
      notifier().setSelectedHands(Hand.left);
      expect(read().selectedHands, Hand.left);
      // The previous immutable snapshot is untouched (copyWith made a new one).
      expect(before.selectedHands, Hand.both);

      notifier().setSelectedHands(Hand.right);
      expect(read().selectedHands, Hand.right);
    });

    test('switching hands re-arms the onset gate', () async {
      await build();
      notifier().togglePlay();
      notifier().noteOn(60); // latch the C4 onset
      expect(read().gateSatisfied, contains(60));
      notifier().setSelectedHands(Hand.left);
      expect(read().gateSatisfied, isEmpty);
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

  group('wait mode (onset gate)', () {
    test('a single press releases the gate without holding the note', () async {
      await build(); // demo: C4 [0,500), D4 [500,1000)
      notifier().togglePlay();
      notifier().advance(50);
      expect(read().blocked, isTrue); // frozen on the C4 onset

      notifier().noteOn(60);
      notifier().advance(50);
      expect(read().blocked, isFalse);
      final moved = read().elapsedMs;
      expect(moved, greaterThan(0));

      // Release the note: it must NOT re-freeze mid-note (the old bug).
      notifier().noteOff(60);
      notifier().advance(50);
      expect(read().blocked, isFalse);
      expect(read().elapsedMs, greaterThan(moved));
    });

    test('continues automatically and freezes at the next onset', () async {
      await build();
      notifier().togglePlay();
      notifier().noteOn(60); // satisfy the first onset
      notifier().advance(50); // unblock and start moving
      notifier().noteOff(60);
      notifier().advance(1000); // travels but clamps at the D4 onset (500)
      expect(read().elapsedMs, 500);

      notifier().advance(50); // now waiting on D4, not yet pressed
      expect(read().blocked, isTrue);
      expect(read().elapsedMs, 500);
      expect(read().expectedKeys, {62}); // preview moved to the next note
    });

    test('an early press does not pre-satisfy the next onset', () async {
      await build(); // C4 [0,500), D4 [500,1000)
      notifier().togglePlay();
      notifier().noteOn(60);
      notifier().advance(50); // pass the C4 onset
      notifier().noteOff(60);

      // Press D4 early, while travelling well before its 500ms onset.
      notifier().noteOn(62);
      notifier().advance(1000); // clamps to the D4 onset (500)
      expect(read().elapsedMs, 500);
      notifier().advance(50);
      expect(read().blocked, isTrue); // the early press did not count
    });

    test('a repeated pitch must be attacked again at the next onset', () async {
      // Same pitch (C4) on two consecutive onsets.
      await build(
        score: Score(
          bpm: 80,
          measures: [
            Measure(
              index: 0,
              notes: [
                Note(
                  pitch: 60,
                  startMs: BigInt.zero,
                  durationMs: BigInt.from(500),
                ),
                Note(
                  pitch: 60,
                  startMs: BigInt.from(500),
                  durationMs: BigInt.from(500),
                ),
              ],
            ),
          ],
        ),
      );
      notifier().togglePlay();
      notifier().noteOn(60);
      notifier().advance(50); // pass the first C4 (key stays held)
      notifier().advance(1000); // clamp to the second C4 onset (500)
      expect(read().elapsedMs, 500);
      notifier().advance(50);
      expect(read().blocked, isTrue); // a held key does not carry over

      notifier().noteOff(60);
      notifier().noteOn(60); // fresh attack
      notifier().advance(50);
      expect(read().blocked, isFalse);
    });

    test('a chord onset requires every pitch before releasing', () async {
      // C4 + E4 together at 0, then D4 at 500.
      await build(
        score: Score(
          bpm: 80,
          measures: [
            Measure(
              index: 0,
              notes: [
                Note(
                  pitch: 60,
                  startMs: BigInt.zero,
                  durationMs: BigInt.from(500),
                ),
                Note(
                  pitch: 64,
                  startMs: BigInt.zero,
                  durationMs: BigInt.from(500),
                ),
                Note(
                  pitch: 62,
                  startMs: BigInt.from(500),
                  durationMs: BigInt.from(500),
                ),
              ],
            ),
          ],
        ),
      );
      notifier().togglePlay();
      notifier().advance(50);
      expect(read().blocked, isTrue);

      notifier().noteOn(60); // only half the chord
      notifier().advance(50);
      expect(read().blocked, isTrue);

      notifier().noteOn(64); // the rest of the chord
      notifier().advance(50);
      expect(read().blocked, isFalse);
      expect(read().elapsedMs, greaterThan(0));
    });
  });
}
