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
import 'package:mockito/mockito.dart';
import 'package:music/services/audio_capture_service.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/connectivity_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/src/rust/api/midi.dart' show MidiEvent, MidiEventKind;
import 'package:music/services/play_sync_service.dart';
import 'package:music/state/acoustic_input_access.dart';
import 'package:music/state/notation_data.dart';
import 'package:music/state/notation_notifier.dart';
import 'package:music/state/play_sync_notifier.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_input_source.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/session_notifier.dart';

import '../support/fakes.dart';
import '../support/notation_fakes.dart';
import '../support/prefs_fakes.dart';
import 'input_calibration_notifier_test.mocks.dart';
import 'practice_range_test.mocks.dart';

/// The acoustic capture session (change: add-acoustic-piano-input): bound to
/// the player with the microphone source and a keyboard score, never
/// synthesizing live notes, and never touching the remembered MIDI state.
class _SilentConnectivity implements ConnectivityService {
  @override
  Stream<void> get onOnline => const Stream<void>.empty();
  @override
  Stream<bool> get onlineStatus => const Stream.empty();
  @override
  Future<bool> isOnline() async => true;
  @override
  Future<bool> isDefinitelyOffline() async => false;
}

class _NoopRetryScheduler implements PlayRetryScheduler {
  @override
  void schedule(Duration delay, void Function() action) {}
  @override
  void cancel() {}
}

class _FixedNotation extends Notation {
  _FixedNotation(this._data);
  final NotationData _data;
  @override
  NotationData build() => _data;
}

void main() {
  TestWidgetsFlutterBinding.ensureInitialized();

  late MockAudioCaptureService capture;
  late FakePreferencesService prefs;

  late FakeMidiService midi;

  ProviderContainer harness() {
    capture = MockAudioCaptureService();
    prefs = FakePreferencesService();
    midi = FakeMidiService();
    when(
      capture.routeChanges(),
    ).thenAnswer((_) => const Stream<CaptureRoute?>.empty());
    when(capture.beginCapture()).thenAnswer((_) async => true);
    when(capture.endCapture()).thenAnswer((_) async {});
    final sync = MockPlaySyncService();
    when(sync.recordPractice(any)).thenAnswer((_) async => 0);
    when(sync.recordSession(any)).thenAnswer((_) async => 0);
    final container = ProviderContainer(
      overrides: [
        midiServiceProvider.overrideWithValue(midi),
        scoreSourceProvider.overrideWithValue(FakeScoreSource(null)),
        audioServiceProvider.overrideWithValue(RecordingAudioService()),
        audioCaptureServiceProvider.overrideWithValue(capture),
        acousticInputEnabledProvider.overrideWithValue(true),
        notationProvider.overrideWith(
          () => _FixedNotation(
            NotationData(document: sampleFourMeasureDocument()),
          ),
        ),
        preferencesServiceProvider.overrideWithValue(prefs),
        playSyncServiceProvider.overrideWithValue(sync),
        connectivityServiceProvider.overrideWithValue(_SilentConnectivity()),
        playRetrySchedulerProvider.overrideWithValue(_NoopRetryScheduler()),
        currentUserIdProvider.overrideWithValue('u1'),
        canUseOnlineServicesProvider.overrideWithValue(true),
      ],
    );
    addTearDown(container.dispose);
    // The player provider is autoDispose: hold a subscription for the whole
    // test so its listeners (input source, notation) stay alive.
    final sub = container.listen(playerProvider, (_, _) {});
    addTearDown(sub.close);
    return container;
  }

  Future<void> settle() async {
    for (var i = 0; i < 4; i++) {
      await Future<void>.delayed(Duration.zero);
    }
  }

  test('the microphone source opens the session on a keyboard score', () async {
    final container = harness();
    container.read(playerProvider);
    await settle();
    container
        .read(storedInputSourceProvider.notifier)
        .select(PlayerInputSource.microphone);
    await settle();

    verify(capture.beginCapture()).called(1);
    verify(capture.startDetection()).called(1);
    // The score's active window reached the presence stage.
    verify(
      capture.setExpectedPitches(argThat(isNotEmpty)),
    ).called(greaterThanOrEqualTo(1));
  });

  test('switching back to MIDI closes the session', () async {
    final container = harness();
    container.read(playerProvider);
    await settle();
    final source = container.read(storedInputSourceProvider.notifier);
    source.select(PlayerInputSource.microphone);
    await settle();
    source.select(PlayerInputSource.midi);
    await settle();

    verify(capture.stopDetection()).called(1);
    verify(capture.endCapture()).called(1);
  });

  test('an acoustic session never synthesizes live instrument notes', () async {
    final container = harness();
    container.read(playerProvider);
    await settle();
    container
        .read(storedInputSourceProvider.notifier)
        .select(PlayerInputSource.microphone);
    await settle();

    final data = container.read(playerProvider);
    expect(data.usesMicrophoneInput, isTrue);
    // Inherent to the source — independent of the instrument-sounds-itself
    // setting, which is off here.
    expect(data.instrumentSoundsItself, isFalse);
    expect(data.synthesizes(NoteSource.midiDevice), isFalse);
    // The on-screen keyboard still sounds (spec: other audio unaffected).
    expect(data.synthesizes(NoteSource.onScreen), isTrue);
  });

  test('selecting the microphone leaves the MIDI memory untouched', () async {
    final container = harness();
    container.read(playerProvider);
    await settle();
    container
        .read(storedInputSourceProvider.notifier)
        .select(PlayerInputSource.microphone);
    await settle();

    // The source choice persists under its own key; nothing rewrote the play
    // preferences (where the remembered MIDI port lives).
    expect(prefs.store[StoredInputSource.prefsKey], 'microphone');
    expect(prefs.store.keys, isNot(contains('player_preferences')));
  });

  test(
    'free-run with an uncalibrated microphone steers to Wait Mode',
    () async {
      final container = harness();
      await settle();
      container
          .read(storedInputSourceProvider.notifier)
          .select(PlayerInputSource.microphone);
      await settle();

      final player = container.read(playerProvider.notifier);
      // The user turned Wait Mode off; no calibration is stored for the route.
      player.toggleWaitMode();
      expect(container.read(playerProvider).waitMode, isFalse);

      player.setPlaying(true);

      final data = container.read(playerProvider);
      // Steered — with the flag its listener turns into localized copy — never
      // a silently degraded score.
      expect(data.waitMode, isTrue);
      expect(data.micSteeredToWaitMode, isTrue);
      player.acknowledgeMicSteer();
      expect(container.read(playerProvider).micSteeredToWaitMode, isFalse);
    },
  );

  test(
    'Wait Mode play is available uncalibrated and gated by detection',
    () async {
      final container = harness();
      await settle();
      container
          .read(storedInputSourceProvider.notifier)
          .select(PlayerInputSource.microphone);
      await settle();

      final player = container.read(playerProvider.notifier);
      expect(container.read(playerProvider).waitMode, isTrue);
      player.setPlaying(true);

      // No steer: Wait Mode never depends on a calibration measurement.
      expect(container.read(playerProvider).micSteeredToWaitMode, isFalse);
      expect(container.read(playerProvider).isPlaying, isTrue);
    },
  );

  test(
    'a detected note satisfies the Wait Mode gate like a MIDI one',
    () async {
      final container = harness();
      await settle();
      container
          .read(storedInputSourceProvider.notifier)
          .select(PlayerInputSource.microphone);
      await settle();

      final player = container.read(playerProvider.notifier);
      player.setPlaying(true);
      // Land the playhead on the first onset: countdown, then up to the gate.
      player.advance(10000);
      player.advance(5000);
      var data = container.read(playerProvider);
      expect(data.waitMode, isTrue);
      expect(data.blocked, isTrue, reason: 'the gate should be holding');
      final pitch = data.visibleNotes.first.pitch;

      // A detection emission is a normalized event on the SAME stream the MIDI
      // path uses — the player cannot tell the source (spec: source-blind).
      midi.emit(
        MidiEvent(
          kind: MidiEventKind.noteOn,
          pitch: pitch,
          velocity: 80,
          channel: 0,
          timestampMs: BigInt.zero,
        ),
      );
      await settle();

      data = container.read(playerProvider);
      expect(data.gateSatisfied, contains(pitch));
    },
  );

  test('a MIDI session is untouched by the free-run gate', () async {
    final container = harness();
    await settle();

    final player = container.read(playerProvider.notifier);
    player.toggleWaitMode();
    player.setPlaying(true);

    final data = container.read(playerProvider);
    expect(data.waitMode, isFalse);
    expect(data.micSteeredToWaitMode, isFalse);
  });

  test('the flag off forces the MIDI source whatever is stored', () async {
    final container = ProviderContainer(
      overrides: [
        preferencesServiceProvider.overrideWithValue(
          FakePreferencesService({StoredInputSource.prefsKey: 'microphone'}),
        ),
        acousticInputEnabledProvider.overrideWithValue(false),
      ],
    );
    addTearDown(container.dispose);
    container.read(storedInputSourceProvider);
    await settle();

    // Stored value survives (presentational fallback, never a rewrite)…
    expect(
      container.read(storedInputSourceProvider),
      PlayerInputSource.microphone,
    );
    // …but the effective source is MIDI while the flag is off.
    expect(
      container.read(effectivePlayerInputSourceProvider),
      PlayerInputSource.midi,
    );
  });
}
