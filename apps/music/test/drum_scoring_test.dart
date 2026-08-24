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
import 'package:music/services/clock_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/notation_data.dart';
import 'package:music/state/notation_notifier.dart';
import 'package:music/state/performance_scoring.dart';
import 'package:music/state/performance_scoring_core.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/session_summary.dart';

import 'support/fakes.dart';
import 'support/notation_fakes.dart';
import 'support/prefs_fakes.dart';

// Percussion judgment over the player's own path (change: add-drum-scoring):
// strokes bind at the kit piece's grain, the hi-hat articulation shades the
// verdict, there is no sustain dimension, and the Wait Mode gate releases on
// exactly the strokes the scorer binds.
//
// The fixture is `sampleOpenGrooveDocument` at 100 BPM / 4 divisions per
// quarter, so a quarter is 600 ms and an eighth 300 ms:
//   beat 1 (0 ms)     crash 49 + closed hat 42 (hands), kick 36 (feet)
//   0.5  (300 ms)     hat 42
//   beat 2 (600 ms)   hat 42 + snare 38
//   …
//   3.5  (2100 ms)    OPEN hat 46 — the run's articulation case
// Measure 2 opens at 2400 ms.

class _FixedNotation extends Notation {
  _FixedNotation(this._value);
  final NotationData _value;
  @override
  NotationData build() => _value;
}

/// Deterministic clock, so a Wait-Mode reaction time is a number the test set.
class _FakeClock implements Clock {
  int now = 0;
  @override
  int nowMs() => now;
}

Future<void> _flush() => Future<void>.delayed(Duration.zero);

void main() {
  late ProviderContainer container;
  late _FakeClock clock;

  PlayerData data() => container.read(playerProvider);
  Player player() => container.read(playerProvider.notifier);
  ScoringData scoring() => container.read(performanceScorerProvider);

  Future<void> build() async {
    clock = _FakeClock();
    container = ProviderContainer(
      overrides: [
        midiServiceProvider.overrideWithValue(FakeMidiService()),
        scoreSourceProvider.overrideWithValue(FakeScoreSource()),
        audioServiceProvider.overrideWithValue(RecordingAudioService()),
        preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
        clockProvider.overrideWithValue(clock),
        notationProvider.overrideWith(
          () => _FixedNotation(
            NotationData(document: sampleOpenGrooveDocument()),
          ),
        ),
      ],
    );
    addTearDown(container.dispose);
    container.listen(playerProvider, (_, _) {}, fireImmediately: true);
    container.listen(
      performanceScorerProvider,
      (_, _) {},
      fireImmediately: true,
    );
    await _flush();
    expect(data().isPercussion, isTrue);
  }

  /// A free-run scored run, playhead at the piece's start.
  Future<void> freeRun() async {
    await build();
    player()
      ..toggleWaitMode()
      ..setPlaying(true);
    expect(scoring().active, isTrue);
  }

  /// The verdict recorded for the note at [startMs] in [result].
  TimingVerdict verdictAt(SessionResult result, int startMs, int pitch) =>
      result.notes
          .firstWhere(
            (j) => !j.wrong && j.startMs == startMs && j.pitch == pitch,
          )
          .verdict;

  /// Plays the piece out so the run finalizes, and returns the record.
  SessionResult finish() {
    for (var i = 0; i < 120 && scoring().active; i++) {
      player().advance(100);
    }
    final result = scoring().lastResult;
    expect(result, isNotNull, reason: 'the run never finalized');
    return result!;
  }

  group('binding at the kit piece grain', () {
    test(
      'an electric snare (40) satisfies the written acoustic snare (38)',
      () async {
        await freeRun();
        player().advance(600); // the snare onset
        expect(data().onsetPitchesAt(data().elapsedMs), contains(38));
        // The score never writes a 40 anywhere.
        expect(data().notes.any((n) => n.pitch == 40), isFalse);
        player().noteOn(40);

        final result = finish();
        expect(verdictAt(result, 600, 38), TimingVerdict.perfect);
        expect(result.wrongNotes, 0);
      },
    );

    test('a side stick (37) satisfies it too, and a tom does not', () async {
      await freeRun();
      player()
        ..advance(600)
        ..noteOn(37);
      expect(scoring().combo, 1);
      // A high tom belongs to no lane of this groove: nobody asked for it.
      player().noteOn(47);
      expect(scoring().combo, 0);
      final result = finish();
      expect(verdictAt(result, 600, 38), TimingVerdict.perfect);
      expect(result.wrongNotes, 1);
    });

    test('either kick number drives the same pedal', () async {
      await freeRun();
      // The score writes its kick as 36; the kit sends 35.
      expect(data().notes.any((n) => n.pitch == 35), isFalse);
      player().noteOn(35); // the kick onset at 0 ms
      final result = finish();
      expect(verdictAt(result, 0, 36), TimingVerdict.perfect);
    });

    test('a china (52) does not satisfy the written crash (49)', () async {
      await freeRun();
      player().noteOn(52); // at the crash's own onset
      final result = finish();
      expect(result.wrongNotes, 1);
      expect(verdictAt(result, 0, 49), TimingVerdict.missed);
    });
  });

  group('the hi-hat articulation shades, never gates', () {
    test(
      'a closed stroke binds to a written open hi-hat, capped below perfect',
      () async {
        await freeRun();
        player()
          ..advance(2100) // the open hi-hat
          ..noteOn(42); // a kit with no hi-hat controller can only send this
        final result = finish();
        final verdict = verdictAt(result, 2100, 46);
        expect(verdict, isNot(TimingVerdict.missed)); // it bound
        expect(verdict, isNot(TimingVerdict.perfect)); // and it cost something
        expect(verdict, TimingVerdict.good);
        expect(result.wrongNotes, 0);
      },
    );

    test('the written articulation is judged on timing alone', () async {
      await freeRun();
      player()
        ..advance(2100)
        ..noteOn(46);
      expect(verdictAt(finish(), 2100, 46), TimingVerdict.perfect);
    });

    test('the cap never lifts a worse verdict', () async {
      await freeRun();
      player()
        ..advance(2100 + 120) // past the `good` window: a late stroke
        ..noteOn(42);
      // Capping is one-directional — it lowers a `perfect`, it never raises
      // anything toward it.
      expect(verdictAt(finish(), 2100, 46), TimingVerdict.late);
    });

    test(
      'an open stroke on a written closed hat is capped the same way',
      () async {
        await freeRun();
        player()
          ..advance(600) // hat 42 + snare 38 share this onset
          ..noteOn(46);
        final result = finish();
        expect(verdictAt(result, 600, 42), TimingVerdict.good);
        expect(result.wrongNotes, 0);
      },
    );
  });

  group('a percussion run has no sustain dimension', () {
    test('the record carries no aggregate and no per-stroke ratio', () async {
      await freeRun();
      player()
        ..advance(600)
        ..noteOn(38)
        ..noteOff(38); // an e-kit releases within milliseconds
      final result = finish();
      expect(result.sustain, isNull);
      expect(result.notes.every((j) => j.sustainRatio == null), isTrue);
      // The instant release cost the stroke nothing.
      expect(verdictAt(result, 600, 38), TimingVerdict.perfect);
    });

    test('the live percentage blends two dimensions', () async {
      await freeRun();
      // Land the whole first onset (crash + hat + kick) dead on.
      player()
        ..noteOn(49)
        ..noteOn(42)
        ..noteOn(36);
      // Three perfect strokes, nothing wrong: the blend saturates without any
      // sustain contribution at all.
      expect(scoring().syncPercent, closeTo(100, 1e-9));
    });

    test('a do-nothing run scores 0', () async {
      await freeRun();
      final result = finish(); // never touched a pad
      expect(result.overallSyncPct, 0);
      expect(result.sustain, isNull);
      expect(result.timing, 0);
      expect(result.correctness, 0);
    });
  });

  group('extra strokes lower correctness and never freeze playback', () {
    test('a stroke nobody asked for is recorded and play continues', () async {
      await freeRun();
      player().advance(450); // between two onsets
      final before = data().elapsedMs;
      player().noteOn(47); // a tom this groove does not contain
      expect(scoring().syncPercent, lessThan(100));
      player().advance(100);
      expect(data().isPlaying, isTrue);
      expect(data().elapsedMs, greaterThan(before));
      expect(finish().wrongNotes, 1);
    });

    test(
      'a stroke of a required piece inside the window credits its onset',
      () async {
        await freeRun();
        player()
          ..advance(600 - 30) // inside the perfect window
          ..noteOn(38);
        final result = finish();
        expect(result.wrongNotes, 0);
        expect(verdictAt(result, 600, 38), TimingVerdict.perfect);
      },
    );
  });

  group('run activation over the percussion path', () {
    test('a full run in the cascade arms the scorer', () async {
      await build();
      expect(scoring().active, isFalse);
      player()
        ..toggleWaitMode()
        ..setPlaying(true);
      expect(scoring().active, isTrue);
    });

    test('a selective run never arms it — unscored by construction', () async {
      await build();
      player()
        ..toggleWaitMode()
        ..setPracticeRange(0, 0); // one of the two written measures
      expect(data().isSelectiveRun, isTrue);
      player().setPlaying(true);
      expect(scoring().active, isFalse);
      for (var i = 0; i < 40; i++) {
        player()
          ..advance(100)
          ..noteOn(38);
      }
      expect(scoring().active, isFalse);
      expect(scoring().lastResult, isNull);
      expect(scoring().recentHits, isEmpty);
    });

    test('the mode stamping and classification are unchanged', () async {
      await build();
      // Wait Mode on for the first onset, then off for the rest: the run is
      // `mixed` and carries both sub-scores, exactly as a keyboard run does.
      player().setPlaying(true);
      player()
        ..advance(16)
        ..noteOn(49)
        ..noteOn(42)
        ..noteOn(36);
      player().toggleWaitMode();
      player()
        ..advance(600)
        ..noteOn(38);
      final result = finish();
      expect(result.runMode, RunMode.mixed);
      expect(result.waitOnsetCount, greaterThan(0));
      expect(result.freeOnsetCount, greaterThan(0));
      expect(result.waitSyncPct, isNotNull);
      expect(result.freeSyncPct, isNotNull);
      expect(result.avgReactionMs, isNotNull);
    });
  });

  group('hands / feet scoping', () {
    test(
      'a feet-only run judges only foot events and records the selection',
      () async {
        await build();
        player()
          ..toggleWaitMode()
          ..setSelectedHands(Hand.left) // the feet
          ..setPlaying(true);
        expect(scoring().active, isTrue);
        // Only the kick is in the judged set.
        expect(data().visibleNotes.every((n) => n.pitch == 36), isTrue);
        player().noteOn(36); // the kick at 0 ms
        player().noteOn(38); // a hand stroke: nothing in this run asks for it
        final result = finish();
        expect(result.hands, 'feet');
        expect(result.wrongNotes, 1);
        expect(result.notes.every((j) => j.wrong || j.pitch == 36), isTrue);
      },
    );

    test('a hands-only run records the hands reading', () async {
      await build();
      player()
        ..toggleWaitMode()
        ..setSelectedHands(Hand.right)
        ..setPlaying(true);
      expect(data().visibleNotes.any((n) => n.pitch == 36), isFalse);
      expect(finish().hands, 'hands');
    });

    test('both records the hands-and-feet reading', () async {
      await freeRun();
      expect(finish().hands, 'handsAndFeet');
    });
  });

  group('the gate releases exactly when the scorer binds', () {
    // The drift the one-identity requirement forbids: a gate that releases on
    // a stroke the scorer then counts as wrong, or the reverse.
    Future<void> expectAgreement(int gm, {required bool binds}) async {
      await build(); // Wait Mode is on by default
      player().setPlaying(true);
      player().advance(16); // reach the first onset and freeze there
      expect(data().blocked, isTrue);
      final onset = data().onsetPitchesAt(data().elapsedMs);
      expect(onset, isNotEmpty);

      final wrongBefore = scoring().recentHits.where((h) => h.wrong).length;
      player().noteOn(gm);
      final released = data().gateSatisfied.isNotEmpty;
      final bound =
          scoring().recentHits.where((h) => h.wrong).length == wrongBefore &&
          scoring().recentHits.isNotEmpty;

      expect(released, binds, reason: 'gate disagreed on $gm');
      expect(bound, binds, reason: 'scorer disagreed on $gm');
    }

    test('a required piece releases and binds', () async {
      await expectAgreement(42, binds: true); // the closed hat at 0 ms
    });

    test('an equivalent number of a required piece does both too', () async {
      await expectAgreement(35, binds: true); // the kick, written as 36
    });

    test('a piece the onset does not require does neither', () async {
      await expectAgreement(47, binds: false); // a tom
      await expectAgreement(52, binds: false); // the china, not the crash
    });
  });
}
