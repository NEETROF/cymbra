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
import 'package:music/services/notation_engine.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/src/rust/api/musicxml.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/score_upload_notifier.dart';

import '../support/notation_fakes.dart';

@GenerateNiceMocks([
  MockSpec<FilePickerService>(),
  MockSpec<ScoreUploadService>(),
])
import 'score_upload_notifier_test.mocks.dart';

/// A picker stubbed to return [next] once.
MockFilePickerService _picker(PickedScoreFile? next) {
  final p = MockFilePickerService();
  when(p.pickScore()).thenAnswer((_) async => next);
  return p;
}

/// An upload service whose `upload()` echoes the requested level back (matching
/// the server), unless [error] is set — then it throws it.
MockScoreUploadService _upload({Object? error}) {
  final u = MockScoreUploadService();
  when(
    u.upload(
      data: anyNamed('data'),
      filename: anyNamed('filename'),
      level: anyNamed('level'),
      rightsBasis: anyNamed('rightsBasis'),
      rightsAck: anyNamed('rightsAck'),
      fallbackTitle: anyNamed('fallbackTitle'),
      fallbackComposer: anyNamed('fallbackComposer'),
    ),
  ).thenAnswer((inv) async {
    if (error != null) throw error;
    return ContributedScore(
      id: 'new-id',
      level: inv.namedArguments[#level] as PracticeLevel,
      createdAt: DateTime.utc(2026),
      measureCount: 4,
      timeSig: '4/4',
      keyFifths: 0,
      title: 'Sample',
    );
  });
  return u;
}

/// The whole-argument matcher for "any upload call" — for `verifyNever`.
Future<ContributedScore> _anyUpload(MockScoreUploadService u) => u.upload(
  data: anyNamed('data'),
  filename: anyNamed('filename'),
  level: anyNamed('level'),
  rightsBasis: anyNamed('rightsBasis'),
  rightsAck: anyNamed('rightsAck'),
  fallbackTitle: anyNamed('fallbackTitle'),
  fallbackComposer: anyNamed('fallbackComposer'),
);

/// Captures `(level, rightsBasis, rightsAck)` of the single upload call.
List<Object?> _capturedInputs(MockScoreUploadService u) => verify(
  u.upload(
    data: anyNamed('data'),
    filename: anyNamed('filename'),
    level: captureAnyNamed('level'),
    rightsBasis: captureAnyNamed('rightsBasis'),
    rightsAck: captureAnyNamed('rightsAck'),
    fallbackTitle: anyNamed('fallbackTitle'),
    fallbackComposer: anyNamed('fallbackComposer'),
  ),
).captured;

PickedScoreFile _file() => PickedScoreFile(
  name: 'x.musicxml',
  bytes: Uint8List.fromList(const [1, 2, 3]),
);

ProviderContainer _make({
  PickedScoreFile? pick,
  NotationEngine? engine,
  MockScoreUploadService? upload,
}) {
  final c = ProviderContainer(
    overrides: [
      filePickerProvider.overrideWithValue(_picker(pick)),
      notationEngineProvider.overrideWithValue(engine ?? FakeNotationEngine()),
      scoreUploadServiceProvider.overrideWithValue(upload ?? _upload()),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

void main() {
  test(
    'pickAndValidate populates the server-parity summary on a valid file',
    () async {
      final c = _make(pick: _file());
      final n = c.read(scoreUploadNotifierProvider.notifier);
      await n.pickAndValidate();
      final s = c.read(scoreUploadNotifierProvider);
      expect(s.isValidated, isTrue);
      expect(s.summary?.title, 'Sample');
      expect(s.validating, isFalse);
    },
  );

  test('a cancelled pick is a no-op', () async {
    final c = _make(pick: null);
    await c.read(scoreUploadNotifierProvider.notifier).pickAndValidate();
    expect(c.read(scoreUploadNotifierProvider).file, isNull);
  });

  test(
    'a rejected file surfaces the typed code and is not validated',
    () async {
      final engine = FakeNotationEngine(
        validateOutcome: ValidationOutcome(rejectCode: 'no_notes'),
      );
      final c = _make(pick: _file(), engine: engine);
      final n = c.read(scoreUploadNotifierProvider.notifier);
      await n.pickAndValidate();
      final s = c.read(scoreUploadNotifierProvider);
      expect(s.isValidated, isFalse);
      expect(s.rejectCode, 'no_notes');
    },
  );

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

  test(
    'finalize is gated on difficulty and submits the right inputs',
    () async {
      final upload = _upload();
      final c = _make(pick: _file(), upload: upload);
      final n = c.read(scoreUploadNotifierProvider.notifier);
      await n.pickAndValidate();
      n.setRightsBasis(RightsBasis.publicDomain);
      n.setRightsAck(true);
      n.goToVerify();
      n.goToConfirm();
      await n.submit(); // no level yet → blocked
      verifyNever(_anyUpload(upload));
      n.setLevel(PracticeLevel.intermediate);
      await n.submit();
      expect(_capturedInputs(upload), [
        PracticeLevel.intermediate,
        RightsBasis.publicDomain,
        true,
      ]);
      expect(c.read(scoreUploadNotifierProvider).isDone, isTrue);
    },
  );

  test(
    'finalize is blocked without a title until a fallback is typed',
    () async {
      // A file with no parsed title.
      final engine = FakeNotationEngine(
        validateOutcome: const ValidationOutcome(
          summary: ScoreSummary(
            title: null,
            composer: null,
            titleNorm: null,
            workKey: '::',
            isPiano: true,
            instrument: InstrumentKind.keyboard,
            staves: 2,
            keyFifths: 0,
            timeSig: '4/4',
            measureCount: 4,
            noteCount: 8,
          ),
        ),
      );
      final upload = _upload();
      final c = _make(pick: _file(), engine: engine, upload: upload);
      final n = c.read(scoreUploadNotifierProvider.notifier);
      await n.pickAndValidate();
      n.setRightsBasis(RightsBasis.author);
      n.setRightsAck(true);
      n.setLevel(PracticeLevel.advanced);

      // A difficulty alone is not enough — no title yet.
      expect(c.read(scoreUploadNotifierProvider).canFinalize, isFalse);
      await n.submit();
      verifyNever(_anyUpload(upload));

      // Whitespace is not a title.
      n.setFallbackTitle('   ');
      expect(c.read(scoreUploadNotifierProvider).canFinalize, isFalse);

      // A real fallback title unblocks the upload.
      n.setFallbackTitle('My Piece');
      expect(c.read(scoreUploadNotifierProvider).canFinalize, isTrue);
      await n.submit();
      expect(_capturedInputs(upload).first, PracticeLevel.advanced);
      expect(c.read(scoreUploadNotifierProvider).isDone, isTrue);
    },
  );

  test(
    'a submit error is surfaced as a friendly message, inputs kept',
    () async {
      final upload = _upload(
        error: const AuthException(
          AuthError.alreadyExists,
          'score already uploaded',
        ),
      );
      final c = _make(pick: _file(), upload: upload);
      final n = c.read(scoreUploadNotifierProvider.notifier);
      await n.pickAndValidate();
      n.setRightsBasis(RightsBasis.author);
      n.setRightsAck(true);
      n.setLevel(PracticeLevel.beginner);
      await n.submit();
      final s = c.read(scoreUploadNotifierProvider);
      // Mapped to a clean FR message — the raw exception is never shown. Worded
      // as a fact, not a failure (change: add-client-transport-deadlines):
      // retrying after an abandoned upload is the expected path.
      expect(
        s.submitError,
        'Cette partition est déjà dans votre bibliothèque.',
      );
      expect(s.submitError, isNot(contains('AuthException')));
      expect(s.isDone, isFalse);
      expect(s.level, PracticeLevel.beginner);
    },
  );
}
