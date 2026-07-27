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
import 'package:music/services/score_upload_service.dart';
import 'package:music/state/contributed_scores.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';

/// Records the mutations and reloads, and lets a test force a failure.
class _FakeUpload implements ScoreUploadService {
  _FakeUpload(this.mine);
  List<ContributedScore> mine;
  final List<(String, bool)> favoriteCalls = [];
  final List<String> deleteCalls = [];
  int listCalls = 0;
  bool throwNext = false;

  @override
  Future<List<ContributedScore>> listMyScores() async {
    listCalls++;
    return mine;
  }

  @override
  Future<void> deleteScore(String id) async {
    if (throwNext) throw StateError('boom');
    deleteCalls.add(id);
    mine = mine.where((s) => s.id != id).toList();
  }

  @override
  Future<void> setFavorite(String id, bool favorite) async {
    if (throwNext) throw StateError('boom');
    favoriteCalls.add((id, favorite));
  }

  @override
  Future<Uint8List> fetchBytes(String id) async => Uint8List(0);
  @override
  Future<ContributedScore> upload({
    required Uint8List data,
    required String filename,
    required PracticeLevel level,
    required RightsBasis rightsBasis,
    required bool rightsAck,
    String? fallbackTitle,
    String? fallbackComposer,
  }) async => throw UnimplementedError();
}

ContributedScore _upload(String id, {bool favorite = false}) =>
    ContributedScore(
      id: id,
      level: PracticeLevel.beginner,
      createdAt: DateTime.utc(2026, 5, 1),
      measureCount: 4,
      timeSig: '4/4',
      keyFifths: 0,
      title: 'T-$id',
      favorite: favorite,
    );

ProviderContainer _container(_FakeUpload upload) {
  final c = ProviderContainer(
    overrides: [
      scoreUploadServiceProvider.overrideWithValue(upload),
      canUseOnlineServicesProvider.overrideWithValue(true),
    ],
  );
  final sub = c.listen(myUploadsProvider, (_, _) {});
  addTearDown(sub.close);
  addTearDown(c.dispose);
  return c;
}

void main() {
  test('build loads the caller uploads', () async {
    final fake = _FakeUpload([_upload('a'), _upload('b')]);
    final c = _container(fake);
    final list = await c.read(myUploadsProvider.future);
    expect(list.map((s) => s.id), ['a', 'b']);
  });

  test(
    'toggleFavorite calls the service then reloads (no direct UI call)',
    () async {
      final fake = _FakeUpload([_upload('a')]);
      final c = _container(fake);
      await c.read(myUploadsProvider.future); // initial load (listCalls == 1)

      await c
          .read(myUploadsProvider.notifier)
          .toggleFavorite('a', favorite: true);

      expect(fake.favoriteCalls, [('a', true)]);
      expect(fake.listCalls, 2); // reloaded after the mutation
      expect(c.read(myUploadsProvider).hasValue, isTrue);
    },
  );

  test('delete calls the service then reloads', () async {
    final fake = _FakeUpload([_upload('a'), _upload('b')]);
    final c = _container(fake);
    await c.read(myUploadsProvider.future);

    await c.read(myUploadsProvider.notifier).delete('a');

    expect(fake.deleteCalls, ['a']);
    final list = await c.read(myUploadsProvider.future);
    expect(list.map((s) => s.id), ['b']); // reflects the deletion
  });

  test('a failed mutation lands in the state, never thrown', () async {
    final fake = _FakeUpload([_upload('a')])..throwNext = true;
    final c = _container(fake);
    await c.read(myUploadsProvider.future);

    // Does not throw — the guard captures it into the AsyncValue.
    await c.read(myUploadsProvider.notifier).delete('a');

    expect(c.read(myUploadsProvider).hasError, isTrue);
  });
}
