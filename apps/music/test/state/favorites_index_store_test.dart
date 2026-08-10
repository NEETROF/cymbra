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

import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/preferences_service.dart';
import 'package:music/state/favorites_index_store.dart';
import 'package:music/state/score_catalog.dart';

/// In-memory [PreferencesService] (plaintext, no platform channel).
class _FakePrefs implements PreferencesService {
  final Map<String, String> _m = {};
  @override
  Future<String?> getString(String key) async => _m[key];
  @override
  Future<void> setString(String key, String value) async => _m[key] = value;
  @override
  Future<void> remove(String key) async => _m.remove(key);
}

CatalogEntry _catalog(String id) => CatalogEntry(
  id: 'catalog-$id',
  title: 'Title $id',
  composer: 'Composer $id',
  level: PracticeLevel.intermediate,
  catalogId: id,
);

CatalogEntry _upload(String id) => CatalogEntry(
  id: 'contrib-$id',
  title: 'Upload $id',
  composer: 'Me',
  level: PracticeLevel.beginner,
  contributedId: id,
  uploaderHandle: 'me',
);

void main() {
  test('round-trips the metadata snapshot (no bytes) per user', () async {
    final store = FavoritesIndexStore(_FakePrefs());
    final entries = [_catalog('a'), _upload('b')];
    await store.write('u1', entries);

    final read = await store.read('u1');
    expect(read.map((e) => e.id), ['catalog-a', 'contrib-b']);
    expect(read[0].catalogId, 'a');
    expect(read[0].title, 'Title a');
    expect(read[0].level, PracticeLevel.intermediate);
    expect(read[1].contributedId, 'b');
    expect(read[1].uploaderHandle, 'me');
  });

  test('is scoped per user', () async {
    final store = FavoritesIndexStore(_FakePrefs());
    await store.write('u1', [_catalog('a')]);
    expect(await store.read('u2'), isEmpty);
  });

  test('an empty write clears the snapshot', () async {
    final store = FavoritesIndexStore(_FakePrefs());
    await store.write('u1', [_catalog('a')]);
    await store.write('u1', const []);
    expect(await store.read('u1'), isEmpty);
  });

  test('clear drops the snapshot', () async {
    final store = FavoritesIndexStore(_FakePrefs());
    await store.write('u1', [_catalog('a')]);
    await store.clear('u1');
    expect(await store.read('u1'), isEmpty);
  });

  test('an unreadable / corrupt snapshot degrades to empty', () async {
    final prefs = _FakePrefs();
    await prefs.setString('favorites-index:u1', 'not-json{');
    final store = FavoritesIndexStore(prefs);
    expect(await store.read('u1'), isEmpty);
  });
}
