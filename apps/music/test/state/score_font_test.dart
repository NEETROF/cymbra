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
import 'package:music/services/preferences_service.dart';
import 'package:music/services/private_soundfont_service.dart';
import 'package:music/services/soundfont_catalog_service.dart';
import 'package:music/services/soundfont_importer.dart';
import 'package:music/services/soundfont_source.dart';
import 'package:music/state/piano_catalog.dart';
import 'package:music/state/score_font.dart';
import 'package:music/state/selected_kit.dart';
import 'package:music/state/selected_piano.dart';

import '../support/fakes.dart';
import '../support/prefs_fakes.dart';
import '../support/soundfont_fakes.dart';

// The font-follows-score controller (change: add-drum-audio-channel): a
// percussion score installs the remembered kit through the AWAITED swap and
// publishes readiness; a keyboard return restores the remembered piano; every
// failure lands in the honest visual-only `unavailable` state.

PianoEntry _kit(String id) =>
    fakeDownloadPiano(id: id, label: id, family: SoundFamily.percussion);

ProviderContainer _container({
  required RecordingAudioService audio,
  FakePreferencesService? prefs,
  FakeSoundFontSource? source,
  List<PianoEntry> serverFonts = const [],
}) {
  final container = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(
        prefs ?? FakePreferencesService(),
      ),
      audioServiceProvider.overrideWithValue(audio),
      soundFontSourceProvider.overrideWithValue(
        source ?? FakeSoundFontSource(),
      ),
      soundFontImporterProvider.overrideWithValue(FakeSoundFontImporter()),
      privateSoundFontServiceProvider.overrideWithValue(
        FakePrivateSoundFontService(),
      ),
      soundFontCatalogServiceProvider.overrideWithValue(
        FakeSoundFontCatalogService(downloadable: serverFonts),
      ),
    ],
  );
  addTearDown(container.dispose);
  container.listen(scoreFontProvider, (_, _) {}, fireImmediately: true);
  container.listen(selectedKitProvider, (_, _) {}, fireImmediately: true);
  container.listen(selectedPianoProvider, (_, _) {}, fireImmediately: true);
  return container;
}

void main() {
  test(
    'a percussion score installs the remembered kit through the awaited swap '
    'and becomes ready',
    () async {
      final audio = RecordingAudioService();
      // The bundled-kit id resolvable (models the asset having landed, served
      // through the catalog like the bundled piano is).
      final c = _container(audio: audio, serverFonts: [_kit(defaultKitId)]);
      await pumpEventQueue();

      await c.read(scoreFontProvider.notifier).setScoreFamily(percussion: true);

      expect(c.read(scoreFontProvider), KitFontStatus.ready);
      expect(audio.awaitedLoads, ['/fake/soundfonts/$defaultKitId.sf2']);
      // The plain (unawaited) swap path was never used for the gate.
      expect(audio.loadedSoundFonts, isEmpty);
    },
  );

  test('a failed awaited install lands in unavailable (visual-only)', () async {
    final audio = RecordingAudioService()..awaitedLoadResult = false;
    final c = _container(audio: audio, serverFonts: [_kit(defaultKitId)]);
    await pumpEventQueue();

    await c.read(scoreFontProvider.notifier).setScoreFamily(percussion: true);

    expect(c.read(scoreFontProvider), KitFontStatus.unavailable);
  });

  test(
    'with no resolvable kit anywhere (bundled asset not landed), the state is '
    'unavailable and nothing is swapped',
    () async {
      final audio = RecordingAudioService();
      final c = _container(audio: audio); // empty catalog: bundled piano only
      await pumpEventQueue();

      await c.read(scoreFontProvider.notifier).setScoreFamily(percussion: true);

      expect(c.read(scoreFontProvider), KitFontStatus.unavailable);
      expect(audio.awaitedLoads, isEmpty);
      expect(audio.loadedSoundFonts, isEmpty);
    },
  );

  test('an unresolvable chosen kit falls back to the bundled kit', () async {
    final audio = RecordingAudioService();
    final prefs = FakePreferencesService({SelectedKit.prefsKey: 'broken-kit'});
    final c = _container(
      audio: audio,
      prefs: prefs,
      source: FakeSoundFontSource(failIds: {'broken-kit'}),
      serverFonts: [_kit('broken-kit'), _kit(defaultKitId)],
    );
    await pumpEventQueue();
    expect(c.read(selectedKitProvider), 'broken-kit');

    await c.read(scoreFontProvider.notifier).setScoreFamily(percussion: true);

    expect(c.read(scoreFontProvider), KitFontStatus.ready);
    expect(audio.awaitedLoads, ['/fake/soundfonts/$defaultKitId.sf2']);
  });

  test('returning to a keyboard surface restores the remembered piano and goes '
      'inactive', () async {
    final audio = RecordingAudioService();
    final c = _container(audio: audio, serverFonts: [_kit(defaultKitId)]);
    await pumpEventQueue();
    final notifier = c.read(scoreFontProvider.notifier);
    await notifier.setScoreFamily(percussion: true);
    expect(c.read(scoreFontProvider), KitFontStatus.ready);

    await notifier.setScoreFamily(percussion: false);

    expect(c.read(scoreFontProvider), KitFontStatus.inactive);
    // The piano came back through the existing resolve+load path (the
    // bundled default here).
    expect(
      audio.loadedSoundFonts,
      contains('/fake/soundfonts/$defaultPianoId.sf2'),
    );
  });

  test(
    'a keyboard score with no kit swap in between never reloads the piano',
    () async {
      final audio = RecordingAudioService();
      final c = _container(audio: audio);
      await pumpEventQueue();

      await c
          .read(scoreFontProvider.notifier)
          .setScoreFamily(percussion: false);

      expect(c.read(scoreFontProvider), KitFontStatus.inactive);
      expect(audio.loadedSoundFonts, isEmpty);
    },
  );

  test(
    'picking a different kit while a percussion score is active re-installs it',
    () async {
      final audio = RecordingAudioService();
      final c = _container(
        audio: audio,
        serverFonts: [_kit(defaultKitId), _kit('other-kit')],
      );
      await pumpEventQueue();
      final notifier = c.read(scoreFontProvider.notifier);
      await notifier.setScoreFamily(percussion: true);
      expect(audio.awaitedLoads, ['/fake/soundfonts/$defaultKitId.sf2']);

      // The picker writes the selection; this controller reacts via listen.
      await c.read(selectedKitProvider.notifier).select('other-kit');
      await pumpEventQueue();

      expect(c.read(scoreFontProvider), KitFontStatus.ready);
      expect(audio.awaitedLoads, [
        '/fake/soundfonts/$defaultKitId.sf2',
        '/fake/soundfonts/other-kit.sf2',
      ]);
    },
  );
}
