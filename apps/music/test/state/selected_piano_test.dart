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
import 'package:music/state/selected_piano.dart';

import '../support/fakes.dart';
import '../support/prefs_fakes.dart';
import '../support/soundfont_fakes.dart';

ProviderContainer _container({
  required FakePreferencesService prefs,
  required RecordingAudioService audio,
  FakeSoundFontSource? source,
  FakeSoundFontImporter? importer,
  List<PianoEntry>? serverFonts,
}) {
  final container = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(prefs),
      audioServiceProvider.overrideWithValue(audio),
      soundFontSourceProvider.overrideWithValue(
        source ?? FakeSoundFontSource(),
      ),
      soundFontImporterProvider.overrideWithValue(
        importer ?? FakeSoundFontImporter(),
      ),
      // Private library seam: empty (offline-equivalent) so the registry stays
      // local and never hits the network during selection-restore tests.
      privateSoundFontServiceProvider.overrideWithValue(
        FakePrivateSoundFontService(),
      ),
      // The server's downloadable pianos (YDP/Salamander live here now, not in
      // the built-in list); defaults to the two CC-BY grands.
      soundFontCatalogServiceProvider.overrideWithValue(
        FakeSoundFontCatalogService(
          downloadable:
              serverFonts ??
              [
                fakeDownloadPiano(id: 'ydp-grand', label: 'YDP Grand Piano'),
                fakeDownloadPiano(
                  id: 'salamander-grand',
                  label: 'Salamander Grand Piano',
                ),
              ],
        ),
      ),
    ],
  );
  addTearDown(container.dispose);
  // Keep the notifiers alive for the test (they are keepAlive in production).
  container.listen(selectedPianoProvider, (_, _) {}, fireImmediately: true);
  container.listen(pianoCatalogProvider, (_, _) {}, fireImmediately: true);
  return container;
}

void main() {
  test('seeds the bundled default synchronously before restore', () async {
    final container = _container(
      prefs: FakePreferencesService(),
      audio: RecordingAudioService(),
    );
    // Read before pumping the queue: the async restore has not run yet.
    expect(container.read(selectedPianoProvider), defaultPianoId);
    // Let the async restore finish before the container is torn down.
    await pumpEventQueue();
  });

  test('selecting a piano loads its SoundFont and persists the id', () async {
    final audio = RecordingAudioService();
    final prefs = FakePreferencesService();
    final source = FakeSoundFontSource();
    final container = _container(prefs: prefs, audio: audio, source: source);
    await pumpEventQueue();

    await container.read(selectedPianoProvider.notifier).select('ydp-grand');

    expect(container.read(selectedPianoProvider), 'ydp-grand');
    // The right bytes were resolved and handed to the engine.
    expect(source.resolved.map((e) => e.id), contains('ydp-grand'));
    expect(audio.loadedSoundFonts, contains('/fake/soundfonts/ydp-grand.sf2'));
    // The choice is persisted so it survives a relaunch.
    expect(prefs.store[SelectedPiano.prefsKey], 'ydp-grand');
  });

  test('a persisted selection is restored and loaded on launch', () async {
    final audio = RecordingAudioService();
    final container = _container(
      prefs: FakePreferencesService({SelectedPiano.prefsKey: 'ydp-grand'}),
      audio: audio,
    );
    await pumpEventQueue();

    expect(container.read(selectedPianoProvider), 'ydp-grand');
    expect(audio.loadedSoundFonts, contains('/fake/soundfonts/ydp-grand.sf2'));
  });

  test(
    'the default is not reloaded on launch (init already loaded it)',
    () async {
      // A restored *default* selection needs no swap — audio_init loaded it.
      final audio = RecordingAudioService();
      final container = _container(
        prefs: FakePreferencesService({SelectedPiano.prefsKey: defaultPianoId}),
        audio: audio,
      );
      await pumpEventQueue();

      expect(container.read(selectedPianoProvider), defaultPianoId);
      expect(audio.loadedSoundFonts, isEmpty);
    },
  );

  test(
    'an unknown persisted id falls back to the default and re-persists',
    () async {
      final prefs = FakePreferencesService({
        SelectedPiano.prefsKey: 'ghost-piano',
      });
      final container = _container(
        prefs: prefs,
        audio: RecordingAudioService(),
      );
      await pumpEventQueue();

      expect(container.read(selectedPianoProvider), defaultPianoId);
      expect(prefs.store[SelectedPiano.prefsKey], defaultPianoId);
    },
  );

  test('a failing source falls back to the default without crashing', () async {
    final audio = RecordingAudioService();
    final prefs = FakePreferencesService({SelectedPiano.prefsKey: 'ydp-grand'});
    // The persisted download piano cannot be fetched.
    final source = FakeSoundFontSource(failIds: {'ydp-grand'});
    final container = _container(prefs: prefs, audio: audio, source: source);
    await pumpEventQueue();

    expect(container.read(selectedPianoProvider), defaultPianoId);
    expect(prefs.store[SelectedPiano.prefsKey], defaultPianoId);
    // It fell back to loading the bundled default's bytes.
    expect(
      audio.loadedSoundFonts,
      contains('/fake/soundfonts/$defaultPianoId.sf2'),
    );
  });

  test('selecting while audio is a no-op still persists the choice', () async {
    // Model "audio unavailable" with a source that resolves fine but audio that
    // records (never throws): the choice must still persist and be applied.
    final audio = RecordingAudioService(failInit: true);
    final prefs = FakePreferencesService();
    final container = _container(prefs: prefs, audio: audio);
    await pumpEventQueue();

    await container
        .read(selectedPianoProvider.notifier)
        .select('salamander-grand');

    expect(container.read(selectedPianoProvider), 'salamander-grand');
    expect(prefs.store[SelectedPiano.prefsKey], 'salamander-grand');
  });

  test(
    'removing the selected imported piano falls back to the default',
    () async {
      final audio = RecordingAudioService();
      final prefs = FakePreferencesService();
      final imported = fakeUserPiano(id: 'user-x');
      final importer = FakeSoundFontImporter(next: imported);
      final container = _container(
        prefs: prefs,
        audio: audio,
        importer: importer,
      );
      await pumpEventQueue();

      // Import, select it, then remove it.
      await container
          .read(importedSoundFontsProvider.notifier)
          .importSoundFont();
      await container.read(selectedPianoProvider.notifier).select('user-x');
      expect(container.read(selectedPianoProvider), 'user-x');

      await container
          .read(importedSoundFontsProvider.notifier)
          .remove('user-x');
      await pumpEventQueue();

      // The catalog listener drops the vanished selection back to the default.
      expect(container.read(selectedPianoProvider), defaultPianoId);
      expect(prefs.store[SelectedPiano.prefsKey], defaultPianoId);
      // The copied file was deleted.
      expect(importer.deleted.map((e) => e.id), contains('user-x'));
    },
  );
}
