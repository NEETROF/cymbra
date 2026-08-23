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
import 'package:music/state/imported_soundfonts.dart';
import 'package:music/state/piano_catalog.dart';
import 'package:music/state/selected_kit.dart';
import 'package:music/state/selected_piano.dart';

import '../support/fakes.dart';
import '../support/prefs_fakes.dart';
import '../support/soundfont_fakes.dart';

// The per-family kit memory (change: add-drum-audio-channel): same
// restore/fallback shape as SelectedPiano, under its own prefs key, defaulting
// and self-healing to the bundled kit id — and, unlike the piano notifier,
// never touching the synthesizer itself (the font-follows-score controller is
// the application point).

PianoEntry _kit(String id, {String label = 'Kit'}) =>
    fakeDownloadPiano(id: id, label: label, family: SoundFamily.percussion);

PianoEntry _userKit({String id = 'user-kit'}) => PianoEntry(
  id: id,
  label: 'My Kit',
  kind: PianoKind.user,
  source: '/copied/$id.sf2',
  family: SoundFamily.percussion,
);

ProviderContainer _container({
  required FakePreferencesService prefs,
  RecordingAudioService? audio,
  FakeSoundFontImporter? importer,
  List<PianoEntry>? serverFonts,
}) {
  final container = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(prefs),
      audioServiceProvider.overrideWithValue(audio ?? RecordingAudioService()),
      soundFontSourceProvider.overrideWithValue(FakeSoundFontSource()),
      soundFontImporterProvider.overrideWithValue(
        importer ?? FakeSoundFontImporter(),
      ),
      privateSoundFontServiceProvider.overrideWithValue(
        FakePrivateSoundFontService(),
      ),
      soundFontCatalogServiceProvider.overrideWithValue(
        FakeSoundFontCatalogService(
          downloadable:
              serverFonts ??
              [
                _kit('server-kit', label: 'Server Kit'),
                fakeDownloadPiano(id: 'ydp-grand', label: 'YDP Grand Piano'),
              ],
        ),
      ),
    ],
  );
  addTearDown(container.dispose);
  container.listen(selectedKitProvider, (_, _) {}, fireImmediately: true);
  container.listen(pianoCatalogProvider, (_, _) {}, fireImmediately: true);
  return container;
}

void main() {
  test('seeds the bundled kit id synchronously before restore', () async {
    final container = _container(prefs: FakePreferencesService());
    expect(container.read(selectedKitProvider), defaultKitId);
    await pumpEventQueue();
  });

  test(
    'selecting a kit persists it under its own key without touching the synth '
    'or the piano memory',
    () async {
      final prefs = FakePreferencesService();
      final audio = RecordingAudioService();
      final container = _container(prefs: prefs, audio: audio);
      await pumpEventQueue();

      await container.read(selectedKitProvider.notifier).select('server-kit');

      expect(container.read(selectedKitProvider), 'server-kit');
      expect(prefs.store[SelectedKit.prefsKey], 'server-kit');
      // The piano memory is untouched (separate per-family selection)…
      expect(prefs.store[SelectedPiano.prefsKey], isNull);
      // …and NO font swap happened here: the kit only sounds under a
      // percussion score, applied by the font-follows-score controller.
      expect(audio.loadedSoundFonts, isEmpty);
      expect(audio.awaitedLoads, isEmpty);
    },
  );

  test('a persisted kit id is restored on launch', () async {
    final container = _container(
      prefs: FakePreferencesService({SelectedKit.prefsKey: 'server-kit'}),
    );
    await pumpEventQueue();
    expect(container.read(selectedKitProvider), 'server-kit');
  });

  test(
    'an unknown persisted id self-heals to the bundled kit and re-persists',
    () async {
      final prefs = FakePreferencesService({SelectedKit.prefsKey: 'ghost-kit'});
      final container = _container(prefs: prefs);
      await pumpEventQueue();

      expect(container.read(selectedKitProvider), defaultKitId);
      expect(prefs.store[SelectedKit.prefsKey], defaultKitId);
    },
  );

  test('a keyboard font id is not selectable as the kit', () async {
    final prefs = FakePreferencesService();
    final container = _container(prefs: prefs);
    await pumpEventQueue();

    await container.read(selectedKitProvider.notifier).select('ydp-grand');

    expect(container.read(selectedKitProvider), defaultKitId);
    expect(prefs.store[SelectedKit.prefsKey], isNull);
  });

  test(
    'removing the selected imported kit falls back to the bundled kit',
    () async {
      final prefs = FakePreferencesService();
      final importer = FakeSoundFontImporter(next: _userKit());
      final container = _container(prefs: prefs, importer: importer);
      await pumpEventQueue();

      await container
          .read(importedSoundFontsProvider.notifier)
          .importSoundFont();
      await container.read(selectedKitProvider.notifier).select('user-kit');
      expect(container.read(selectedKitProvider), 'user-kit');

      await container
          .read(importedSoundFontsProvider.notifier)
          .remove('user-kit');
      await pumpEventQueue();

      expect(container.read(selectedKitProvider), defaultKitId);
      expect(prefs.store[SelectedKit.prefsKey], defaultKitId);
    },
  );

  test(
    'the bundled kit id stays valid while its asset has not landed',
    () async {
      // No percussion font anywhere: the persisted default must NOT be
      // "self-healed" away — it IS the default; readiness (not selection)
      // reports the honest no-kit outcome.
      final prefs = FakePreferencesService({
        SelectedKit.prefsKey: defaultKitId,
      });
      final container = _container(prefs: prefs, serverFonts: const []);
      await pumpEventQueue();

      expect(container.read(selectedKitProvider), defaultKitId);
    },
  );
}
