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
import 'package:music/services/score_upload_service.dart';
import 'package:music/state/contributed_scores.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';

@GenerateNiceMocks([MockSpec<ScoreUploadService>()])
import 'contributed_scores_test.mocks.dart';

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

ProviderContainer _container(ScoreUploadService upload) {
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
    final upload = MockScoreUploadService();
    when(
      upload.listMyScores(),
    ).thenAnswer((_) async => [_upload('a'), _upload('b')]);

    final c = _container(upload);
    final list = await c.read(myUploadsProvider.future);
    expect(list.map((s) => s.id), ['a', 'b']);
  });

  test(
    'toggleFavorite calls the service then reloads (no direct UI call)',
    () async {
      final upload = MockScoreUploadService();
      when(upload.listMyScores()).thenAnswer((_) async => [_upload('a')]);
      when(upload.setFavorite(any, any)).thenAnswer((_) async {});

      final c = _container(upload);
      await c.read(myUploadsProvider.future); // initial load

      await c
          .read(myUploadsProvider.notifier)
          .toggleFavorite('a', favorite: true);

      verify(upload.setFavorite('a', true)).called(1);
      // Reloaded after the mutation: build + reload == 2 list calls.
      verify(upload.listMyScores()).called(2);
      expect(c.read(myUploadsProvider).hasValue, isTrue);
    },
  );

  test('delete calls the service then reloads', () async {
    final upload = MockScoreUploadService();
    // A tiny bit of state so the post-delete reload reflects the removal.
    var mine = [_upload('a'), _upload('b')];
    when(upload.listMyScores()).thenAnswer((_) async => mine);
    when(upload.deleteScore(any)).thenAnswer((inv) async {
      final id = inv.positionalArguments.first as String;
      mine = mine.where((s) => s.id != id).toList();
    });

    final c = _container(upload);
    await c.read(myUploadsProvider.future);

    await c.read(myUploadsProvider.notifier).delete('a');

    verify(upload.deleteScore('a')).called(1);
    final list = await c.read(myUploadsProvider.future);
    expect(list.map((s) => s.id), ['b']); // reflects the deletion
  });

  test('a failed mutation lands in the state, never thrown', () async {
    final upload = MockScoreUploadService();
    when(upload.listMyScores()).thenAnswer((_) async => [_upload('a')]);
    when(upload.deleteScore(any)).thenThrow(StateError('boom'));

    final c = _container(upload);
    await c.read(myUploadsProvider.future);

    // Does not throw — the guard captures it into the AsyncValue.
    await c.read(myUploadsProvider.notifier).delete('a');

    expect(c.read(myUploadsProvider).hasError, isTrue);
  });
}
