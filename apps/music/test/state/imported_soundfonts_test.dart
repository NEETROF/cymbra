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

import 'dart:convert';
import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/soundfont_catalog_service.dart';
import 'package:music/services/soundfont_importer.dart';
import 'package:music/state/imported_soundfonts.dart';
import 'package:music/state/piano_catalog.dart';

import '../support/prefs_fakes.dart';
import '../support/soundfont_fakes.dart';

ProviderContainer _container(
  FakePreferencesService prefs,
  FakeSoundFontImporter importer,
) {
  final container = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(prefs),
      soundFontImporterProvider.overrideWithValue(importer),
      // No server download list needed for import/registry tests.
      soundFontCatalogServiceProvider.overrideWithValue(
        FakeSoundFontCatalogService(),
      ),
    ],
  );
  addTearDown(container.dispose);
  container.listen(
    importedSoundFontsProvider,
    (_, _) {},
    fireImmediately: true,
  );
  container.listen(pianoCatalogProvider, (_, _) {}, fireImmediately: true);
  return container;
}

String _encodeRegistry(List<PianoEntry> entries) =>
    jsonEncode([for (final e in entries) e.toJson()]);

void main() {
  test('a valid import is added to the catalog and persisted', () async {
    final prefs = FakePreferencesService();
    final entry = fakeUserPiano(id: 'u1', label: 'My Grand');
    final container = _container(prefs, FakeSoundFontImporter(next: entry));
    await pumpEventQueue();

    final imported = await container
        .read(importedSoundFontsProvider.notifier)
        .importSoundFont();

    expect(imported?.id, 'u1');
    expect(
      container.read(importedSoundFontsProvider).requireValue.map((e) => e.id),
      ['u1'],
    );
    // It joins the full catalog alongside the built-ins.
    expect(
      container.read(pianoCatalogProvider).map((e) => e.id),
      containsAll(<String>[defaultPianoId, 'u1']),
    );
    // Persisted for the next launch.
    expect(prefs.store[ImportedSoundFonts.prefsKey], contains('u1'));
  });

  test('an imported piano survives a relaunch', () async {
    // A "previous launch" persisted one user piano.
    final entry = fakeUserPiano(id: 'kept', label: 'Kept Piano');
    final prefs = FakePreferencesService({
      ImportedSoundFonts.prefsKey: _encodeRegistry([entry]),
    });
    final container = _container(prefs, FakeSoundFontImporter());
    await pumpEventQueue();

    final restored = container.read(importedSoundFontsProvider).requireValue;
    expect(restored.map((e) => e.id), ['kept']);
    expect(restored.single.label, 'Kept Piano');
    expect(restored.single.kind, PianoKind.user);
  });

  test(
    'an invalid file is rejected and leaves the catalog unchanged',
    () async {
      final prefs = FakePreferencesService();
      final container = _container(
        prefs,
        FakeSoundFontImporter(throwInvalid: true),
      );
      await pumpEventQueue();

      await expectLater(
        container.read(importedSoundFontsProvider.notifier).importSoundFont(),
        throwsA(isA<SoundFontImportException>()),
      );

      expect(container.read(importedSoundFontsProvider).requireValue, isEmpty);
      expect(prefs.store.containsKey(ImportedSoundFonts.prefsKey), isFalse);
    },
  );

  test('cancelling the picker adds nothing', () async {
    final prefs = FakePreferencesService();
    // next == null models a cancel.
    final container = _container(prefs, FakeSoundFontImporter());
    await pumpEventQueue();

    final result = await container
        .read(importedSoundFontsProvider.notifier)
        .importSoundFont();

    expect(result, isNull);
    expect(container.read(importedSoundFontsProvider).requireValue, isEmpty);
  });

  test(
    'removing an imported piano deletes its file and drops the entry',
    () async {
      final entry = fakeUserPiano(id: 'gone');
      final prefs = FakePreferencesService({
        ImportedSoundFonts.prefsKey: _encodeRegistry([entry]),
      });
      final importer = FakeSoundFontImporter();
      final container = _container(prefs, importer);
      await pumpEventQueue();
      expect(
        container
            .read(importedSoundFontsProvider)
            .requireValue
            .map((e) => e.id),
        ['gone'],
      );

      await container.read(importedSoundFontsProvider.notifier).remove('gone');

      expect(container.read(importedSoundFontsProvider).requireValue, isEmpty);
      expect(importer.deleted.map((e) => e.id), ['gone']);
      // Persisted registry is now empty.
      expect(
        prefs.store[ImportedSoundFonts.prefsKey],
        _encodeRegistry(const []),
      );
    },
  );

  test('header validation accepts a RIFF/sfbk file and rejects others', () {
    // RIFF <size> sfbk ...
    final ok = [0x52, 0x49, 0x46, 0x46, 0, 0, 0, 0, 0x73, 0x66, 0x62, 0x6b];
    expect(isValidSoundFont(Uint8List.fromList(ok)), isTrue);
    // RIFF WAVE — a WAV, not a SoundFont.
    final wav = [0x52, 0x49, 0x46, 0x46, 0, 0, 0, 0, 0x57, 0x41, 0x56, 0x45];
    expect(isValidSoundFont(Uint8List.fromList(wav)), isFalse);
    expect(isValidSoundFont(Uint8List.fromList(const [1, 2, 3])), isFalse);
  });
}
