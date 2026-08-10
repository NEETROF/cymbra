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

import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/auth_service.dart';
import 'package:music/services/catalog_service.dart';
import 'package:music/services/offline_score_cache.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/state/favorite_scores.dart';
import 'package:music/state/favorites_index_store.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';

class _FakeCatalog extends Fake implements CatalogService {
  _FakeCatalog({this.saved = const [], this.throws = false});
  final List<CatalogHit> saved;
  final bool throws;
  @override
  Future<List<CatalogHit>> listSaved() async {
    if (throws) throw AuthException(AuthError.unavailable);
    return saved;
  }
}

class _FakeUpload extends Fake implements ScoreUploadService {
  _FakeUpload({this.throws = false});
  final bool throws;
  @override
  Future<List<ContributedScore>> listMyScores() async {
    if (throws) throw AuthException(AuthError.unavailable);
    return const [];
  }
}

class _FakePrefs implements PreferencesService {
  final Map<String, String> _m = {};
  @override
  Future<String?> getString(String key) async => _m[key];
  @override
  Future<void> setString(String key, String value) async => _m[key] = value;
  @override
  Future<void> remove(String key) async => _m.remove(key);
}

CatalogHit _hit(String id, String title) => CatalogHit(
  id: id,
  title: title,
  composer: 'Composer',
  level: PracticeLevel.beginner,
  license: 'CC-BY-4.0',
  source: 'pdmx',
);

CatalogEntry _catalogEntry(String id) => CatalogEntry(
  id: 'catalog-$id',
  title: 'T $id',
  composer: 'C',
  level: PracticeLevel.beginner,
  catalogId: id,
);

Uint8List _bytes() => Uint8List.fromList(const [1]);

void main() {
  ProviderContainer build({
    required CatalogService catalog,
    required ScoreUploadService upload,
    required OfflineScoreCache cache,
    required FavoritesIndexStore store,
    String? userId = 'u1',
    bool online = true,
  }) {
    final c = ProviderContainer(
      overrides: [
        catalogServiceProvider.overrideWithValue(catalog),
        scoreUploadServiceProvider.overrideWithValue(upload),
        canUseOnlineServicesProvider.overrideWithValue(online),
        currentUserIdProvider.overrideWithValue(userId),
        offlineScoreCacheProvider.overrideWithValue(cache),
        favoritesIndexStoreProvider.overrideWithValue(store),
      ],
    );
    addTearDown(c.dispose);
    return c;
  }

  test('a successful fetch writes the snapshot and sweeps orphans', () async {
    final store = FavoritesIndexStore(_FakePrefs());
    final cache = InMemoryOfflineScoreCache();
    await cache.write(
      'catalog:keep',
      _bytes(),
      etag: 'e',
    ); // a favorite's bytes
    await cache.write('catalog:orphan', _bytes(), etag: 'e'); // not a favorite

    final c = build(
      catalog: _FakeCatalog(saved: [_hit('keep', 'Keep')]),
      upload: _FakeUpload(),
      cache: cache,
      store: store,
    );

    final favorites = await c.read(favoriteScoresProvider.future);
    expect(favorites.map((e) => e.catalogId), ['keep']);
    // Snapshot persisted (metadata only).
    expect((await store.read('u1')).map((e) => e.catalogId), ['keep']);
    // Orphan cache file swept; the favorite's bytes kept.
    expect(await cache.has('catalog:orphan'), isFalse);
    expect(await cache.has('catalog:keep'), isTrue);
  });

  test('offline: falls back to the last-known-good snapshot', () async {
    final store = FavoritesIndexStore(_FakePrefs());
    await store.write('u1', [_catalogEntry('a'), _catalogEntry('b')]);

    final c = build(
      catalog: _FakeCatalog(throws: true),
      upload: _FakeUpload(throws: true),
      cache: InMemoryOfflineScoreCache(),
      store: store,
    );

    final favorites = await c.read(favoriteScoresProvider.future);
    expect(favorites.map((e) => e.id), ['catalog-a', 'catalog-b']);
  });

  test('offline with no snapshot surfaces the failure', () async {
    final c = build(
      catalog: _FakeCatalog(throws: true),
      upload: _FakeUpload(throws: true),
      cache: InMemoryOfflineScoreCache(),
      store: FavoritesIndexStore(_FakePrefs()),
    );
    await expectLater(
      c.read(favoriteScoresProvider.future),
      throwsA(isA<AuthException>()),
    );
  });

  test('guest / signed-out yields an empty list', () async {
    final c = build(
      catalog: _FakeCatalog(saved: [_hit('x', 'X')]),
      upload: _FakeUpload(),
      cache: InMemoryOfflineScoreCache(),
      store: FavoritesIndexStore(_FakePrefs()),
      online: false,
    );
    expect(await c.read(favoriteScoresProvider.future), isEmpty);
  });

  test(
    'offlinePlayableIds reflects which favorites have cached bytes',
    () async {
      final cache = InMemoryOfflineScoreCache();
      await cache.write('catalog:keep', _bytes(), etag: 'e'); // cached
      // 'catalog:other' intentionally NOT cached.
      final c = build(
        catalog: _FakeCatalog(
          saved: [_hit('keep', 'Keep'), _hit('other', 'Other')],
        ),
        upload: _FakeUpload(),
        cache: cache,
        store: FavoritesIndexStore(_FakePrefs()),
      );

      final playable = await c.read(offlinePlayableIdsProvider.future);
      expect(playable, {'catalog-keep'});
    },
  );
}
