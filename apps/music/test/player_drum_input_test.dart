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
import 'package:music/services/private_soundfont_service.dart';
import 'package:music/services/soundfont_catalog_service.dart';
import 'package:music/services/soundfont_importer.dart';
import 'package:music/services/soundfont_source.dart';
import 'package:music/state/drum_kit.dart';
import 'package:music/state/notation_data.dart';
import 'package:music/state/notation_notifier.dart';
import 'package:music/state/performance_scoring.dart';
import 'package:music/state/piano_catalog.dart';
import 'package:music/state/player_data.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/src/rust/api/midi.dart' show MidiEcho;
import 'package:music/state/score_font.dart';

import 'support/fakes.dart';
import 'support/notation_fakes.dart';
import 'support/prefs_fakes.dart';
import 'support/soundfont_fakes.dart';

// The percussion STROKE path (change: add-drum-input-mapping): every stroke —
// pad, pedal, external kit — converges on the player's note entry points,
// sounds as a one-shot and flashes the surface it resolves to. Since
// add-drum-scoring those strokes are also judged; the judgment itself is
// covered by `drum_scoring_test.dart`.

/// A [Notation] fixed to a parsed document, so the player loads a percussion
/// timeline without byte sources.
class _FixedNotation extends Notation {
  _FixedNotation(this._value);
  final NotationData _value;
  @override
  NotationData build() => _value;
}

Future<void> _flush() => Future<void>.delayed(Duration.zero);

void main() {
  late RecordingAudioService audio;
  late FakeMidiService midi;
  late ProviderContainer container;

  /// The player's current state / notifier — local functions (a test body
  /// cannot declare getters), read fresh on every use.
  PlayerData data() => container.read(playerProvider);
  Player player() => container.read(playerProvider.notifier);

  Future<void> build({bool percussion = true}) async {
    audio = RecordingAudioService();
    // `echoTo` makes the fake behave like the engine: when the app arms the
    // echo, a stroke emitted here is sounded in the MIDI callback — before it
    // ever reaches the notifier, which is exactly why the notifier no longer
    // sounds it (change: add-drum-input-mapping, beta fix for input latency).
    midi = FakeMidiService(
      ports: const ['Kit'],
      connected: 'Kit',
      echoTo: audio,
    );
    container = ProviderContainer(
      overrides: [
        midiServiceProvider.overrideWithValue(midi),
        scoreSourceProvider.overrideWithValue(FakeScoreSource()),
        audioServiceProvider.overrideWithValue(audio),
        preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
        soundFontSourceProvider.overrideWithValue(FakeSoundFontSource()),
        soundFontImporterProvider.overrideWithValue(FakeSoundFontImporter()),
        privateSoundFontServiceProvider.overrideWithValue(
          FakePrivateSoundFontService(),
        ),
        soundFontCatalogServiceProvider.overrideWithValue(
          FakeSoundFontCatalogService(
            downloadable: [
              fakeDownloadPiano(
                id: defaultKitId,
                label: 'Kit',
                family: SoundFamily.percussion,
              ),
            ],
          ),
        ),
        if (percussion)
          notationProvider.overrideWith(
            () => _FixedNotation(NotationData(document: sampleDrumDocument())),
          ),
      ],
    );
    addTearDown(container.dispose);
    container.listen(playerProvider, (_, _) {}, fireImmediately: true);
    await _flush();
  }

  /// Opens the kit-readiness gate the way the player subtree's listener does,
  /// so live strokes may sound.
  Future<void> readyKit() async {
    await container
        .read(scoreFontProvider.notifier)
        .setScoreFamily(percussion: true);
    expect(container.read(scoreFontProvider), KitFontStatus.ready);
  }

  /// Every note-level call the seam received, in order — the init and
  /// SoundFont-swap noise filtered out.
  List<String> strokeCalls() => [
    for (final c in audio.calls)
      if (c.startsWith('drumOn') ||
          c.startsWith('drumOff') ||
          c.startsWith('on:') ||
          c.startsWith('off:') ||
          c == 'allOff')
        c,
  ];

  tearDown(() async => midi.close());

  // The beta report was "strong latency between the hit and the sound": a
  // stroke used to be sounded only after crossing the bridge and going through
  // the Dart event loop, so the delay a player heard was whatever the UI thread
  // happened to be doing. The engine now sounds it in its own MIDI callback —
  // the app keeps the policy and pushes it (`setEcho`), and stops sounding what
  // the engine already did, so no stroke is ever played twice.
  group('the engine sounds live strokes itself', () {
    test('a ready kit arms the drum echo, and the app then leaves a device '
        'stroke alone — exactly one sound per hit', () async {
      await build();
      await readyKit();
      expect(midi.echo, MidiEcho.drum);

      midi.emit(noteOnEvent(38));
      await _flush();
      // One sound, and it came from the engine (the fake sounded it in its
      // callback, before the notifier ever saw the event).
      expect(audio.drumOns.map((e) => e.key), [38]);
      // …while everything the stroke drives still ran in the app.
      expect(data().activeNotes, contains(38));
      expect(data().struckSurfacesMs, isNotEmpty);
    });

    test(
      'the echo follows the app\'s own two guards, and nothing else',
      () async {
        await build();
        // Visual-only until the kit lands: the engine must not sound a stroke
        // through the still-loaded piano font either.
        expect(midi.echo, MidiEcho.off);
        await readyKit();
        expect(midi.echo, MidiEcho.drum);

        // An instrument that sounds its own strokes is never doubled.
        player().setInstrumentSoundsItself(enabled: true);
        expect(midi.echo, MidiEcho.off);
        player().setInstrumentSoundsItself(enabled: false);
        expect(midi.echo, MidiEcho.drum);
      },
    );

    test('a keyboard score echoes on the melodic channel', () async {
      await build(percussion: false);
      expect(midi.echo, MidiEcho.melodic);
      midi.emit(noteOnEvent(60));
      await _flush();
      expect(audio.noteOns.map((e) => e.pitch), [60]);
      expect(audio.drumOns, isEmpty);
    });

    test('leaving the player disarms it: no surface plays what the instrument '
        'sends', () async {
      await build();
      await readyKit();
      expect(midi.echo, MidiEcho.drum);
      container.dispose();
      expect(midi.echo, MidiEcho.off);
    });
  });

  group('sounding — one-shot, never the piano voice', () {
    test('an on-screen stroke sounds through drumOn, never noteOn', () async {
      await build();
      await readyKit();
      // The snare of the loaded groove (38), as the pad emits it.
      player().noteOn(38);
      expect(audio.drumOns.map((e) => e.key), [38]);
      expect(audio.noteOns, isEmpty);
      expect(audio.calls.where((c) => c.startsWith('on:')), isEmpty);
    });

    test('a stroke sounds while playback is stopped', () async {
      await build();
      await readyKit();
      expect(data().isPlaying, isFalse);
      player().noteOn(42);
      expect(audio.drumOns.map((e) => e.key), [42]);
    });

    test('velocity stays unconsumed: however hard the event, the one-shot is '
        'invoked identically', () async {
      await build();
      await readyKit();
      midi
        ..emit(noteOnEvent(38, velocity: 1))
        ..emit(noteOnEvent(38, velocity: 127));
      await _flush();
      expect(audio.drumOns, hasLength(2));
      expect(
        audio.drumOns.map((e) => e.velocity),
        everyElement(AudioService.defaultVelocity),
      );
    });

    test('before the kit install resolves a stroke is silent, but it still '
        'registers and still flashes', () async {
      await build(); // no readyKit: percussion is visual-only
      player().noteOn(38);
      expect(audio.drumOns, isEmpty);
      expect(audio.noteOns, isEmpty); // and NEVER through the piano voice
      // Sounding is what is unavailable — the stroke itself exists.
      expect(data().activeNotes, contains(38));
      expect(data().struckSurfacesMs.keys, [data().struckSurfaceFor(38)]);
    });

    test('a keyboard score is untouched: live notes still take the melodic '
        'path', () async {
      await build(percussion: false);
      player().noteOn(60);
      expect(audio.noteOns.map((e) => e.pitch), [60]);
      expect(audio.drumOns, isEmpty);
      player().noteOff(60);
      expect(audio.noteOffs, [60]);
    });
  });

  group('releases are bookkeeping, never meaning', () {
    test('an immediate note-off changes nothing audible — the one-shot rings '
        'to its natural end', () async {
      await build();
      await readyKit();
      player()
        ..noteOn(38)
        ..noteOff(38); // the release an e-kit sends milliseconds later
      // The release is deliberately NOT forwarded: the engine's drum_off is a
      // NoteOff on the drum channel, which would put the voice into its
      // release stage and clip the sound a millisecond after the stick left.
      expect(audio.drumOffs, isEmpty);
      expect(audio.noteOffs, isEmpty);
      expect(strokeCalls(), ['drumOn:38']);
      // …and the flash is untouched by the release (it decays on its own).
      expect(data().struckSurfacesMs, isNotEmpty);
    });

    test('a missing note-off leaves no sounding voice, and the stale held '
        'entry clears on the next pair', () async {
      await build();
      await readyKit();
      player().noteOn(38); // a sloppy kit never sends the release
      expect(data().activeNotes, contains(38));
      // Nothing sustains: the only call made is the self-terminating one-shot.
      expect(strokeCalls(), ['drumOn:38']);
      player()
        ..noteOn(38)
        ..noteOff(38);
      expect(data().activeNotes, isNot(contains(38)));
      expect(strokeCalls(), ['drumOn:38', 'drumOn:38']);
    });
  });

  group('convergence and external input', () {
    test('a pad stroke and a device stroke are identical below the sounding '
        'decision', () async {
      await build();
      await readyKit();
      player().noteOn(38, source: NoteSource.onScreen);
      final onScreen = (
        active: data().activeNotes,
        struck: data().struckSurfacesMs.keys.toList(),
      );
      player().noteOff(38, source: NoteSource.onScreen);

      midi.emit(noteOnEvent(38));
      await _flush();
      expect(data().activeNotes, onScreen.active);
      expect(data().struckSurfacesMs.keys.toList(), onScreen.struck);
      // Both sounded through the one-shot; only the source differed.
      expect(audio.drumOns.map((e) => e.key), [38, 38]);
    });

    test(
      'a device stroke sounds and flashes the pad its number resolves to',
      () async {
        await build();
        await readyKit();
        midi.emit(noteOnEvent(42)); // the closed hi-hat, lane 0
        await _flush();
        expect(audio.drumOns.map((e) => e.key), [42]);
        expect(data().struckSurfacesMs.keys, [
          laneIndexOf(data().drumLanes, 42),
        ]);
        // The open stroke an e-kit produces naturally lights the SAME pad.
        midi.emit(noteOnEvent(46));
        await _flush();
        expect(data().struckSurfacesMs.keys, [0]);
      },
    );

    test('a module that sounds itself is not doubled, while the stroke still '
        'registers and flashes', () async {
      await build();
      await readyKit();
      player().setInstrumentSoundsItself(enabled: true);
      midi.emit(noteOnEvent(38));
      await _flush();
      expect(audio.drumOns, isEmpty);
      expect(data().activeNotes, contains(38));
      expect(data().struckSurfacesMs, isNotEmpty);
      // The rule exempts only the instrument's own strokes: a pad tap sounds.
      player().noteOn(38, source: NoteSource.onScreen);
      expect(audio.drumOns.map((e) => e.key), [38]);
    });

    test('a kick stroke flashes the pedal, from either kick number', () async {
      await build();
      await readyKit();
      midi.emit(noteOnEvent(36));
      await _flush();
      expect(data().struckSurfacesMs.keys, [kPedalSurface]);
      expect(audio.drumOns.map((e) => e.key), [36]);
    });
  });

  group('free play and unsuppressed input', () {
    test('a stroke outside the score\'s kit sounds, flashes nothing and '
        'raises no error', () async {
      await build();
      await readyKit();
      // The groove uses hi-hat, snare and kick only.
      midi.emit(noteOnEvent(49)); // a crash
      await _flush();
      expect(audio.drumOns.map((e) => e.key), [49]);
      expect(data().struckSurfacesMs, isEmpty);
      expect(data().activeNotes, contains(49));
    });

    test('hand selection never suppresses input: a kick still sounds and '
        'flashes the pedal during hands-only practice', () async {
      await build();
      await readyKit();
      player().setSelectedHands(Hand.right); // hands only
      // The filter is presentation-only: the foot events are gone from the
      // cascade…
      expect(
        data().visibleNotes.any((n) => kKickGmNumbers.contains(n.pitch)),
        isFalse,
      );
      // …while the pedal is still there to play, and still answers.
      player().noteOn(36);
      expect(audio.drumOns.map((e) => e.key), [36]);
      expect(data().struckSurfacesMs.keys, [kPedalSurface]);

      // And the reverse: a hand stroke during feet-only practice.
      player().setSelectedHands(Hand.left);
      player().noteOn(38);
      expect(audio.drumOns.map((e) => e.key), [36, 38]);
      expect(
        data().struckSurfacesMs.keys,
        contains(laneIndexOf(data().drumLanes, 38)),
      );
    });
  });

  group('honest feedback', () {
    test('a stroke on an onset and one far from any onset produce the '
        'identical flash', () async {
      await build();
      await readyKit();
      player()
        ..toggleWaitMode() // free run: the gate would freeze at the first onset
        ..setPlaying(true)
        ..advance(500); // the groove's snare onset
      expect(data().onsetPitchesAt(data().elapsedMs), contains(38));
      player().noteOn(38);
      final onOnset = Map.of(data().struckSurfacesMs);

      player().advance(125); // between two eighths: no onset at all
      expect(data().onsetPitchesAt(data().elapsedMs), isEmpty);
      player().noteOn(38);
      final farFromAny = Map.of(data().struckSurfacesMs);

      // One state either way — same surface, same single entry, and the decay
      // is a pure function of the stroke's age. The **struck** flash still
      // claims nothing about correctness now that a matcher exists (change:
      // add-drum-scoring): the verdict is the cascade's spark, and what the
      // gate awaits is the pad's expected outline — three channels, each
      // saying one thing.
      expect(onOnset.keys, [laneIndexOf(data().drumLanes, 38)]);
      expect(farFromAny.keys, onOnset.keys);
    });

    test('the flash survives an immediate release: it decays on its own '
        'clock, never on the note-off', () async {
      await build();
      await readyKit();
      player()
        ..noteOn(38)
        ..noteOff(38);
      final struck = data().struckSurfacesMs;
      expect(struck, hasLength(1));
      // Freshly stamped: the release cleared nothing.
      expect(
        DateTime.now().millisecondsSinceEpoch - struck.values.first,
        lessThan(1000),
      );
    });
  });

  // The interim this group used to pin ("the scorer never arms for a
  // percussion score") is lifted by add-drum-scoring: the matcher exists, so a
  // full percussion run is judged like any other. The successor assertions
  // live in `drum_scoring_test.dart`; what stays here is the keyboard
  // regression and the practice carve-out, which are the two things this file
  // is positioned to catch.
  group('run activation over the percussion path', () {
    test(
      'a full percussion run arms the scorer and accumulates judgments',
      () async {
        await build();
        await readyKit();
        player()
          ..toggleWaitMode() // free run
          ..setPlaying(true);
        expect(container.read(performanceScorerProvider).active, isTrue);
        player()
          ..advance(500) // the groove's snare onset
          ..noteOn(38)
          ..noteOff(38);
        expect(
          container.read(performanceScorerProvider).recentHits,
          isNotEmpty,
        );
      },
    );

    test('a keyboard run still scores exactly as before', () async {
      await build(percussion: false);
      player()
        ..toggleWaitMode() // free run
        ..setPlaying(true);
      expect(container.read(performanceScorerProvider).active, isTrue);
      player().noteOn(60);
      player().advance(100);
      expect(container.read(performanceScorerProvider).active, isTrue);
    });
  });
}
