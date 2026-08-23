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
import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/private_soundfont_service.dart';
import 'package:music/services/soundfont_importer.dart';
import 'package:music/services/soundfont_storage.dart';
import 'package:music/src/rust/api/audio.dart' show SoundFontFamilyEvidence;
import 'package:music/state/imported_soundfonts.dart';
import 'package:music/state/piano_catalog.dart';

import '../support/prefs_fakes.dart';
import '../support/soundfont_fakes.dart';

// Import family detection (change: add-drum-audio-channel): the family is
// DETECTED from preset banks — kit-only → percussion, otherwise keyboard,
// null evidence (cannot verify) → keyboard — recorded in the persisted
// registry, and re-detected for fonts pulled from the private server library
// (the sync now declares the detected family; the server verifies the claim).

const _kitOnly = SoundFontFamilyEvidence(
  hasPercussionPresets: true,
  hasMelodicPresets: false,
);
const _melodicOnly = SoundFontFamilyEvidence(
  hasPercussionPresets: false,
  hasMelodicPresets: true,
);
const _bothBanks = SoundFontFamilyEvidence(
  hasPercussionPresets: true,
  hasMelodicPresets: true,
);

void main() {
  group('familyFromEvidence', () {
    test('only bank-128 presets detect percussion', () {
      expect(familyFromEvidence(_kitOnly), SoundFamily.percussion);
    });

    test('melodic and both-banks fonts detect keyboard', () {
      expect(familyFromEvidence(_melodicOnly), SoundFamily.keyboard);
      // A both-banks import lands keyboard by design (single-valued family).
      expect(familyFromEvidence(_bothBanks), SoundFamily.keyboard);
    });

    test('null evidence (cannot verify) records keyboard', () {
      expect(familyFromEvidence(null), SoundFamily.keyboard);
    });
  });

  group('registry persistence and server pull', () {
    late Directory tmp;

    setUp(() async {
      tmp = await Directory.systemTemp.createTemp('sf_family_test');
    });
    tearDown(() async {
      if (tmp.existsSync()) await tmp.delete(recursive: true);
    });

    ProviderContainer container({
      required FakePreferencesService prefs,
      FakeSoundFontImporter? importer,
      FakePrivateSoundFontService? private,
      SoundFontFamilyEvidence? probeEvidence,
    }) {
      final c = ProviderContainer(
        overrides: [
          preferencesServiceProvider.overrideWithValue(prefs),
          soundFontImporterProvider.overrideWithValue(
            importer ?? FakeSoundFontImporter(),
          ),
          privateSoundFontServiceProvider.overrideWithValue(
            private ?? FakePrivateSoundFontService(),
          ),
          soundFontStorageDirProvider.overrideWith((ref) async => tmp),
          // The engine probe, doubled: canned evidence instead of the FFI.
          soundFontFamilyProbeProvider.overrideWithValue(
            (path) async => probeEvidence,
          ),
        ],
      );
      addTearDown(c.dispose);
      return c;
    }

    test('the detected family rides the sync as the declared claim', () async {
      // The server verifies a declared family against the bytes' preset
      // banks; sending the detection turns a silent re-derivation into a
      // checked claim (change: add-drum-audio-channel, 7.9).
      final private = FakePrivateSoundFontService();
      final kit = PianoEntry(
        id: 'kit-sync',
        label: 'Sync Kit',
        kind: PianoKind.user,
        source: '${tmp.path}/kit-sync.sf2',
        family: SoundFamily.percussion,
      );
      await File(kit.source).writeAsBytes([0x52, 0x49, 0x46, 0x46]);
      final c = container(
        prefs: FakePreferencesService(),
        importer: FakeSoundFontImporter(next: kit),
        private: private,
      );
      await c.read(importedSoundFontsProvider.notifier).importSoundFont();
      expect(private.importedFamilies, ['percussion']);
    });

    test('an imported kit is persisted with its detected family and survives a '
        'relaunch', () async {
      final prefs = FakePreferencesService();
      final kit = PianoEntry(
        id: 'kit-1',
        label: 'My Kit',
        kind: PianoKind.user,
        source: '${tmp.path}/kit-1.sf2',
        family: SoundFamily.percussion,
      );
      final c = container(
        prefs: prefs,
        importer: FakeSoundFontImporter(next: kit),
      );
      await c.read(importedSoundFontsProvider.notifier).importSoundFont();

      // Persisted with the detected family…
      expect(
        prefs.store[ImportedSoundFonts.prefsKey],
        contains('"family":"percussion"'),
      );

      // …and restored with it on the next launch (a fresh container).
      final relaunched = container(prefs: prefs);
      final restored = await relaunched.read(importedSoundFontsProvider.future);
      expect(restored.single.family, SoundFamily.percussion);
    });

    test(
      'a font pulled from the private server library is re-detected from its '
      'cached bytes (the sync wire carries no family)',
      () async {
        final prefs = FakePreferencesService();
        final c = container(
          prefs: prefs,
          private: FakePrivateSoundFontService(
            library: const [
              RemoteSoundFont(id: 'r1', label: 'Server Kit', sizeBytes: 16),
            ],
          ),
          probeEvidence: _kitOnly,
        );

        final pulled = await c.read(importedSoundFontsProvider.future);

        expect(pulled.single.remoteId, 'r1');
        expect(pulled.single.family, SoundFamily.percussion);
        expect(
          prefs.store[ImportedSoundFonts.prefsKey],
          contains('"family":"percussion"'),
        );
      },
    );

    test(
      'the production importer detects the saved file through the probe seam',
      () async {
        // No importer override: the DEFAULT SoundFontImporterImpl runs, so
        // save() itself exercises the write→probe→record chain.
        final c = ProviderContainer(
          overrides: [
            preferencesServiceProvider.overrideWithValue(
              FakePreferencesService(),
            ),
            soundFontStorageDirProvider.overrideWith((ref) async => tmp),
            soundFontFamilyProbeProvider.overrideWithValue(
              (path) async => _kitOnly,
            ),
          ],
        );
        addTearDown(c.dispose);

        final saved = await c
            .read(soundFontImporterProvider)
            .save(Uint8List.fromList('RIFF____sfbkKIT!'.codeUnits), 'A Kit');

        expect(saved.kind, PianoKind.user);
        expect(saved.family, SoundFamily.percussion);
        expect(File(saved.source).existsSync(), isTrue);
      },
    );

    test(
      'a pulled font whose bytes cannot be verified lands keyboard',
      () async {
        final c = container(
          prefs: FakePreferencesService(),
          private: FakePrivateSoundFontService(
            library: const [
              RemoteSoundFont(id: 'r2', label: 'Opaque', sizeBytes: 16),
            ],
          ),
          probeEvidence: null,
        );

        final pulled = await c.read(importedSoundFontsProvider.future);
        expect(pulled.single.family, SoundFamily.keyboard);
      },
    );
  });
}
