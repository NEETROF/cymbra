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
import 'package:music/state/performance_scoring.dart';
import 'package:music/state/performance_scoring_core.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';

import '../support/fakes.dart';

/// The output offset (change: add-audio-output-routing) shifts the *reference*
/// the player is judged against — the position they are actually hearing — and
/// the playhead drawn for them, from one number. These tests pin both ends: 0
/// must reproduce today's verdicts exactly, and a real offset must stop
/// penalizing someone who plays in time with delayed sound.
void main() {
  /// Plays the demo score's first onset (C4 at 0 ms) [attackAtMs] after the
  /// playhead left the top, with the given output offset, in free run.
  /// Returns the verdict the scorer gave the attack.
  Future<TimingVerdict> verdictFor({
    required int offsetMs,
    required double attackAtMs,
    double speed = 1.0,
  }) async {
    final midi = FakeMidiService(ports: const ['Piano'], connected: 'Piano');
    final container = ProviderContainer(
      overrides: [
        midiServiceProvider.overrideWithValue(midi),
        scoreSourceProvider.overrideWithValue(FakeScoreSource()),
        audioServiceProvider.overrideWithValue(RecordingAudioService()),
      ],
    );
    addTearDown(container.dispose);
    addTearDown(midi.close);
    container.listen(playerProvider, (_, _) {}, fireImmediately: true);
    await Future<void>.delayed(Duration.zero);
    await Future<void>.delayed(Duration.zero);

    final notifier = container.read(playerProvider.notifier)
      ..setOutputOffsetMs(offsetMs)
      ..setSpeed(speed)
      ..toggleWaitMode() // free run: judged by offset from the onset
      ..togglePlay();

    // Walk the playhead (the *emission* clock) up to the attack instant.
    while (container.read(playerProvider).elapsedMs < attackAtMs) {
      notifier.advance(10);
    }
    notifier.noteOn(60, source: NoteSource.onScreen);

    final hits = container.read(performanceScorerProvider).recentHits;
    expect(hits, isNotEmpty, reason: 'the attack should have been judged');
    return hits.last.verdict;
  }

  test('offset 0 leaves an on-time attack perfect', () async {
    expect(await verdictFor(offsetMs: 0, attackAtMs: 0), TimingVerdict.perfect);
  });

  test('offset 0 still judges a late attack late', () async {
    // 120 ms past the onset is outside the ±90 ms "good" window.
    expect(await verdictFor(offsetMs: 0, attackAtMs: 120), TimingVerdict.late);
  });

  test(
    'a delayed route does not penalize a player following the sound',
    () async {
      // The route is 120 ms late, so the sound of the 0 ms onset reaches the ear
      // when the emission clock reads 120. Attacking there is *on time*.
      expect(
        await verdictFor(offsetMs: 120, attackAtMs: 120),
        TimingVerdict.perfect,
      );
    },
  );

  test('the offset does not make an early attack pass', () async {
    // Attacking at the emission instant on a 120 ms route means playing 120 ms
    // before the note is heard — still bound to the onset, and judged early.
    // Asserting the verdict exactly (rather than `isNot(perfect)`) is what
    // separates "bound and judged early" from "rejected as an extra note",
    // which also lands in recentHits, as `missed` + `wrong`.
    expect(await verdictFor(offsetMs: 120, attackAtMs: 0), TimingVerdict.early);
  });

  group('the playhead and the reference come from one number', () {
    test('offset 0 makes the reference the playhead itself', () {
      const data = PlayerData(elapsedMs: 640);
      expect(data.referenceMs, data.elapsedMs);
    });

    test('at 1x a non-zero offset shifts the reference back by exactly it', () {
      const data = PlayerData(elapsedMs: 640, outputOffsetMs: 200);
      expect(data.referenceMs, 440);
    });

    test('Wait Mode judges on the emission clock, free run on the heard one', () {
      const waiting = PlayerData(
        elapsedMs: 640,
        outputOffsetMs: 200,
        waitMode: true,
      );
      // Nothing has sounded during a freeze: the gate is identified by where the
      // playhead actually is, or it matches no onset at all.
      expect(waiting.judgmentClockMs, 640);

      // (Wait Mode is the PlayerData default, so free run is the explicit one.)
      const free = PlayerData(
        elapsedMs: 640,
        outputOffsetMs: 200,
        waitMode: false,
      );
      expect(free.judgmentClockMs, 440);
    });

    test('neither judgment clock moves when the tempo does', () {
      // The clock the scorer measures durations against must not be a function
      // of a control the player can tap mid-run: a jump would sweep pending
      // onsets into `missed` and clamp sustains to zero. Pinned in both modes.
      for (final speed in const [0.25, 1.0, 2.0]) {
        expect(
          PlayerData(
            elapsedMs: 640,
            outputOffsetMs: 200,
            speed: speed,
          ).judgmentClockMs,
          640,
          reason: 'Wait Mode at ${speed}x',
        );
        expect(
          PlayerData(
            elapsedMs: 640,
            outputOffsetMs: 200,
            speed: speed,
            waitMode: false,
          ).judgmentClockMs,
          440,
          reason: 'free run at ${speed}x',
        );
      }
    });
  });

  test('a mid-run tempo tap does not sweep pending onsets into missed', () async {
    // Regression guard. Making the judgment clock `elapsed - offset * speed`
    // fixes the wall-clock/score-clock unit error but makes the clock depend on
    // a control the player can tap at any time: at offset 200, dropping 1x to
    // 0.25x teleports it forward by 150 ms against a 160 ms bind window, and the
    // very next frame marks every pending onset missed. Whatever eventually
    // fixes the unit error must keep this test green.
    final midi = FakeMidiService(ports: const ['Piano'], connected: 'Piano');
    final container = ProviderContainer(
      overrides: [
        midiServiceProvider.overrideWithValue(midi),
        scoreSourceProvider.overrideWithValue(FakeScoreSource()),
        audioServiceProvider.overrideWithValue(RecordingAudioService()),
      ],
    );
    addTearDown(container.dispose);
    addTearDown(midi.close);
    container.listen(playerProvider, (_, _) {}, fireImmediately: true);
    await Future<void>.delayed(Duration.zero);
    await Future<void>.delayed(Duration.zero);

    final notifier = container.read(playerProvider.notifier)
      ..setOutputOffsetMs(200)
      ..toggleWaitMode() // free run: this is where the miss sweep runs
      ..togglePlay();

    // Sit just inside the bind window of the 0 ms onset on the heard clock.
    while (container.read(playerProvider).elapsedMs < 220) {
      notifier.advance(10);
    }
    expect(container.read(performanceScorerProvider).recentHits, isEmpty);

    notifier
      ..setSpeed(0.25)
      ..advance(10);

    expect(
      container.read(performanceScorerProvider).recentHits,
      isEmpty,
      reason: 'the tempo tap must not resolve an onset the player still owns',
    );
  });

  test('Wait Mode credits an awaited press on a delayed route', () async {
    final midi = FakeMidiService(ports: const ['Piano'], connected: 'Piano');
    final container = ProviderContainer(
      overrides: [
        midiServiceProvider.overrideWithValue(midi),
        scoreSourceProvider.overrideWithValue(FakeScoreSource()),
        audioServiceProvider.overrideWithValue(RecordingAudioService()),
      ],
    );
    addTearDown(container.dispose);
    addTearDown(midi.close);
    container.listen(playerProvider, (_, _) {}, fireImmediately: true);
    await Future<void>.delayed(Duration.zero);
    await Future<void>.delayed(Duration.zero);

    final notifier = container.read(playerProvider.notifier)
      ..setOutputOffsetMs(200)
      ..setSpeed(0.25) // the gate must not care about the tempo either
      ..togglePlay();
    expect(container.read(playerProvider).waitMode, isTrue);

    // The gate holds on the emission clock, and so does the scorer in Wait
    // Mode: the frozen playhead has to match its own onset to the millisecond.
    notifier.advance(16);
    expect(container.read(playerProvider).blocked, isTrue);

    final onset = container
        .read(playerProvider)
        .onsetPitchesAt(container.read(playerProvider).elapsedMs);
    for (final p in onset) {
      notifier.noteOn(p, source: NoteSource.midiDevice);
    }
    notifier.advance(16);

    expect(container.read(playerProvider).blocked, isFalse);
    // The reaction is measured on the wall clock, so the press is credited,
    // not marked missed.
    final hits = container.read(performanceScorerProvider).recentHits;
    // Assert the press was recorded AT ALL before asking what it scored: this
    // used to read `isNot(contains(missed))` on an empty list, which passes for
    // free — and it was empty, because the shifted clock matched no onset.
    expect(hits, isNotEmpty, reason: 'the awaited press must be judged');
    expect(hits.map((h) => h.verdict), isNot(contains(TimingVerdict.missed)));
  });
}
