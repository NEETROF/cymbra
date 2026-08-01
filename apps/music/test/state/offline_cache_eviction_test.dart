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
import 'package:music/services/catalog_service.dart';
import 'package:music/services/offline_score_cache.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/state/contributed_scores.dart';
import 'package:music/state/saved_catalog_scores.dart';
import 'package:music/state/session_notifier.dart';

class _FakeCatalog extends Fake implements CatalogService {
  @override
  Future<void> remove(String catalogId) async {}
  @override
  Future<List<CatalogHit>> listSaved() async => const [];
}

class _FakeUpload extends Fake implements ScoreUploadService {
  @override
  Future<void> deleteScore(String id) async {}
  @override
  Future<void> setFavorite(String id, bool favorite) async {}
  @override
  Future<List<ContributedScore>> listMyScores() async => const [];
}

Future<void> _flush() async {
  for (var i = 0; i < 4; i++) {
    await Future<void>.delayed(Duration.zero);
  }
}

void main() {
  late InMemoryOfflineScoreCache cache;

  ProviderContainer container() {
    cache = InMemoryOfflineScoreCache();
    final c = ProviderContainer(
      overrides: [
        offlineScoreCacheProvider.overrideWithValue(cache),
        catalogServiceProvider.overrideWithValue(_FakeCatalog()),
        scoreUploadServiceProvider.overrideWithValue(_FakeUpload()),
        canUseOnlineServicesProvider.overrideWithValue(true),
      ],
    );
    addTearDown(c.dispose);
    return c;
  }

  Future<void> seed(String key) =>
      cache.write(key, Uint8List.fromList(const [1]), etag: 'e');

  test('removing a saved catalog score evicts its cache file', () async {
    final c = container();
    await seed('catalog:x');
    await c.read(savedCatalogScoresProvider.notifier).remove('x');
    expect(await cache.has('catalog:x'), isFalse);
  });

  test('deleting an upload evicts its cache file', () async {
    final c = container();
    await seed('contributed:y');
    await c.read(myUploadsProvider.notifier).delete('y');
    expect(await cache.has('contributed:y'), isFalse);
  });

  test('un-favoriting an upload evicts, favoriting keeps the file', () async {
    final c = container();
    await seed('contributed:a');
    await seed('contributed:b');
    await c
        .read(myUploadsProvider.notifier)
        .toggleFavorite('a', favorite: false);
    await c
        .read(myUploadsProvider.notifier)
        .toggleFavorite('b', favorite: true);
    expect(await cache.has('contributed:a'), isFalse, reason: 'un-favorited');
    expect(await cache.has('contributed:b'), isTrue, reason: 'still favorite');
  });

  test('evicting an absent entry is a harmless no-op', () async {
    final c = container();
    await c.read(savedCatalogScoresProvider.notifier).remove('missing');
    expect(await cache.has('catalog:missing'), isFalse);
  });

  test('purgeAll clears every cached entry', () async {
    final c = container();
    await seed('catalog:x');
    await seed('contributed:y');
    await c.read(offlineScoreCacheProvider).purgeAll();
    await _flush();
    expect(await cache.has('catalog:x'), isFalse);
    expect(await cache.has('contributed:y'), isFalse);
  });
}
