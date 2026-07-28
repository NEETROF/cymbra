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
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/catalog_service.dart';
import 'package:music/state/saved_catalog_scores.dart';
import 'package:music/state/session_notifier.dart';

@GenerateNiceMocks([MockSpec<CatalogService>()])
import 'saved_catalog_scores_test.mocks.dart';

List<CatalogHit> _hits(Set<String> ids) => [
  for (final id in ids)
    CatalogHit(id: id, title: 'T-$id', license: 'CC-BY-4.0', source: 'pdmx'),
];

ProviderContainer _container(CatalogService catalog) {
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
    final catalog = MockCatalogService();
    when(catalog.listSaved()).thenAnswer((_) async => _hits({'a', 'b'}));

    final c = _container(catalog);
    final list = await c.read(savedCatalogScoresProvider.future);
    expect(list.map((e) => e.catalogId).toSet(), {'a', 'b'});
  });

  test('remove calls the service then reloads (no direct UI call)', () async {
    final catalog = MockCatalogService();
    // A tiny bit of state so the post-remove reload reflects the removal.
    var saved = {'a', 'b'};
    when(catalog.listSaved()).thenAnswer((_) async => _hits(saved));
    when(catalog.remove(any)).thenAnswer((inv) async {
      saved = {...saved}..remove(inv.positionalArguments.first as String);
    });

    final c = _container(catalog);
    await c.read(savedCatalogScoresProvider.future); // initial load

    await c.read(savedCatalogScoresProvider.notifier).remove('a');

    verify(catalog.remove('a')).called(1);
    // Reloaded after the mutation: build + reload == 2 list calls.
    verify(catalog.listSaved()).called(2);
    final list = await c.read(savedCatalogScoresProvider.future);
    expect(list.map((e) => e.catalogId).toSet(), {'b'});
  });

  test('a failed remove lands in the state, never thrown', () async {
    final catalog = MockCatalogService();
    when(catalog.listSaved()).thenAnswer((_) async => _hits({'a'}));
    when(catalog.remove(any)).thenThrow(StateError('boom'));

    final c = _container(catalog);
    await c.read(savedCatalogScoresProvider.future);

    await c.read(savedCatalogScoresProvider.notifier).remove('a');

    expect(c.read(savedCatalogScoresProvider).hasError, isTrue);
  });
}
