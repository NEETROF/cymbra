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
import 'package:music/state/saved_catalog_scores.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';

class _FakeCatalog implements CatalogService {
  _FakeCatalog(this.savedIds);
  Set<String> savedIds;
  final List<String> removeCalls = [];
  int listSavedCalls = 0;
  bool throwNext = false;

  @override
  Future<void> remove(String catalogId) async {
    if (throwNext) throw StateError('boom');
    removeCalls.add(catalogId);
    savedIds = {...savedIds}..remove(catalogId);
  }

  @override
  Future<List<CatalogHit>> listSaved() async {
    listSavedCalls++;
    return [
      for (final id in savedIds)
        CatalogHit(
          id: id,
          title: 'T-$id',
          license: 'CC-BY-4.0',
          source: 'pdmx',
        ),
    ];
  }

  @override
  Future<void> save(String catalogId) async =>
      savedIds = {...savedIds, catalogId};
  @override
  Future<Uint8List> fetchBytes(String catalogId) async => Uint8List(0);
  @override
  Future<CatalogSearchPage> search({
    String query = '',
    String? author,
    PracticeLevel? level,
    CatalogFilters filters = const CatalogFilters(),
    int limit = 20,
    int offset = 0,
  }) async => const CatalogSearchPage(hits: [], nextOffset: 0, total: 0);
}

ProviderContainer _container(_FakeCatalog catalog) {
  final c = ProviderContainer(
    overrides: [
      catalogServiceProvider.overrideWithValue(catalog),
      canUseOnlineServicesProvider.overrideWithValue(true),
    ],
  );
  final sub = c.listen(savedCatalogScoresProvider, (_, _) {});
  addTearDown(sub.close);
  addTearDown(c.dispose);
  return c;
}

void main() {
  test('build lists the saved catalog scores', () async {
    final c = _container(_FakeCatalog({'a', 'b'}));
    final list = await c.read(savedCatalogScoresProvider.future);
    expect(list.map((e) => e.catalogId).toSet(), {'a', 'b'});
  });

  test('remove calls the service then reloads (no direct UI call)', () async {
    final fake = _FakeCatalog({'a', 'b'});
    final c = _container(fake);
    await c.read(savedCatalogScoresProvider.future); // listSavedCalls == 1

    await c.read(savedCatalogScoresProvider.notifier).remove('a');

    expect(fake.removeCalls, ['a']);
    expect(fake.listSavedCalls, 2); // reloaded after the mutation
    final list = await c.read(savedCatalogScoresProvider.future);
    expect(list.map((e) => e.catalogId).toSet(), {'b'});
  });

  test('a failed remove lands in the state, never thrown', () async {
    final fake = _FakeCatalog({'a'})..throwNext = true;
    final c = _container(fake);
    await c.read(savedCatalogScoresProvider.future);

    await c.read(savedCatalogScoresProvider.notifier).remove('a');

    expect(c.read(savedCatalogScoresProvider).hasError, isTrue);
  });
}
