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
import 'package:music/services/preferences_service.dart';
import 'package:music/src/rust/api/score.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/player_preferences.dart';

import '../support/fakes.dart';
import '../support/prefs_fakes.dart';

/// The written-score mute (change: add-practice-focus-controls): the app stops
/// sounding the notes it is *asking for*, and nothing else about the session
/// changes.
///
/// The tests come in pairs on purpose. Half of them assert what goes quiet; the
/// other half assert what must not — because a mute that also stopped the gate,
/// the scorer or the playhead would be a different feature, and a silent score
/// looks identical to a stalled one from the outside.
void main() {
  late RecordingAudioService audio;
  late FakeMidiService midi;
  late FakePreferencesService prefs;
  late ProviderContainer container;

  Player notifier() => container.read(playerProvider.notifier);
  PlayerData state() => container.read(playerProvider);

  /// Runs the transport until [ms] have elapsed, in frame-sized steps.
  ///
  /// Bounded: the Wait Mode gate holds the playhead indefinitely on purpose, and
  /// an unbounded loop here would hang the suite rather than fail a test.
  void advanceTo(double ms) {
    for (var i = 0; i < 2000; i++) {
      if (container.read(playerProvider).elapsedMs >= ms) return;
      notifier().advance(16);
    }
  }

  /// Free-run playback: Wait Mode is the app default and freezes the playhead at
  /// every onset, which is a different feature's behaviour. The mute is about
  /// what a running transport sounds, so these tests run it freely — except the
  /// one that is specifically about the gate.
  void playFreely() {
    if (container.read(playerProvider).waitMode) notifier().toggleWaitMode();
    notifier().setPlaying(true);
  }

  /// A score long enough to run a transport through. The shared fake's default
  /// ends at 1000 ms, which is shorter than the passages these tests play.
  Score longScore() => Score(
    bpm: 120,
    measures: [
      for (var m = 0; m < 8; m++)
        Measure(
          index: m,
          notes: [
            for (var b = 0; b < 4; b++)
              Note(
                pitch: 60 + b,
                startMs: BigInt.from(m * 2000 + b * 500),
                durationMs: BigInt.from(400),
              ),
          ],
        ),
    ],
  );

  Future<ProviderContainer> make({FakePreferencesService? store}) async {
    prefs = store ?? FakePreferencesService();
    audio = RecordingAudioService();
    midi = FakeMidiService(ports: const ['Piano'], connected: 'Piano');
    final c = ProviderContainer(
      overrides: [
        midiServiceProvider.overrideWithValue(midi),
        scoreSourceProvider.overrideWithValue(FakeScoreSource(longScore())),
        audioServiceProvider.overrideWithValue(audio),
        preferencesServiceProvider.overrideWithValue(prefs),
      ],
    );
    addTearDown(c.dispose);
    addTearDown(midi.close);
    // The player seeds itself from the preferences synchronously in `build`, so
    // storage has to have been read BEFORE it is first watched — otherwise a
    // persisted choice is silently replaced by the default.
    await c.read(playerPreferencesProvider.notifier).restored;
    c.listen(playerProvider, (_, _) {}, fireImmediately: true);
    return c;
  }

  Future<void> settle() async {
    await Future<void>.delayed(Duration.zero);
    await Future<void>.delayed(Duration.zero);
  }

  setUp(() async {
    container = await make();
    await settle();
  });

  group('what goes quiet', () {
    test('off by default — the score sounds as it always has', () {
      expect(state().scoreAudioMuted, isFalse);
      playFreely();
      advanceTo(2000);
      expect(
        audio.noteOns,
        isNotEmpty,
        reason: 'the baseline this whole feature is measured against',
      );
    });

    test('muted, the schedule reaches the synth with nothing', () {
      notifier().setScoreAudioMuted(muted: true);
      audio.calls.clear();

      playFreely();
      advanceTo(2000);

      expect(audio.noteOns, isEmpty);
      expect(audio.noteOffs, isEmpty);
    });

    test('the player\'s own notes still sound', () {
      // The two mutes are independent: this one is about the notes the app
      // asks for, never the ones the player plays.
      notifier().setScoreAudioMuted(muted: true);
      audio.calls.clear();

      notifier().noteOn(64, source: NoteSource.onScreen);
      notifier().noteOff(64, source: NoteSource.onScreen);

      expect(audio.noteOns.map((n) => n.pitch), [64]);
      expect(audio.noteOffs, [64]);
    });

    test('the metronome still clicks', () {
      // Often the whole point of silencing the part: hearing the click.
      if (!state().metronomeEnabled) notifier().toggleMetronome();
      notifier().setScoreAudioMuted(muted: true);
      audio.metronomeClicks.clear();

      playFreely();
      advanceTo(3000);

      expect(audio.metronomeClicks, isNotEmpty);
    });
  });

  group('what does not change', () {
    test('the playhead still advances', () {
      notifier().setScoreAudioMuted(muted: true);
      playFreely();
      advanceTo(1500);
      expect(state().elapsedMs, greaterThanOrEqualTo(1500));
    });

    test('the score is still drawn', () {
      notifier().setScoreAudioMuted(muted: true);
      expect(state().visibleNotes, isNotEmpty);
    });

    test('the Wait Mode gate still holds and releases', () {
      // The gate is the reason a mute cannot be implemented by simply not
      // scheduling: a muted session must still WAIT for the note it is not
      // playing.
      notifier().setScoreAudioMuted(muted: true);
      if (!state().waitMode) notifier().toggleWaitMode();
      notifier().setPlaying(true);
      advanceTo(1);

      final blockedAt = state().elapsedMs;
      for (var i = 0; i < 60; i++) {
        notifier().advance(16);
      }
      final expected = state().expectedKeys;
      expect(expected, isNotEmpty, reason: 'the gate is holding on something');
      expect(
        state().elapsedMs,
        blockedAt,
        reason: 'and the playhead is held there, muted or not',
      );

      for (final p in expected) {
        notifier().noteOn(p, source: NoteSource.midiDevice);
      }
      notifier().advance(16);
      expect(state().elapsedMs, greaterThan(blockedAt));
    });
  });

  group('toggling mid-playback', () {
    test('muting releases what is sounding, so no voice hangs', () {
      playFreely();
      advanceTo(1200);
      expect(audio.noteOns, isNotEmpty);
      final offsBefore = audio.allNotesOffCount;

      notifier().setScoreAudioMuted(muted: true);

      expect(
        audio.allNotesOffCount,
        offsBefore + 1,
        reason:
            'a note the schedule started is owed a release, and the mute '
            'would otherwise suppress the very call that delivers it',
      );
    });

    test('unmuting sounds again, and releases nothing it never started', () {
      notifier().setScoreAudioMuted(muted: true);
      playFreely();
      advanceTo(1500);
      expect(audio.noteOns, isEmpty);
      expect(
        audio.noteOffs,
        isEmpty,
        reason: 'no release is owed for a voice that was never started',
      );

      notifier().setScoreAudioMuted(muted: false);
      final before = audio.noteOns.length;
      advanceTo(state().elapsedMs + 2000);
      expect(audio.noteOns.length, greaterThan(before));
    });

    test('setting it to what it already is does nothing', () {
      final offsBefore = audio.allNotesOffCount;
      notifier().setScoreAudioMuted(muted: false);
      expect(audio.allNotesOffCount, offsBefore);
    });
  });

  group('persistence', () {
    test('the choice is remembered across a relaunch', () async {
      notifier().setScoreAudioMuted(muted: true);
      await settle();

      // Same device storage, fresh app.
      container = await make(store: prefs);
      await settle();

      expect(state().scoreAudioMuted, isTrue);
    });
  });
}
