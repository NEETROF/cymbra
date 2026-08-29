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
import 'package:music/services/auth_service.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/score_collections.dart';

@GenerateNiceMocks([MockSpec<ScoreUploadService>()])
import 'score_collections_test.mocks.dart';

ScoreCollection _collection(String id, String name) =>
    ScoreCollection(id: id, name: name, createdAt: DateTime.utc(2026));

ContributedScore _score(String id) => ContributedScore(
  id: id,
  level: PracticeLevel.beginner,
  createdAt: DateTime.utc(2026),
  measureCount: 4,
  timeSig: '4/4',
  keyFifths: 0,
  title: id,
);

ProviderContainer _make(MockScoreUploadService service) {
  final c = ProviderContainer(
    overrides: [scoreUploadServiceProvider.overrideWithValue(service)],
  );
  addTearDown(c.dispose);
  return c;
}

void main() {
  test('the list is read from the server and re-read after a mutation', () async {
    final service = MockScoreUploadService();
    var listCalls = 0;
    when(service.listCollections()).thenAnswer((_) async {
      listCalls++;
      return [_collection('c1', 'Chopin')];
    });
    when(service.createCollection(any)).thenAnswer(
      (_) async => _collection('c2', 'Etudes'),
    );
    final c = _make(service);

    expect(await c.read(scoreCollectionsProvider.future), hasLength(1));
    expect(listCalls, 1);

    final err = await c.read(scoreCollectionsProvider.notifier).create('Etudes');
    expect(err, isNull);
    // The server is the source of truth, so the list is read again — that is
    // what makes another device's change show up here.
    await c.read(scoreCollectionsProvider.future);
    expect(listCalls, 2);
  });

  test('a name collision surfaces as a typed reason, not an exception', () async {
    final service = MockScoreUploadService();
    when(service.listCollections()).thenAnswer((_) async => const []);
    when(
      service.createCollection(any),
    ).thenThrow(AuthException(AuthError.alreadyExists));
    final c = _make(service);
    await c.read(scoreCollectionsProvider.future);

    final err = await c.read(scoreCollectionsProvider.notifier).create('Chopin');
    expect(err, CollectionError.nameTaken);
  });

  test('an empty name surfaces as invalidName, anything else as failed', () async {
    final service = MockScoreUploadService();
    when(service.listCollections()).thenAnswer((_) async => const []);
    when(
      service.createCollection('  '),
    ).thenThrow(AuthException(AuthError.invalidArgument));
    when(service.createCollection('boom')).thenThrow(StateError('offline'));
    final c = _make(service);
    await c.read(scoreCollectionsProvider.future);
    final n = c.read(scoreCollectionsProvider.notifier);

    expect(await n.create('  '), CollectionError.invalidName);
    expect(await n.create('boom'), CollectionError.failed);
  });

  test('membership changes go through and refresh the list', () async {
    final service = MockScoreUploadService();
    when(service.listCollections()).thenAnswer((_) async => const []);
    final c = _make(service);
    await c.read(scoreCollectionsProvider.future);
    final n = c.read(scoreCollectionsProvider.notifier);

    expect(await n.addScore('c1', 's1'), isNull);
    expect(await n.removeScore('c1', 's1'), isNull);
    expect(await n.remove('c1'), isNull);
    verify(service.addToCollection('c1', 's1')).called(1);
    verify(service.removeFromCollection('c1', 's1')).called(1);
    verify(service.deleteCollection('c1')).called(1);
  });

  group('the filter', () {
    test('no filter fetches nothing; selecting one fetches that collection',
        () async {
      final service = MockScoreUploadService();
      when(service.listCollections()).thenAnswer((_) async => const []);
      when(
        service.listMyScoresInCollection('c1'),
      ).thenAnswer((_) async => [_score('s1')]);
      final c = _make(service);

      expect(await c.read(scoresInCollectionProvider.future), isEmpty);
      verifyNever(service.listMyScoresInCollection(any));

      c.read(collectionFilterProvider.notifier).select('c1');
      expect(
        (await c.read(scoresInCollectionProvider.future)).map((s) => s.id),
        ['s1'],
      );
    });

    test('clearing the filter returns to the whole library', () async {
      final service = MockScoreUploadService();
      when(service.listCollections()).thenAnswer((_) async => const []);
      when(
        service.listMyScoresInCollection('c1'),
      ).thenAnswer((_) async => [_score('s1')]);
      final c = _make(service);
      c.read(collectionFilterProvider.notifier).select('c1');
      await c.read(scoresInCollectionProvider.future);

      c.read(collectionFilterProvider.notifier).select(null);
      expect(c.read(collectionFilterProvider), isNull);
      expect(await c.read(scoresInCollectionProvider.future), isEmpty);
    });
  });
}
