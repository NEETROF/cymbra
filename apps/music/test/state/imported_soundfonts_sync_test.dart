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

import 'dart:io';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/private_soundfont_service.dart';
import 'package:music/services/soundfont_importer.dart';
import 'package:music/services/soundfont_storage.dart';
import 'package:music/state/imported_soundfonts.dart';
import 'package:music/state/piano_catalog.dart';

import '../support/prefs_fakes.dart';
import '../support/soundfont_fakes.dart';

/// Private-library sync (change: add-soundfont-moderation): imports upload to the
/// server, propose/remove call through, and a build pulls the server library
/// down. Uses a real temp dir so the cache-to-file path runs.
void main() {
  late Directory tmp;

  setUp(() async {
    tmp = await Directory.systemTemp.createTemp('sf_sync_test');
  });
  tearDown(() async {
    if (tmp.existsSync()) await tmp.delete(recursive: true);
  });

  ProviderContainer container({
    required FakeSoundFontImporter importer,
    required FakePrivateSoundFontService private,
    FakePreferencesService? prefs,
  }) {
    final c = ProviderContainer(
      overrides: [
        preferencesServiceProvider.overrideWithValue(
          prefs ?? FakePreferencesService(),
        ),
        soundFontImporterProvider.overrideWithValue(importer),
        privateSoundFontServiceProvider.overrideWithValue(private),
        soundFontStorageDirProvider.overrideWith((ref) async => tmp),
      ],
    );
    addTearDown(c.dispose);
    return c;
  }

  /// A real on-disk `.sf2` the importer "returns" (so the notifier can read its
  /// bytes to upload).
  Future<PianoEntry> realImport(String id, String label) async {
    final f = File('${tmp.path}/$id.sf2');
    await f.writeAsBytes('RIFF____sfbk$id'.codeUnits, flush: true);
    return PianoEntry(
      id: id,
      label: label,
      kind: PianoKind.user,
      source: f.path,
    );
  }

  test('import uploads to the private library and records the remote id', () async {
    final entry = await realImport('u1', 'My Grand');
    final private = FakePrivateSoundFontService();
    final c = container(
      importer: FakeSoundFontImporter(next: entry),
      private: private,
    );
    await c.read(importedSoundFontsProvider.future);

    final imported = await c
        .read(importedSoundFontsProvider.notifier)
        .importSoundFont();

    expect(private.imported, ['My Grand']); // uploaded
    expect(imported?.remoteId, isNotNull); // server id recorded
    expect(
      c.read(importedSoundFontsProvider).requireValue.single.remoteId,
      imported?.remoteId,
    );
  });

  test('proposeToPublicCatalog forwards licence + attestation', () async {
    final entry = await realImport('u1', 'My Grand');
    final private = FakePrivateSoundFontService();
    final c = container(
      importer: FakeSoundFontImporter(next: entry),
      private: private,
    );
    await c.read(importedSoundFontsProvider.future);
    final imported = await c
        .read(importedSoundFontsProvider.notifier)
        .importSoundFont();

    await c
        .read(importedSoundFontsProvider.notifier)
        .proposeToPublicCatalog(
          imported!.id,
          license: 'CC-BY-3.0',
          attribution: 'Me',
          attestation: true,
        );

    expect(private.proposed, hasLength(1));
    expect(private.proposed.single.license, 'CC-BY-3.0');
    expect(private.proposed.single.attestation, isTrue);
    expect(private.proposed.single.id, imported.remoteId);
  });

  test('proposing an un-synced font throws instead of a silent no-op', () async {
    // A local-only entry (import upload failed): no remote id yet.
    final entry = await realImport('u1', 'My Grand');
    final private = FakePrivateSoundFontService()..failImport = true;
    final c = container(
      importer: FakeSoundFontImporter(next: entry),
      private: private,
    );
    await c.read(importedSoundFontsProvider.future);
    final imported = await c
        .read(importedSoundFontsProvider.notifier)
        .importSoundFont();
    expect(imported?.remoteId, isNull);

    expect(
      () => c
          .read(importedSoundFontsProvider.notifier)
          .proposeToPublicCatalog(imported!.id, license: 'x', attestation: true),
      throwsA(isA<PrivateSoundFontException>()),
    );
  });

  test('remove deletes the font server-side so it stops syncing', () async {
    final entry = await realImport('u1', 'My Grand');
    final private = FakePrivateSoundFontService();
    final c = container(
      importer: FakeSoundFontImporter(next: entry),
      private: private,
    );
    await c.read(importedSoundFontsProvider.future);
    final imported = await c
        .read(importedSoundFontsProvider.notifier)
        .importSoundFont();

    await c.read(importedSoundFontsProvider.notifier).remove(imported!.id);

    expect(private.deleted, [imported.remoteId]);
    expect(c.read(importedSoundFontsProvider).requireValue, isEmpty);
  });

  test('build pulls a server-only font into the registry (cross-device sync)', () async {
    // The server library already has a font this device never imported.
    final private = FakePrivateSoundFontService(
      library: const [
        RemoteSoundFont(id: 'remote-x', label: 'Shared Grand', sizeBytes: 12),
      ],
    );
    final c = container(
      importer: FakeSoundFontImporter(),
      private: private,
    );

    final synced = await c.read(importedSoundFontsProvider.future);

    // It appears as a user piano backed by a downloaded cache file.
    final font = synced.singleWhere((e) => e.remoteId == 'remote-x');
    expect(font.label, 'Shared Grand');
    expect(font.kind, PianoKind.user);
    expect(File(font.source).existsSync(), isTrue);
  });
}
