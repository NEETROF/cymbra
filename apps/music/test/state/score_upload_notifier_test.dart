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
import 'package:music/services/file_picker_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/src/rust/api/musicxml.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/score_upload_notifier.dart';

import '../support/notation_fakes.dart';

class _FakePicker implements FilePickerService {
  _FakePicker(this.next);
  final PickedScoreFile? next;
  @override
  Future<PickedScoreFile?> pickScore() async => next;
}

class _FakeUpload implements ScoreUploadService {
  Object? uploadError;
  final List<({PracticeLevel level, RightsBasis basis, bool ack, int len})>
  uploads = [];

  @override
  Future<ContributedScore> upload({
    required Uint8List data,
    required String filename,
    required PracticeLevel level,
    required RightsBasis rightsBasis,
    required bool rightsAck,
    String? fallbackTitle,
    String? fallbackComposer,
  }) async {
    uploads.add((
      level: level,
      basis: rightsBasis,
      ack: rightsAck,
      len: data.length,
    ));
    if (uploadError != null) throw uploadError!;
    return ContributedScore(
      id: 'new-id',
      level: level,
      createdAt: DateTime.utc(2026),
      measureCount: 4,
      timeSig: '4/4',
      keyFifths: 0,
      title: 'Sample',
    );
  }

  @override
  Future<List<ContributedScore>> listMyScores() async => const [];
  @override
  Future<void> deleteScore(String id) async {}
  @override
  Future<Uint8List> fetchBytes(String id) async => Uint8List(0);
}

PickedScoreFile _file() =>
    PickedScoreFile(name: 'x.musicxml', bytes: Uint8List.fromList(const [1, 2, 3]));

ProviderContainer _make({
  PickedScoreFile? pick,
  NotationEngine? engine,
  _FakeUpload? upload,
}) {
  final c = ProviderContainer(
    overrides: [
      filePickerProvider.overrideWithValue(_FakePicker(pick)),
      notationEngineProvider.overrideWithValue(engine ?? FakeNotationEngine()),
      scoreUploadServiceProvider.overrideWithValue(upload ?? _FakeUpload()),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

void main() {
  test('pickAndValidate populates the server-parity summary on a valid file',
      () async {
    final c = _make(pick: _file());
    final n = c.read(scoreUploadNotifierProvider.notifier);
    await n.pickAndValidate();
    final s = c.read(scoreUploadNotifierProvider);
    expect(s.isValidated, isTrue);
    expect(s.summary?.title, 'Sample');
    expect(s.validating, isFalse);
  });

  test('a cancelled pick is a no-op', () async {
    final c = _make(pick: null);
    await c.read(scoreUploadNotifierProvider.notifier).pickAndValidate();
    expect(c.read(scoreUploadNotifierProvider).file, isNull);
  });

  test('a rejected file surfaces the typed code and is not validated', () async {
    final engine = FakeNotationEngine(
      validateOutcome: ValidationOutcome(rejectCode: 'no_notes'),
    );
    final c = _make(pick: _file(), engine: engine);
    final n = c.read(scoreUploadNotifierProvider.notifier);
    await n.pickAndValidate();
    final s = c.read(scoreUploadNotifierProvider);
    expect(s.isValidated, isFalse);
    expect(s.rejectCode, 'no_notes');
  });

  test('Verify is gated on validation AND the rights attestation', () async {
    final c = _make(pick: _file());
    final n = c.read(scoreUploadNotifierProvider.notifier);
    await n.pickAndValidate();
    n.goToVerify();
    expect(c.read(scoreUploadNotifierProvider).step, UploadStep.upload);
    n.setRightsBasis(RightsBasis.author);
    n.setRightsAck(true);
    n.goToVerify();
    expect(c.read(scoreUploadNotifierProvider).step, UploadStep.verify);
  });

  test('finalize is gated on difficulty and submits the right inputs', () async {
    final upload = _FakeUpload();
    final c = _make(pick: _file(), upload: upload);
    final n = c.read(scoreUploadNotifierProvider.notifier);
    await n.pickAndValidate();
    n.setRightsBasis(RightsBasis.publicDomain);
    n.setRightsAck(true);
    n.goToVerify();
    n.goToConfirm();
    await n.submit(); // no level yet → blocked
    expect(upload.uploads, isEmpty);
    n.setLevel(PracticeLevel.intermediate);
    await n.submit();
    expect(upload.uploads.single.level, PracticeLevel.intermediate);
    expect(upload.uploads.single.basis, RightsBasis.publicDomain);
    expect(upload.uploads.single.ack, isTrue);
    expect(c.read(scoreUploadNotifierProvider).isDone, isTrue);
  });

  test('a submit error is surfaced and the user inputs are kept', () async {
    final upload = _FakeUpload()..uploadError = Exception('server said no');
    final c = _make(pick: _file(), upload: upload);
    final n = c.read(scoreUploadNotifierProvider.notifier);
    await n.pickAndValidate();
    n.setRightsBasis(RightsBasis.author);
    n.setRightsAck(true);
    n.setLevel(PracticeLevel.beginner);
    await n.submit();
    final s = c.read(scoreUploadNotifierProvider);
    expect(s.submitError, contains('server said no'));
    expect(s.isDone, isFalse);
    expect(s.level, PracticeLevel.beginner);
  });
}
