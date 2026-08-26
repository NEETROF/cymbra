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
import 'package:music/src/rust/api/musicxml.dart';
import 'package:music/src/rust/api/score.dart';
import 'package:music/state/countdown.dart';
import 'package:music/state/notation_data.dart';
import 'package:music/state/notation_notifier.dart';
import 'package:music/state/performance_scoring.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/player_preferences.dart';
import 'package:music/state/score_catalog.dart';

import 'support/fakes.dart';
import 'support/notation_fakes.dart';

/// A [Notation] whose state is fixed to a parsed document, so the player loads a
/// real timeline (notes + rests) without touching the byte sources — the only
/// way to get trailing rests (songEndMs > last note) into the player state.
class _FixedNotation extends Notation {
  _FixedNotation(this._value);
  final NotationData _value;
  @override
  NotationData build() => _value;
}

/// Lets the async score load and the broadcast MIDI stream settle.
Future<void> _flush() => Future<void>.delayed(Duration.zero);

/// A parsed document with no `<work-title>` — what several bundled scores (and
/// plenty of user uploads) actually look like.
ScoreDocument _untitled() {
  final titled = sampleFourMeasureDocument();
  return ScoreDocument(
    instruments: const [],
    playOrder: const [],
    meta: const ScoreMeta(composer: 'Christian Petzold'),
    staves: titled.staves,
    attributes: titled.attributes,
    measures: titled.measures,
  );
}

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
    ScoreDocument? document,
    CatalogEntry? entry,
  }) async {
    audio = audioService ?? RecordingAudioService();
    // The fake engine sounds live MIDI notes itself when the app arms the echo
    // (change: add-drum-input-mapping, beta fix for input latency), exactly as
    // the real one does in its MIDI callback.
    midi = service ?? FakeMidiService();
    midi.echoTo = audio;
    container = ProviderContainer(
      overrides: [
        midiServiceProvider.overrideWithValue(midi),
        scoreSourceProvider.overrideWithValue(FakeScoreSource(score)),
        audioServiceProvider.overrideWithValue(audio),
        // A parsed document loads a real timeline (with rests) instead of the
        // demo; used by the "stop at last note" group to get trailing silence.
        if (document != null)
          notationProvider.overrideWith(
            () => _FixedNotation(NotationData(document: document)),
          ),
      ],
    );
    addTearDown(container.dispose);
    // The opened library entry, recorded before the player builds — it is what
    // names a piece whose MusicXML carries no title.
    if (entry != null) {
      container.read(selectedScoreProvider.notifier).select(entry);
    }
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

  group('piece title', () {
    const entry = CatalogEntry(
      id: 'minuet-in-g',
      title: 'Minuet in G (BWV Anh. 114)',
      composer: 'Christian Petzold',
      assetPath: 'assets/scores/intermediate/minuet_in_g.musicxml',
      level: PracticeLevel.intermediate,
    );

    test('the engraved title wins over the library entry', () async {
      await build(document: sampleFourMeasureDocument(), entry: entry);
      expect(read().title, 'FourBars');
    });

    test('falls back to the opened entry when the document has none', () async {
      // Several bundled scores carry no `<work-title>`; without the fallback the
      // header reads "Now Playing: —" for a piece the library had just named.
      await build(document: _untitled(), entry: entry);
      expect(read().title, 'Minuet in G (BWV Anh. 114)');
    });

    test('stays null when neither names the piece', () async {
      await build(document: _untitled());
      expect(read().title, isNull);
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
      // Defaults to auto-fit, sized to the fake score (pitches 60 & 62).
      expect(read().keyboardRange, KeyboardRangeMode.auto);
      final auto = read().keyboardBounds;
      expect(auto.low, lessThanOrEqualTo(60));
      expect(auto.high, greaterThanOrEqualTo(62));

      // Pinning the 88-key preset shows the full piano.
      notifier().setKeyboardRange(KeyboardRangeMode.keys88);
      expect(read().keyboardRange, KeyboardRangeMode.keys88);
      expect(read().keyboardBounds.low, 21);
      expect(read().keyboardBounds.high, 108);
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

  group('pre-start countdown', () {
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

    test(
      'the countdown runs on wall-clock time, not on the transport speed',
      () async {
        await build();
        notifier().toggleWaitMode(); // countdown is a free-run feature

        // A quarter-speed run: the playhead crawls, but the get-ready beat is a
        // real-world beat — 3…2…1…GO must still last kCountdownStartMs of real
        // time, not 4x that (bug: the countdown consumed the speed-scaled delta).
        notifier().setSpeed(0.25);
        notifier().startPlayback();
        expect(read().countdownMs, kCountdownStartMs);

        // Feed exactly the countdown's worth of REAL frame time.
        notifier().advance(kCountdownStartMs);
        expect(read().countdownMs, 0);
        expect(read().elapsedMs, 0); // frozen for the whole countdown

        // ...and the same at double speed: the countdown is never shortened.
        notifier().restartFromTop();
        notifier().setSpeed(2);
        expect(read().countdownMs, kCountdownStartMs);
        notifier().advance(kCountdownStartMs - 1);
        expect(read().countdownMs, 1);
      },
    );

    test(
      'past the countdown, the playhead advances at the transport speed',
      () async {
        await build();
        notifier().toggleWaitMode(); // free run
        notifier().setSpeed(0.5);
        notifier().setPlaying(true); // no countdown via setPlaying
        notifier().advance(400); // 400ms of real time at 0.5x
        expect(read().elapsedMs, 200);
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

    test(
      'restartFromTop resets to the top and replays the countdown',
      () async {
        await build();
        notifier().toggleWaitMode(); // free run
        notifier().setPlaying(true); // (no countdown via setPlaying)
        notifier().advance(300); // move off the top
        expect(read().elapsedMs, 300);
        notifier().restartFromTop();
        expect(read().elapsedMs, 0);
        expect(read().countdownMs, greaterThan(0));
        expect(read().isPlaying, isTrue);
      },
    );

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

  group('furthest playhead (how much of the piece was heard)', () {
    // change: add-post-play-rating-prompt — feeds `playedNoteFraction`, which
    // decides whether the player is asked to rate the piece on the way out.
    test('a never-started player has heard nothing', () async {
      await build();
      expect(read().furthestElapsedMs, read().startMs);
      expect(read().furthestElapsedMs, 0);
    });

    test('tracks the playhead forward', () async {
      await build();
      notifier().toggleWaitMode(); // free run
      notifier().togglePlay();
      notifier().advance(300);
      expect(read().furthestElapsedMs, 300);
    });

    test('pausing keeps the furthest point', () async {
      await build();
      notifier().toggleWaitMode();
      notifier().togglePlay();
      notifier().advance(300);
      notifier().togglePlay(); // pause
      notifier().advance(100); // ignored while paused
      expect(read().furthestElapsedMs, 300);
    });

    test('restarting from the top does not lower it', () async {
      await build();
      notifier().toggleWaitMode();
      notifier().togglePlay();
      notifier().advance(700);
      notifier().restart();
      // The playhead is back at the top, but the piece was still heard to 700 —
      // a restart is not amnesia about what the user already listened to.
      expect(read().elapsedMs, read().startMs);
      expect(read().furthestElapsedMs, 700);
    });

    test('a loop wrap does not lower it', () async {
      await build();
      notifier().toggleWaitMode();
      notifier().togglePlay();
      container.read(performanceScorerProvider.notifier).cancelRun();
      notifier().advance(500);
      notifier().advance(600); // crosses songEndMs (1000) → wraps to 0
      expect(read().elapsedMs, 0);
      expect(read().furthestElapsedMs, 1000);
    });

    test('switching hands keeps it (same piece)', () async {
      await build();
      notifier().toggleWaitMode();
      notifier().togglePlay();
      notifier().advance(400);
      notifier().setSelectedHands(Hand.right);
      expect(read().furthestElapsedMs, 400);
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
      'a tied note held into a chord continuation does not deadlock the gate',
      () async {
        // The prod "Mariage d'Amour" freeze: C5 tied across two onsets, with a
        // fresh E5 chord member at the second one. Holding the tied C5 (as the
        // score demands) while attacking E5 must open the gate — the tie is a
        // single attack, never a re-press.
        final doc = ScoreDocument(
          instruments: const [],
          playOrder: const [],
          meta: const ScoreMeta(title: 'Tied', composer: 'T'),
          staves: 1,
          attributes: const Attributes(
            divisions: 4,
            clefs: [Clef(staff: 1, sign: ClefSign.g, line: 2)],
            keyFifths: 0,
            time: TimeSignature(beats: 4, beatType: 4),
          ),
          measures: [
            NotationMeasure(
              repeats: noRepeats,
              index: 0,
              clefs: const [],
              keyFifths: 0,
              minWidth: 120,
              directions: const [],
              notes: [
                noteEvent(
                  positionDivisions: 0,
                  pitch: const Pitch(step: 'C', octave: 5, alter: 0),
                  tieStart: true,
                ),
                noteEvent(
                  positionDivisions: 4,
                  pitch: const Pitch(step: 'C', octave: 5, alter: 0),
                  tieStop: true,
                ),
                noteEvent(
                  positionDivisions: 4,
                  isChord: true,
                  pitch: const Pitch(step: 'E', octave: 5, alter: 0),
                ),
              ],
            ),
          ],
        );
        await build(document: doc); // Wait Mode on by default
        notifier().togglePlay();
        notifier().advance(50);
        expect(read().blocked, isTrue); // waiting for the C5 attack

        notifier().noteOn(72); // attack the tied C5 at its own onset…
        notifier().advance(1500); // …hold it; clamp to the chord onset (667)
        notifier().advance(50);
        // Only the fresh E5 is awaited there — the held tie is not re-gated.
        expect(read().expectedKeys, {76});

        notifier().noteOn(76);
        notifier().advance(50);
        expect(read().blocked, isFalse);
        expect(read().elapsedMs, greaterThan(667));
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
      // The persisted preferences hold the choice independently of the player.
      expect(container.read(playerPreferencesProvider).metronome, isTrue);

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

  group('start at the first note (trim leading silence)', () {
    // A score whose first note lands in the 3rd measure (80bpm 4/4 →
    // 3000ms/measure): C4 [6000,6500), D4 [6500,7000). Song ends at 7000ms.
    // Effective start = 6000 − kStartLeadInMs(1000) = 5000.
    Score leadingRestScore() => Score(
      bpm: 80,
      measures: [
        Measure(
          index: 0,
          notes: [
            Note(
              pitch: 60,
              startMs: BigInt.from(6000),
              durationMs: BigInt.from(500),
            ),
            Note(
              pitch: 62,
              startMs: BigInt.from(6500),
              durationMs: BigInt.from(500),
            ),
          ],
        ),
      ],
    );
    const start = 5000.0;

    test('initial load seeds the effective start, not zero', () async {
      await build(score: leadingRestScore());
      expect(read().notes.first.startMs, 6000);
      expect(read().startMs, start);
      expect(read().elapsedMs, start);
    });

    test('a piece starting at time zero is unchanged (regression)', () async {
      await build(); // demo opens on a note at 0
      expect(read().startMs, 0);
      expect(read().elapsedMs, 0);
    });

    test('restart returns to the effective start', () async {
      await build(score: leadingRestScore());
      notifier().toggleWaitMode(); // free run so the playhead moves
      notifier().setPlaying(true);
      // Reach the plain advance path (a scored run would finish at the end).
      container.read(performanceScorerProvider.notifier).cancelRun();
      notifier().advance(500); // 5000 → 5500, off the start
      expect(read().elapsedMs, greaterThan(start));
      notifier().restart();
      expect(read().elapsedMs, start);
    });

    test('changing hands recomputes the start for that selection', () async {
      await build(score: leadingRestScore()); // demo notes are right-hand
      notifier().setSelectedHands(Hand.right);
      expect(read().elapsedMs, start);
      // The left hand has no notes here → nothing to await → start falls to 0.
      notifier().setSelectedHands(Hand.left);
      expect(read().startMs, 0);
      expect(read().elapsedMs, 0);
    });

    test('loop wraps to the effective start, not zero', () async {
      await build(score: leadingRestScore());
      notifier().toggleWaitMode(); // free run
      notifier().togglePlay();
      // Reach the unscored loop path (every mode is scored otherwise).
      container.read(performanceScorerProvider.notifier).cancelRun();
      notifier().advance(1000); // 5000 → 6000
      expect(read().elapsedMs, 6000);
      notifier().advance(1500); // 6000 → 7500 ≥ songEnd(7000): loops
      expect(read().elapsedMs, start);
    });

    test('a scored run opens at a non-zero effective start', () async {
      await build(score: leadingRestScore());
      expect(read().elapsedMs, start);
      notifier().togglePlay(); // Synthesia + from the top → scored run
      expect(container.read(performanceScorerProvider).active, isTrue);
    });

    test('free-run countdown arms at a non-zero effective start', () async {
      await build(score: leadingRestScore());
      notifier().toggleWaitMode(); // countdown is a free-run feature
      notifier().startPlayback();
      expect(read().countdownMs, greaterThan(0));
      // The playhead stays frozen at the effective start while it ticks.
      notifier().advance(500);
      expect(read().elapsedMs, start);
      // After the countdown, playback advances past the start.
      notifier().advance(kCountdownStartMs);
      notifier().advance(200);
      expect(read().elapsedMs, greaterThan(start));
    });

    test('resuming mid-piece does not re-trim to the first note', () async {
      await build(score: leadingRestScore());
      notifier().toggleWaitMode(); // free run
      notifier().setPlaying(true);
      notifier().advance(1500); // 5000 → 6500, mid-piece
      expect(read().elapsedMs, 6500);
      notifier().setPlaying(false); // pause
      notifier().startPlayback(); // resume
      expect(read().countdownMs, 0); // no countdown on resume
      expect(read().elapsedMs, 6500); // stays put, no jump to the start
    });

    test('Wait Mode freezes at the first onset after the lead-in', () async {
      await build(score: leadingRestScore()); // Wait Mode on by default
      notifier().togglePlay();
      expect(read().elapsedMs, start); // starts at the lead-in, not 0
      // Advance through the lead-in: the playhead clamps at the first onset…
      notifier().advance(2000); // 5000 → clamps to the 6000 onset
      expect(read().elapsedMs, 6000);
      notifier().advance(50);
      expect(read().blocked, isTrue); // …and freezes there, awaiting C4
      notifier().noteOn(60);
      notifier().advance(50);
      expect(read().blocked, isFalse);
      expect(read().elapsedMs, greaterThan(6000));
    });
  });

  group('stop at the last note (trim trailing silence)', () {
    // A single treble note followed by a trailing rest, so the last note
    // resolves well before songEndMs (which the rest inflates). endMs is the
    // note's resolution; songEndMs runs on to the end of the rest.
    ScoreDocument trailingRestDoc() => ScoreDocument(
      instruments: const [],
      playOrder: const [],
      meta: const ScoreMeta(title: 'Trail', composer: 'T'),
      staves: 1,
      attributes: const Attributes(
        divisions: 4,
        clefs: [Clef(staff: 1, sign: ClefSign.g, line: 2)],
        keyFifths: 0,
        time: TimeSignature(beats: 4, beatType: 4),
      ),
      measures: [
        NotationMeasure(
          repeats: noRepeats,
          index: 0,
          clefs: const [],
          keyFifths: 0,
          minWidth: 120,
          directions: const [],
          notes: [
            noteEvent(
              staff: 1,
              positionDivisions: 0,
              pitch: const Pitch(step: 'C', octave: 4, alter: 0),
              durationDivisions: 4, // quarter note
            ),
            // Three beats of trailing rest — no note, only silence.
            noteEvent(
              staff: 1,
              positionDivisions: 4,
              isRest: true,
              durationDivisions: 12,
              noteType: 'half',
              dots: 1,
            ),
          ],
        ),
      ],
    );

    // A grand-staff document whose right hand (staff 1) plays into a 2nd
    // measure while the left hand (staff 2) stops in the 1st — so each hand
    // has a different last-note resolution.
    ScoreDocument grandStaffDoc() => ScoreDocument(
      instruments: const [],
      playOrder: const [],
      meta: const ScoreMeta(title: 'Grand', composer: 'T'),
      staves: 2,
      attributes: const Attributes(
        divisions: 4,
        clefs: [
          Clef(staff: 1, sign: ClefSign.g, line: 2),
          Clef(staff: 2, sign: ClefSign.f, line: 4),
        ],
        keyFifths: 0,
        time: TimeSignature(beats: 4, beatType: 4),
      ),
      measures: [
        NotationMeasure(
          repeats: noRepeats,
          index: 0,
          clefs: const [],
          keyFifths: 0,
          minWidth: 120,
          directions: const [],
          notes: [
            noteEvent(
              staff: 1,
              positionDivisions: 0,
              pitch: const Pitch(step: 'C', octave: 5, alter: 0),
            ),
            noteEvent(
              staff: 2,
              positionDivisions: 0,
              pitch: const Pitch(step: 'C', octave: 3, alter: 0),
              durationDivisions: 16, // left hand's last note (whole)
              noteType: 'whole',
            ),
          ],
        ),
        NotationMeasure(
          repeats: noRepeats,
          index: 1,
          clefs: const [],
          keyFifths: 0,
          minWidth: 120,
          directions: const [],
          notes: [
            noteEvent(
              staff: 1,
              positionDivisions: 0,
              pitch: const Pitch(
                step: 'D',
                octave: 5,
                alter: 0,
              ), // right hand plays on
            ),
          ],
        ),
      ],
    );

    test(
      'a scored run finishes at the last note, not the trailing rest',
      () async {
        await build(document: trailingRestDoc());
        notifier().toggleWaitMode(); // free run so the playhead advances freely
        notifier().togglePlay(); // Synthesia from the top → scored run
        expect(container.read(performanceScorerProvider).active, isTrue);
        final end = read().endMs;
        expect(end, lessThan(read().songEndMs)); // there IS trailing silence
        // Advance just past the last note but before songEndMs.
        notifier().advance(end + 1);
        expect(read().isPlaying, isFalse); // the run finished…
        expect(read().elapsedMs, end); // …clamped to endMs, not songEndMs
      },
    );

    test('an unscored run loops at the last note back to the start', () async {
      await build(document: trailingRestDoc());
      notifier().toggleWaitMode(); // free run
      notifier().togglePlay();
      // Reach the unscored loop path (every mode is scored otherwise).
      container.read(performanceScorerProvider.notifier).cancelRun();
      final end = read().endMs;
      final start = read().startMs;
      expect(end, lessThan(read().songEndMs));
      // Advance just past the last note but before songEndMs: already loops.
      notifier().advance(end + 1);
      expect(read().elapsedMs, start); // wrapped to the trimmed start, not held
    });

    test('Wait Mode completes the run at the last note', () async {
      await build(document: trailingRestDoc()); // Wait Mode on by default
      notifier().togglePlay(); // scored run
      notifier().advance(50);
      expect(read().blocked, isTrue); // frozen on the only onset (C4 at 0)
      notifier().noteOn(60); // hit the last note
      final end = read().endMs;
      // Advance past the last note; the trailing rest is not an onset to gate.
      notifier().advance(end + 1);
      expect(read().isPlaying, isFalse); // completed at the last note…
      expect(read().elapsedMs, end); // …without gating through the rest
    });

    test('changing hands recomputes the effective end for that hand', () async {
      await build(document: grandStaffDoc());
      final bothEnd = read().copyWith(selectedHands: Hand.both).endMs;
      notifier().setSelectedHands(Hand.left);
      final leftEnd = read().endMs;
      notifier().setSelectedHands(Hand.right);
      final rightEnd = read().endMs;
      // The right hand plays into the 2nd measure; the left hand stops in the
      // 1st, so its end is earlier and both-hands equals the later (right) end.
      expect(leftEnd, lessThan(rightEnd));
      expect(bothEnd, rightEnd);
    });
  });
}
