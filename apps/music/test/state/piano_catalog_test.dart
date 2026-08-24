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
import 'dart:io';

import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/soundfont_catalog_service.dart';
import 'package:music/state/piano_catalog.dart';

import '../support/prefs_fakes.dart';
import '../support/soundfont_fakes.dart';

ProviderContainer _container(FakeSoundFontCatalogService catalog) {
  final container = ProviderContainer(
    overrides: [
      preferencesServiceProvider.overrideWithValue(FakePreferencesService()),
      soundFontCatalogServiceProvider.overrideWithValue(catalog),
    ],
  );
  addTearDown(container.dispose);
  container.listen(pianoCatalogProvider, (_, _) {}, fireImmediately: true);
  return container;
}

void main() {
  test('before the server list resolves the catalog is just the default', () {
    final container = _container(FakeSoundFontCatalogService());
    // Synchronous read: the async listing has not resolved yet — the two
    // bundled fonts (the piano and the drum kit) are what ships offline.
    expect(container.read(pianoCatalogProvider).map((e) => e.id), [
      defaultPianoId,
      defaultKitId,
    ]);
  });

  test("server-listed fonts join the catalog as 'download' pianos", () async {
    final container = _container(
      FakeSoundFontCatalogService(
        downloadable: [
          fakeDownloadPiano(id: 'ydp-grand', label: 'YDP Grand Piano'),
          fakeDownloadPiano(id: 'salamander-grand', label: 'Salamander'),
        ],
      ),
    );
    await pumpEventQueue();

    final catalog = container.read(pianoCatalogProvider);
    expect(catalog.map((e) => e.id), [
      defaultPianoId,
      defaultKitId,
      'ydp-grand',
      'salamander-grand',
    ]);
    expect(
      catalog.firstWhere((e) => e.id == 'ydp-grand').kind,
      PianoKind.download,
    );
  });

  test('a server entry duplicating the bundled default is dropped', () async {
    final container = _container(
      FakeSoundFontCatalogService(
        downloadable: [
          // The server catalog also carries the bundled default id.
          fakeDownloadPiano(id: defaultPianoId, label: 'Upright (server)'),
          fakeDownloadPiano(id: 'ydp-grand', label: 'YDP Grand Piano'),
        ],
      ),
    );
    await pumpEventQueue();

    final catalog = container.read(pianoCatalogProvider);
    // The default appears once (the bundled entry), not twice.
    expect(catalog.where((e) => e.id == defaultPianoId).length, 1);
    expect(
      catalog.firstWhere((e) => e.id == defaultPianoId).kind,
      PianoKind.bundled,
    );
    expect(catalog.map((e) => e.id), [
      defaultPianoId,
      defaultKitId,
      'ydp-grand',
    ]);
  });

  test('an empty listing leaves only the bundled fonts', () async {
    final container = _container(FakeSoundFontCatalogService());
    await pumpEventQueue();
    expect(container.read(pianoCatalogProvider).map((e) => e.id), [
      defaultPianoId,
      defaultKitId,
    ]);
  });

  group('instrument family (change: add-drum-audio-channel)', () {
    test('the wire instrument maps onto the family, legacy piano included', () {
      expect(soundFamilyFromWire('percussion'), SoundFamily.percussion);
      expect(soundFamilyFromWire('keyboard'), SoundFamily.keyboard);
      // The not-yet-migrated backend spelling stays the keyboard family.
      expect(soundFamilyFromWire('piano'), SoundFamily.keyboard);
      // Unknown/empty fail open to the historical family, never to a kit.
      expect(soundFamilyFromWire(''), SoundFamily.keyboard);
      expect(soundFamilyFromWire('theremin'), SoundFamily.keyboard);
      expect(soundFamilyFromWire(' Percussion '), SoundFamily.percussion);
    });

    test('the family round-trips through the registry JSON', () {
      final kit = PianoEntry(
        id: 'k1',
        label: 'Kit',
        kind: PianoKind.user,
        source: '/k1.sf2',
        family: SoundFamily.percussion,
      );
      expect(PianoEntry.fromJson(kit.toJson()).family, SoundFamily.percussion);
      // Keyboard is the implicit default: not written, decoded back.
      final piano = fakeUserPiano();
      expect(piano.toJson().containsKey('family'), isFalse);
      expect(PianoEntry.fromJson(piano.toJson()).family, SoundFamily.keyboard);
    });

    test('a registry persisted before the family existed decodes keyboard', () {
      final entry = PianoEntry.fromJson({
        'id': 'old',
        'label': 'Old Import',
        'kind': 'user',
        'source': '/old.sf2',
      });
      expect(entry.family, SoundFamily.keyboard);
    });

    test('the bundled kit is the percussion default, and every built-in entry '
        'points at a declared asset', () {
      // The licence sign-off landed the bytes (tasks 6.1/6.2/9.1): the stable
      // id the selection defaults to now resolves to a real bundled entry.
      expect(defaultKitId, 'fluid-r3-drums');
      final kit = builtInKits.single;
      expect(kit.id, defaultKitId);
      expect(kit.family, SoundFamily.percussion);
      expect(kit.kind, PianoKind.bundled);
      // The asset path must be one pubspec declares — a catalog pointing at
      // bytes that do not ship is the failure this test exists to prevent.
      expect(kit.source, startsWith('assets/soundfonts/'));
      final pubspec = File('pubspec.yaml').readAsStringSync();
      for (final entry in [...builtInPianos, ...builtInKits]) {
        expect(
          pubspec,
          contains(entry.source),
          reason: '${entry.id} points at an undeclared asset',
        );
      }
    });
  });
}
