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
import 'package:mockito/annotations.dart';
import 'package:mockito/mockito.dart';
import 'package:music/services/auth_service.dart';
import 'package:music/services/file_picker_service.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/state/batch_import_notifier.dart';
import 'package:music/state/score_catalog.dart';

@GenerateNiceMocks([MockSpec<ScoreUploadService>()])
import 'batch_import_notifier_test.mocks.dart';

PickedScoreFile _file(String name) =>
    PickedScoreFile(name: name, bytes: Uint8List.fromList(const [1, 2, 3]));

ContributedScore _record(String id) => ContributedScore(
  id: id,
  level: PracticeLevel.beginner,
  createdAt: DateTime.utc(2026),
  measureCount: 4,
  timeSig: '4/4',
  keyFifths: 0,
  title: id,
);

/// An upload service whose `upload()` resolves per filename: a name present in
/// [failures] throws that error, anything else succeeds.
MockScoreUploadService _service({
  Map<String, Object> failures = const {},
  UploadAllowance? allowance,
  Object? allowanceError,
}) {
  final s = MockScoreUploadService();
  when(
    s.upload(
      data: anyNamed('data'),
      filename: anyNamed('filename'),
      level: anyNamed('level'),
      rightsBasis: anyNamed('rightsBasis'),
      rightsAck: anyNamed('rightsAck'),
      fallbackTitle: anyNamed('fallbackTitle'),
      fallbackComposer: anyNamed('fallbackComposer'),
    ),
  ).thenAnswer((inv) async {
    final name = inv.namedArguments[#filename] as String;
    final failure = failures[name];
    if (failure != null) throw failure;
    return _record(name);
  });
  if (allowanceError != null) {
    when(s.uploadAllowance()).thenThrow(allowanceError);
  } else if (allowance != null) {
    when(s.uploadAllowance()).thenAnswer((_) async => allowance);
  }
  return s;
}

ProviderContainer _make(MockScoreUploadService service) {
  final c = ProviderContainer(
    overrides: [scoreUploadServiceProvider.overrideWithValue(service)],
  );
  addTearDown(c.dispose);
  return c;
}

/// Drive a batch to completion with one attestation + one difficulty.
Future<BatchImportState> _run(
  ProviderContainer c,
  List<PickedScoreFile> files, {
  RightsBasis basis = RightsBasis.author,
  PracticeLevel level = PracticeLevel.beginner,
}) async {
  final n = c.read(batchImportNotifierProvider.notifier);
  n.setFiles(files);
  n.setRightsBasis(basis);
  n.setRightsAck(true);
  n.setLevel(level);
  await n.run();
  return c.read(batchImportNotifierProvider);
}

void main() {
  test('the batch cannot start until attestation and difficulty are set', () async {
    final service = _service();
    final c = _make(service);
    final n = c.read(batchImportNotifierProvider.notifier);
    n.setFiles([_file('a.musicxml')]);
    expect(c.read(batchImportNotifierProvider).canStart, isFalse);

    n.setRightsBasis(RightsBasis.author);
    expect(c.read(batchImportNotifierProvider).canStart, isFalse, reason: 'no ack');
    n.setRightsAck(true);
    expect(c.read(batchImportNotifierProvider).canStart, isFalse, reason: 'no level');
    n.setLevel(PracticeLevel.beginner);
    expect(c.read(batchImportNotifierProvider).canStart, isTrue);

    // Running before it is startable uploads nothing.
    final blocked = _make(_service());
    await blocked.read(batchImportNotifierProvider.notifier).run();
    expect(blocked.read(batchImportNotifierProvider).results, isEmpty);
  });

  test('the batch choices are applied to every file', () async {
    final service = _service();
    final c = _make(service);
    await _run(
      c,
      [_file('a.musicxml'), _file('b.musicxml')],
      basis: RightsBasis.privateUse,
      level: PracticeLevel.advanced,
    );
    final captured = verify(
      service.upload(
        data: anyNamed('data'),
        filename: anyNamed('filename'),
        level: captureAnyNamed('level'),
        rightsBasis: captureAnyNamed('rightsBasis'),
        rightsAck: captureAnyNamed('rightsAck'),
        fallbackTitle: anyNamed('fallbackTitle'),
        fallbackComposer: anyNamed('fallbackComposer'),
      ),
    ).captured;
    // Two calls × three captured args, all carrying the same batch choices.
    expect(captured, [
      PracticeLevel.advanced,
      RightsBasis.privateUse,
      true,
      PracticeLevel.advanced,
      RightsBasis.privateUse,
      true,
    ]);
  });

  test('a failing file is recorded and the batch continues', () async {
    final service = _service(
      failures: {
        'dup.musicxml': AuthException(AuthError.alreadyExists),
        'bad.musicxml': AuthException(AuthError.invalidArgument),
        'over.musicxml': AuthException(AuthError.rateLimited),
        'boom.musicxml': StateError('transport'),
      },
    );
    final c = _make(service);
    final state = await _run(c, [
      _file('ok1.musicxml'),
      _file('dup.musicxml'),
      _file('bad.musicxml'),
      _file('over.musicxml'),
      _file('boom.musicxml'),
      _file('ok2.musicxml'),
    ]);

    expect(state.done, isTrue);
    expect(state.running, isFalse);
    expect(
      state.results.map((r) => r.outcome).toList(),
      const [
        BatchOutcome.imported,
        BatchOutcome.duplicate,
        BatchOutcome.invalid,
        BatchOutcome.quotaExceeded,
        BatchOutcome.failed,
        BatchOutcome.imported,
      ],
    );
    // Every file was attempted — the last one especially, after four failures.
    expect(state.results.length, 6);
    expect(state.importedCount, 2);
  });

  test('a quota-exceeded file does not stop the ones after it', () async {
    // The free plan refuses mid-run; the remaining files still get their turn
    // (they may be refused too, but each on its own account).
    final service = _service(
      failures: {'c.musicxml': AuthException(AuthError.rateLimited)},
    );
    final c = _make(service);
    final state = await _run(c, [
      _file('a.musicxml'),
      _file('b.musicxml'),
      _file('c.musicxml'),
      _file('d.musicxml'),
    ]);
    expect(state.results.last.name, 'd.musicxml');
    expect(state.results.last.outcome, BatchOutcome.imported);
  });

  group('quota pre-check', () {
    test('a selection larger than the allowance is flagged before any upload',
        () async {
      final service = _service(
        allowance: const UploadAllowance(
          remaining: 2,
          max: 5,
          windowDays: 7,
          upgradeRaisesLimit: true,
        ),
      );
      final c = _make(service);
      final n = c.read(batchImportNotifierProvider.notifier);
      n.setFiles([
        _file('a.musicxml'),
        _file('b.musicxml'),
        _file('c.musicxml'),
      ]);
      await n.loadAllowance();

      final state = c.read(batchImportNotifierProvider);
      expect(state.exceedsAllowance, isTrue);
      expect(state.overAllowanceCount, 1);
      expect(state.allowance!.upgradeRaisesLimit, isTrue);
      // Nothing has been uploaded to discover this.
      verifyNever(
        service.upload(
          data: anyNamed('data'),
          filename: anyNamed('filename'),
          level: anyNamed('level'),
          rightsBasis: anyNamed('rightsBasis'),
          rightsAck: anyNamed('rightsAck'),
          fallbackTitle: anyNamed('fallbackTitle'),
          fallbackComposer: anyNamed('fallbackComposer'),
        ),
      );
    });

    test('a selection that fits raises no warning', () async {
      final service = _service(
        allowance: const UploadAllowance(
          remaining: 5,
          max: 5,
          windowDays: 7,
          upgradeRaisesLimit: true,
        ),
      );
      final c = _make(service);
      final n = c.read(batchImportNotifierProvider.notifier);
      n.setFiles([_file('a.musicxml')]);
      await n.loadAllowance();
      expect(c.read(batchImportNotifierProvider).exceedsAllowance, isFalse);
      expect(c.read(batchImportNotifierProvider).overAllowanceCount, 0);
    });

    test('an unreadable allowance blocks nothing and warns nothing', () async {
      final service = _service(allowanceError: StateError('offline'));
      final c = _make(service);
      final n = c.read(batchImportNotifierProvider.notifier);
      n.setFiles([_file('a.musicxml')]);
      await n.loadAllowance();
      final state = c.read(batchImportNotifierProvider);
      expect(state.allowance, isNull);
      expect(state.exceedsAllowance, isFalse, reason: 'unknown ⇒ no warning');
    });
  });
}
