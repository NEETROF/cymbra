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
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';

import '../support/fakes.dart';

/// Read-only preview mode (change: add-app-score-rating, task 4.3): the render +
/// audio engine plays the score, but NO scoring/performance events fire and user
/// input is ignored.
void main() {
  late FakeMidiService midi;
  late RecordingAudioService audio;
  late ProviderContainer container;

  Player notifier() => container.read(playerProvider.notifier);
  PlayerData read() => container.read(playerProvider);
  PerformanceScorer scorer() =>
      container.read(performanceScorerProvider.notifier);

  Future<void> flush() => Future<void>.delayed(Duration.zero);

  Future<void> build() async {
    midi = FakeMidiService();
    audio = RecordingAudioService();
    container = ProviderContainer(
      overrides: [
        midiServiceProvider.overrideWithValue(midi),
        scoreSourceProvider.overrideWithValue(FakeScoreSource(null)),
        audioServiceProvider.overrideWithValue(audio),
      ],
    );
    addTearDown(container.dispose);
    container.listen(playerProvider, (_, _) {}, fireImmediately: true);
    await flush(); // let the demo score load
  }

  tearDown(() async => midi.close());

  test('preview never starts a scored run and produces no scoring', () async {
    await build();
    notifier().setPreview(true);
    notifier().setPlaying(true);
    // No scored run is armed in preview.
    expect(container.read(performanceScorerProvider).active, isFalse);
    // Play all the way through the demo score (songEndMs = 1000).
    for (var i = 0; i < 20; i++) {
      notifier().advance(100);
    }
    // Still no run, and no result was ever produced.
    expect(container.read(performanceScorerProvider).active, isFalse);
    expect(container.read(performanceScorerProvider).lastResult, isNull);
    // The preview plays through once and stops at the end (no loop).
    expect(read().isPlaying, isFalse);
    // Playback actually sounded the score's notes (it is a real preview).
    expect(audio.noteOns, isNotEmpty);
  });

  test(
    'preview ignores user input (no judging, no sound from a press)',
    () async {
      await build();
      notifier().setPreview(true);
      final before = audio.noteOns.length;
      // A press, a MIDI event, and a release are all ignored in preview.
      notifier().noteOn(60);
      midi.emit(noteOnEvent(64));
      await flush();
      notifier().noteOff(60);
      expect(read().activeNotes, isEmpty); // input never registered
      expect(audio.noteOns.length, before); // and never sounded
      expect(container.read(performanceScorerProvider).active, isFalse);
    },
  );

  test('preview forces wait mode off so playback is not frozen', () async {
    await build();
    // The demo defaults to wait mode on; preview must clear it.
    expect(read().waitMode, isTrue);
    notifier().setPreview(true);
    expect(read().waitMode, isFalse);
    // toggleWaitMode is inert while previewing.
    notifier().toggleWaitMode();
    expect(read().waitMode, isFalse);
  });

  test('leaving preview restores normal scored playback', () async {
    await build();
    notifier().setPreview(true);
    notifier().setPreview(false);
    expect(read().preview, isFalse);
    // A normal play from the top now opens a scored run again.
    notifier().setPlaying(true);
    expect(container.read(performanceScorerProvider).active, isTrue);
    // Sanity: the scorer is the real one, reset cleanly.
    scorer().cancelRun();
  });
}
