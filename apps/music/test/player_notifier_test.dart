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
import 'package:music/src/rust/api/score.dart';
import 'package:music/state/countdown.dart';
import 'package:music/state/performance_scoring.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';

import 'support/fakes.dart';

/// Lets the async score load and the broadcast MIDI stream settle.
Future<void> _flush() => Future<void>.delayed(Duration.zero);

void main() {
  late FakeMidiService midi;
  late RecordingAudioService audio;
  late ProviderContainer container;

  Player notifier() => container.read(playerProvider.notifier);
  PlayerData read() => container.read(playerProvider);

  Future<void> build({
    FakeMidiService? service,
    RecordingAudioService? audioService,
    Score? score,
  }) async {
    midi = service ?? FakeMidiService();
    audio = audioService ?? RecordingAudioService();
    container = ProviderContainer(
      overrides: [
        midiServiceProvider.overrideWithValue(midi),
        scoreSourceProvider.overrideWithValue(FakeScoreSource(score)),
        audioServiceProvider.overrideWithValue(audio),
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

    test(
      'startPlayback arms a countdown from the top, freezing the playhead',
      () async {
        await build();
        notifier().toggleWaitMode(); // countdown is a free-run feature
        notifier().startPlayback();
        expect(read().isPlaying, isTrue);
        expect(read().countdownMs, greaterThan(0));
        // While the countdown ticks, the playhead stays at 0.
        notifier().advance(500);
        expect(read().elapsedMs, 0);
        expect(read().countdownMs, lessThan(kCountdownStartMs));
        // Once the countdown elapses, playback advances normally.
        notifier().advance(kCountdownStartMs);
        expect(read().countdownMs, 0);
        notifier().advance(200);
        expect(read().elapsedMs, greaterThan(0));
      },
    );

    test('Wait Mode plays immediately with no countdown', () async {
      await build(); // Wait Mode is on by default
      notifier().startPlayback();
      expect(read().isPlaying, isTrue);
      expect(read().countdownMs, 0);
    });

    test('resuming mid-piece plays immediately (no countdown)', () async {
      await build();
      notifier().toggleWaitMode();
      notifier().setPlaying(true);
      notifier().advance(300); // move off the top
      notifier().setPlaying(false);
      notifier().startPlayback(); // resume from 300ms
      expect(read().countdownMs, 0);
    });

    test('presses during the countdown are not scored (warm-up)', () async {
      await build();
      notifier().toggleWaitMode(); // free run → countdown active
      notifier().startPlayback();
      // A warm-up press while counting down does not register with the scorer.
      notifier().noteOn(60);
      final scoring = container.read(performanceScorerProvider);
      expect(scoring.active, isTrue); // the run is armed
      expect(scoring.recentHits, isEmpty); // but the press was not scored
      expect(scoring.combo, 0);
    });

    test('changing hands mid-run restarts scoring from the top', () async {
      await build();
      // Synthesia (default) + play from the top opens a scored run.
      notifier().togglePlay();
      notifier().toggleWaitMode(); // free run so the playhead advances
      notifier().advance(200);
      expect(read().elapsedMs, greaterThan(0));
      expect(container.read(performanceScorerProvider).active, isTrue);
      // A hand switch restarts the piece with a fresh, still-active run for the
      // new selection (demo notes are right-hand, so a run opens).
      notifier().setSelectedHands(Hand.right);
      expect(read().elapsedMs, 0);
      expect(container.read(performanceScorerProvider).active, isTrue);
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
      // A scored run finishes instead of looping; cancel it to exercise the
      // unscored loop path (all render modes are scored now).
      container.read(performanceScorerProvider.notifier).cancelRun();
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

    test(
      'an early press that is released does not pre-satisfy the onset',
      () async {
        await build(); // C4 [0,500), D4 [500,1000)
        notifier().togglePlay();
        notifier().noteOn(60);
        notifier().advance(50); // pass the C4 onset
        notifier().noteOff(60);

        // Press D4 early then release it, both before its 500ms onset.
        notifier().noteOn(62);
        notifier().noteOff(62);
        notifier().advance(1000); // clamps to the D4 onset (500)
        expect(read().elapsedMs, 500);
        notifier().advance(50);
        expect(
          read().blocked,
          isTrue,
        ); // the released early press did not count
      },
    );

    test(
      'a pitch held through the onset satisfies without a re-press',
      () async {
        await build(); // C4 [0,500), D4 [500,1000)
        notifier().togglePlay();
        notifier().noteOn(60);
        notifier().advance(50); // pass the C4 onset
        notifier().noteOff(60);

        // Press D4 early and KEEP holding it through its 500ms onset.
        notifier().noteOn(62);
        notifier().advance(1000); // clamps to the D4 onset (500)
        expect(read().elapsedMs, 500);
        notifier().advance(50);
        expect(
          read().blocked,
          isFalse,
        ); // the sustained hold satisfied the onset
        expect(read().elapsedMs, greaterThan(500));
      },
    );

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

    test(
      'a chord onset accepts a held pitch alongside a fresh press',
      () async {
        // C4 at 0, then E4 + G4 together at 500.
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
                    startMs: BigInt.from(500),
                    durationMs: BigInt.from(500),
                  ),
                  Note(
                    pitch: 67,
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
        notifier().advance(50); // pass the C4 onset
        notifier().noteOff(60);

        // Hold E4 from before the chord onset, then travel to it.
        notifier().noteOn(64);
        notifier().advance(1000); // clamp to the chord onset (500)
        expect(read().elapsedMs, 500);

        notifier().advance(50); // held E4 counts, but G4 is still missing
        expect(read().blocked, isTrue);

        notifier().noteOn(67); // complete the chord with a fresh press
        notifier().advance(50);
        expect(read().blocked, isFalse);
        expect(read().elapsedMs, greaterThan(500));
      },
    );

    test(
      'a held pitch does not satisfy a later repeat onset (N and N+2)',
      () async {
        // C4 at 0, D4 at 500, C4 again at 1000; C4 held continuously.
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
                    pitch: 62,
                    startMs: BigInt.from(500),
                    durationMs: BigInt.from(500),
                  ),
                  Note(
                    pitch: 60,
                    startMs: BigInt.from(1000),
                    durationMs: BigInt.from(500),
                  ),
                ],
              ),
            ],
          ),
        );
        notifier().togglePlay();
        notifier().noteOn(60); // satisfy the first C4 onset (and keep holding)
        notifier().advance(50);
        notifier().advance(1000); // clamp to the D4 onset (500)
        expect(read().elapsedMs, 500);

        notifier().noteOn(62); // satisfy the D4 onset
        notifier().advance(50);
        notifier().advance(1000); // clamp to the second C4 onset (1000)
        expect(read().elapsedMs, 1000);

        notifier().advance(50);
        // C4 has been held the whole time but was consumed at onset 0.
        expect(read().blocked, isTrue);

        notifier().noteOff(60);
        notifier().noteOn(60); // fresh attack
        notifier().advance(50);
        expect(read().blocked, isFalse);
      },
    );
  });

  group('audio — startup & live input (5.1)', () {
    test('initializes the audio engine once at startup', () async {
      await build();
      expect(audio.initCount, 1);
    });

    test('noteOn / noteOff sound and release the piano voice', () async {
      await build();
      notifier().noteOn(60);
      expect(audio.noteOns, contains((pitch: 60, velocity: 100)));
      notifier().noteOff(60);
      expect(audio.noteOffs, contains(60));
    });

    test('MIDI stream events sound through the synth', () async {
      await build();
      midi.emit(noteOnEvent(67));
      await _flush();
      expect(audio.noteOns.map((e) => e.pitch), contains(67));
    });

    test('a key sounds even while playback is stopped', () async {
      await build();
      expect(read().isPlaying, isFalse);
      notifier().noteOn(60);
      expect(audio.noteOns.map((e) => e.pitch), contains(60));
    });
  });

  group('audio — score playback (5.2)', () {
    test('advancing across an onset sounds the note, releases it at its '
        'end', () async {
      await build(); // C4 [0,500), D4 [500,1000)
      notifier().toggleWaitMode(); // free-run
      notifier().togglePlay();

      notifier().advance(50); // cross C4's onset
      expect(audio.noteOns.map((e) => e.pitch), contains(60));

      notifier().advance(500); // playhead → 550: pass C4's end, reach D4
      expect(audio.noteOffs, contains(60));
      expect(audio.noteOns.map((e) => e.pitch), contains(62));
    });

    test(
      'a wider tick (faster speed) sounds more onsets in one step',
      () async {
        await build();
        notifier().toggleWaitMode();
        notifier().togglePlay();
        // One big advance (as a higher speed multiplier would scale dt) crosses
        // both onsets at once → tighter spacing.
        notifier().advance(900);
        final sounded = audio.noteOns.map((e) => e.pitch).toSet();
        expect(sounded, containsAll(<int>[60, 62]));
      },
    );

    test('stopping and restarting each issue all-notes-off', () async {
      await build();
      notifier().toggleWaitMode();
      notifier().togglePlay(); // play
      notifier().advance(50);

      notifier().togglePlay(); // pause → all-notes-off
      final afterPause = audio.allNotesOffCount;
      expect(afterPause, greaterThanOrEqualTo(1));

      notifier().restart(); // → another all-notes-off
      expect(audio.allNotesOffCount, greaterThan(afterPause));
    });

    test('looping at the end silences all voices', () async {
      await build();
      notifier().toggleWaitMode();
      notifier().togglePlay();
      // Unscored playback loops; cancel the auto-started run to reach it.
      container.read(performanceScorerProvider.notifier).cancelRun();
      notifier().advance(500);
      final before = audio.allNotesOffCount;
      notifier().advance(600); // crosses songEnd (1000) → loop
      expect(read().elapsedMs, 0);
      expect(audio.allNotesOffCount, greaterThan(before));
    });
  });

  group('audio — wait mode (5.3)', () {
    // Hidden-hand silencing is driven by `visibleNotes` (covered in
    // hand_selection_test) feeding `scoreNoteEdges` (covered in
    // score_audio_edges_test); staff data only exists on parsed notation, not on
    // the demo Score used here, so it is asserted at those two layers.
    test(
      'a frozen Wait Mode onset does not pre-sound the awaited note',
      () async {
        await build(); // wait mode on by default
        notifier().togglePlay();
        notifier().advance(50); // frozen on the C4 onset
        expect(read().blocked, isTrue);
        // The score must not have sounded the awaited note while frozen.
        expect(audio.noteOns, isEmpty);

        // Press the note: it sounds live, the gate releases…
        notifier().noteOn(60);
        expect(audio.noteOns.map((e) => e.pitch), contains(60));
        // …and once time advances past the onset the score sounds it too.
        audio.noteOns.clear();
        notifier().advance(50);
        expect(read().blocked, isFalse);
        expect(audio.noteOns.map((e) => e.pitch), contains(60));
      },
    );
  });

  group('audio — graceful degradation (5.4)', () {
    test('a failed audio service leaves the player fully functional', () async {
      await build(audioService: RecordingAudioService(failInit: true));
      expect(audio.calls, contains('init:fail'));

      // Visuals/feedback: input still updates state.
      notifier().noteOn(60);
      expect(read().activeNotes, contains(60));

      // Wait Mode still gates and releases as usual.
      expect(read().waitMode, isTrue);
      notifier().togglePlay();
      notifier().advance(50);
      expect(read().blocked, isFalse); // 60 is held → gate satisfied
      expect(read().elapsedMs, greaterThan(0));
    });
  });

  group('metronome', () {
    // The demo Score (bpm 80) carries no measure table, so the metronome uses the
    // tempo fallback: a steady beat every 60000/80 = 750ms, accenting beat 0.
    test('toggleMetronome flips the flag and it survives pause', () async {
      await build();
      expect(read().metronomeEnabled, isFalse);
      notifier().toggleMetronome();
      expect(read().metronomeEnabled, isTrue);
      // Pausing must not clear the preference.
      notifier().setPlaying(false);
      expect(read().metronomeEnabled, isTrue);
      notifier().toggleMetronome();
      expect(read().metronomeEnabled, isFalse);
    });

    test('the enabled flag is global and survives a notifier rebuild', () async {
      await build();
      notifier().toggleMetronome();
      expect(read().metronomeEnabled, isTrue);
      // The app-wide provider holds the choice independently of the player.
      expect(container.read(metronomeEnabledProvider), isTrue);

      // Leaving the player and reopening it on another piece auto-disposes and
      // rebuilds the notifier; the global flag must seed the fresh state.
      container.invalidate(playerProvider);
      await _flush();
      expect(read().metronomeEnabled, isTrue);
    });

    test('enabled + playing clicks and pulses on each beat', () async {
      await build();
      notifier().toggleWaitMode(); // free-run so the playhead isn't gated
      notifier().togglePlay();
      notifier().toggleMetronome();

      notifier().advance(100); // span [0,100): the downbeat at 0
      expect(audio.metronomeClicks, [true]);
      expect(read().beatCount, 1);
      expect(read().lastBeatAccent, isTrue);

      notifier().advance(700); // span [100,800): the normal beat at 750
      expect(audio.metronomeClicks, [true, false]);
      expect(read().beatCount, 2);
      expect(read().lastBeatAccent, isFalse);
    });

    test('silent while paused even when enabled', () async {
      await build();
      notifier().toggleWaitMode();
      notifier().toggleMetronome(); // enabled but NOT playing
      notifier().advance(800);
      expect(audio.metronomeClicks, isEmpty);
      expect(read().beatCount, 0);
    });

    test('no clicks while disabled', () async {
      await build();
      notifier().toggleWaitMode();
      notifier().togglePlay(); // playing but metronome off
      notifier().advance(800);
      expect(audio.metronomeClicks, isEmpty);
      expect(read().beatCount, 0);
    });

    test('no tick across a loop seam, resumes on the next downbeat', () async {
      await build();
      notifier().toggleWaitMode();
      notifier().togglePlay();
      // Unscored playback loops; cancel the auto-started run to reach it.
      container.read(performanceScorerProvider.notifier).cancelRun();
      notifier().toggleMetronome();

      notifier().advance(900); // beats at 0 (accent) and 750
      expect(read().beatCount, 2);
      final beforeLoop = audio.metronomeClicks.length;

      notifier().advance(200); // 900→1100 ≥ songEnd(1000): loops, no seam tick
      expect(read().elapsedMs, 0);
      expect(audio.metronomeClicks.length, beforeLoop); // nothing added

      notifier().advance(100); // back at the top → downbeat fires again
      expect(audio.metronomeClicks.length, beforeLoop + 1);
      expect(audio.metronomeClicks.last, isTrue);
    });
  });
}
