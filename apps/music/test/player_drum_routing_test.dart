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
import 'package:music/state/notation_data.dart';
import 'package:music/state/notation_notifier.dart';
import 'package:music/state/piano_catalog.dart';
import 'package:music/state/player_notifier.dart';
import 'package:music/state/score_font.dart';

import 'support/fakes.dart';
import 'support/notation_fakes.dart';
import 'support/prefs_fakes.dart';
import 'support/soundfont_fakes.dart';

// Playback routing by the score's family (change: add-drum-audio-channel): a
// percussion score's scheduled notes go through AudioService.drumOn/drumOff —
// gated on the kit readiness — while a keyboard score's melodic calls are
// byte-identical to before and never touch the drum pair.

/// A [Notation] fixed to a parsed document (same shape as the player notifier
/// suite's), so the player loads the percussion timeline without byte sources.
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

  Future<void> build({bool percussion = true}) async {
    audio = RecordingAudioService();
    midi = FakeMidiService();
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
        // The bundled-kit id resolvable through the catalog, so the readiness
        // gate can actually open in the "ready" cases.
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

  tearDown(() async => midi.close());

  test(
    'a percussion score routes scheduled notes through drumOn/drumOff, never '
    'the melodic pair, once the kit is ready',
    () async {
      await build();
      expect(container.read(playerProvider).isPercussion, isTrue);
      // Install the kit (what the player subtree's listener does on open).
      await container
          .read(scoreFontProvider.notifier)
          .setScoreFamily(percussion: true);
      expect(container.read(scoreFontProvider), KitFontStatus.ready);

      final notifier = container.read(playerProvider.notifier)
        ..setPlaying(true)
        // Cross the first onsets: hi-hat (42) + kick (36) at 0ms, then the
        // next hi-hat eighth at 250ms.
        ..advance(300);

      expect(audio.drumOns.map((e) => e.key), containsAll([42, 36]));
      expect(audio.noteOns, isEmpty);

      // Passing the eighth's end releases it on the drum pair.
      notifier.advance(300);
      expect(audio.drumOffs, isNotEmpty);
      expect(audio.noteOffs, isEmpty);

      // Stop still silences everything (all-off covers both channels).
      notifier.setPlaying(false);
      expect(audio.allNotesOffCount, greaterThan(0));
    },
  );

  test(
    'before the kit install resolves, percussion playback is visual-only: the '
    'playhead advances and nothing reaches the synth',
    () async {
      await build();
      // No setScoreFamily: the readiness gate stays inactive (same outcome as
      // a swap still in flight or an unavailable kit).
      final notifier = container.read(playerProvider.notifier)
        ..setPlaying(true)
        ..advance(500);

      expect(container.read(playerProvider).elapsedMs, greaterThan(0));
      expect(audio.drumOns, isEmpty);
      expect(audio.noteOns, isEmpty);

      notifier.setPlaying(false);
    },
  );

  test(
    'an unavailable kit (no resolvable font) keeps percussion visual-only',
    () async {
      await build();
      // Force the honest no-kit outcome: the awaited swap fails.
      audio.awaitedLoadResult = false;
      await container
          .read(scoreFontProvider.notifier)
          .setScoreFamily(percussion: true);
      expect(container.read(scoreFontProvider), KitFontStatus.unavailable);

      container.read(playerProvider.notifier)
        ..setPlaying(true)
        ..advance(500)
        ..setPlaying(false);

      expect(audio.drumOns, isEmpty);
      expect(audio.noteOns, isEmpty);
    },
  );

  test('a keyboard score is byte-identical: melodic calls as before, zero drum '
      'calls', () async {
    await build(percussion: false); // the demo score: C4 [0,500), D4 [500,1000)
    container.read(playerProvider.notifier)
      // Wait Mode defaults on for keyboard scores; free-run the playback so
      // the schedule sounds without input.
      ..toggleWaitMode()
      ..setPlaying(true)
      ..advance(300)
      ..advance(300);

    // Same call sequence the melodic path always produced…
    expect(audio.calls.where((c) => c.startsWith('on:')), ['on:60', 'on:62']);
    expect(audio.calls.where((c) => c.startsWith('off:')), ['off:60']);
    // …and the drum pair untouched.
    expect(audio.drumOns, isEmpty);
    expect(audio.drumOffs, isEmpty);
  });
}
