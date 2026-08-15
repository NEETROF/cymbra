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
import 'package:music/state/contributed_scores.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';

@GenerateNiceMocks([MockSpec<ScoreUploadService>()])
import 'contributed_scores_test.mocks.dart';

ContributedScore _upload(String id, {bool favorite = false, String? status}) =>
    ContributedScore(
      id: id,
      level: PracticeLevel.beginner,
      createdAt: DateTime.utc(2026, 5, 1),
      measureCount: 4,
      timeSig: '4/4',
      keyFifths: 0,
      title: 'T-$id',
      favorite: favorite,
      proposalStatus: status,
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

  test(
    'proposeToPublicCatalog calls the service (with justification) then reloads',
    () async {
      final upload = MockScoreUploadService();
      when(upload.listMyScores()).thenAnswer((_) async => [_upload('a')]);
      when(
        upload.propose(
          scoreId: anyNamed('scoreId'),
          license: anyNamed('license'),
          attestation: anyNamed('attestation'),
          attribution: anyNamed('attribution'),
          resubmissionNote: anyNamed('resubmissionNote'),
        ),
      ).thenAnswer((_) async {});

      final c = _container(upload);
      await c.read(myUploadsProvider.future); // initial load

      await c
          .read(myUploadsProvider.notifier)
          .proposeToPublicCatalog(
            'a',
            license: 'CC-BY-4.0',
            attestation: true,
            resubmissionNote: 'fixed the key signature',
          );

      verify(
        upload.propose(
          scoreId: 'a',
          license: 'CC-BY-4.0',
          attestation: true,
          attribution: '',
          resubmissionNote: 'fixed the key signature',
        ),
      ).called(1);
      // Reloaded after the mutation: build + reload == 2 list calls.
      verify(upload.listMyScores()).called(2);
      expect(
        c.read(scoreProposalFeedbackProvider),
        ScoreProposalOutcome.submitted,
      );
    },
  );

  test(
    'a refused proposal reports the duplicate outcome and keeps the list',
    () async {
      final upload = MockScoreUploadService();
      when(upload.listMyScores()).thenAnswer((_) async => [_upload('a')]);
      when(
        upload.propose(
          scoreId: anyNamed('scoreId'),
          license: anyNamed('license'),
          attestation: anyNamed('attestation'),
          attribution: anyNamed('attribution'),
          resubmissionNote: anyNamed('resubmissionNote'),
        ),
      ).thenThrow(
        const AuthException(
          AuthError.alreadyExists,
          'this score is already in the catalog (cid)',
        ),
      );

      final c = _container(upload);
      await c.read(myUploadsProvider.future);

      await c
          .read(myUploadsProvider.notifier)
          .proposeToPublicCatalog('a', license: 'CC0-1.0', attestation: true);

      // The refusal is typed for the UI…
      expect(
        c.read(scoreProposalFeedbackProvider),
        ScoreProposalOutcome.alreadyInCatalog,
      );
      // …and the uploads list is untouched: the server changed nothing, so
      // "mes partitions" must not empty itself over a per-card message.
      expect(c.read(myUploadsProvider).hasError, isFalse);
      expect(c.read(myUploadsProvider).requireValue.map((s) => s.id), ['a']);
      verify(upload.listMyScores()).called(1); // no pointless reload
    },
  );

  test('any other refusal reports the generic failure', () async {
    final upload = MockScoreUploadService();
    when(upload.listMyScores()).thenAnswer((_) async => [_upload('a')]);
    when(
      upload.propose(
        scoreId: anyNamed('scoreId'),
        license: anyNamed('license'),
        attestation: anyNamed('attestation'),
        attribution: anyNamed('attribution'),
        resubmissionNote: anyNamed('resubmissionNote'),
      ),
    ).thenThrow(StateError('boom'));

    final c = _container(upload);
    await c.read(myUploadsProvider.future);

    await c
        .read(myUploadsProvider.notifier)
        .proposeToPublicCatalog('a', license: 'CC0-1.0', attestation: true);

    expect(c.read(scoreProposalFeedbackProvider), ScoreProposalOutcome.failed);
    expect(c.read(myUploadsProvider).hasError, isFalse);
  });

  test(
    'refresh re-fetches the uploads (picks up server-side status changes)',
    () async {
      final upload = MockScoreUploadService();
      var status = 'accepted';
      when(
        upload.listMyScores(),
      ).thenAnswer((_) async => [_upload('a', status: status)]);

      final c = _container(upload);
      var list = await c.read(myUploadsProvider.future); // build (call 1)
      expect(list.single.proposalStatus, 'accepted');

      // A moderator flips it to pending server-side; a refresh must reflect it.
      status = 'pending';
      await c.read(myUploadsProvider.notifier).refresh(); // call 2
      verify(upload.listMyScores()).called(2);
      list = c.read(myUploadsProvider).requireValue;
      expect(list.single.proposalStatus, 'pending');
    },
  );

  test('contributedEntry carries the proposal status + rejection reason', () {
    final entry = contributedEntry(
      ContributedScore(
        id: 'x',
        level: PracticeLevel.beginner,
        createdAt: DateTime.utc(2026),
        measureCount: 1,
        timeSig: '4/4',
        keyFifths: 0,
        title: 'T',
        proposalStatus: 'rejected',
        rejectionReason: 'blurry scan',
      ),
    );
    expect(entry.proposalStatus, 'rejected');
    expect(entry.proposalRejectionReason, 'blurry scan');
  });
}
