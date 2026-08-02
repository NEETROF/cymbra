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
    // Synchronous read: the async listing has not resolved yet.
    expect(container.read(pianoCatalogProvider).map((e) => e.id), [
      defaultPianoId,
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
    expect(catalog.map((e) => e.id), [defaultPianoId, 'ydp-grand']);
  });

  test('an empty listing leaves only the bundled default', () async {
    final container = _container(FakeSoundFontCatalogService());
    await pumpEventQueue();
    expect(container.read(pianoCatalogProvider).map((e) => e.id), [
      defaultPianoId,
    ]);
  });
}
