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

import 'dart:async';
import 'dart:typed_data';

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/services/auth_service.dart';
import 'package:music/services/file_picker_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/score_upload_notifier.dart';

import '../support/notation_fakes.dart';

/// Upload seam gated on [gate]: the submit stays in flight until the test
/// completes (or fails) it — after the container is disposed.
class _GatedUpload extends Fake implements ScoreUploadService {
  final gate = Completer<ContributedScore>();
  int calls = 0;

  @override
  Future<ContributedScore> upload({
    required Uint8List data,
    required String filename,
    required PracticeLevel level,
    required RightsBasis rightsBasis,
    required bool rightsAck,
    String? fallbackTitle,
    String? fallbackComposer,
  }) {
    calls++;
    return gate.future;
  }
}

class _Picker extends Fake implements FilePickerService {
  @override
  Future<PickedScoreFile?> pickScore() async => PickedScoreFile(
    name: 'x.musicxml',
    bytes: Uint8List.fromList(const [1, 2, 3]),
  );
}

/// Drives the wizard to the ready-to-submit state and starts the submit.
Future<ScoreUploadNotifier> _startSubmit(ProviderContainer c) async {
  final n = c.read(scoreUploadNotifierProvider.notifier);
  await n.pickAndValidate();
  n.setRightsBasis(RightsBasis.publicDomain);
  n.setRightsAck(true);
  n.goToVerify();
  n.goToConfirm();
  n.setLevel(PracticeLevel.beginner);
  unawaited(n.submit());
  await Future<void>.delayed(Duration.zero);
  return n;
}

void main() {
  test(
    'leaving mid-upload: a late success writes nothing and throws nothing',
    () async {
      // Design D11: leaving the screen disposes the (autoDispose) notifier. This
      // pins the CONTRACT (late completion → no write, no throw, no message).
      // Verified by mutation: Riverpod 2.6.1 happens to drop the unguarded write
      // silently, so today the explicit `_disposed` guard is belt-and-braces —
      // but 3.x throws on post-dispose writes, so this same test is what catches
      // a missing guard after the migration (migrate-riverpod-3).
      final upload = _GatedUpload();
      final c = ProviderContainer(
        overrides: [
          filePickerProvider.overrideWithValue(_Picker()),
          notationEngineProvider.overrideWithValue(FakeNotationEngine()),
          scoreUploadServiceProvider.overrideWithValue(upload),
        ],
      );
      await _startSubmit(c);
      expect(upload.calls, 1, reason: 'the submit is in flight');

      // The user leaves the upload flow.
      c.dispose();

      // …and the upload lands anyway. No state write, no throw, no message —
      // MyUploads reports the truth on its next refresh.
      upload.gate.complete(
        ContributedScore(
          id: 'landed',
          level: PracticeLevel.beginner,
          createdAt: DateTime.utc(2026),
          measureCount: 4,
          timeSig: '4/4',
          keyFifths: 0,
          title: 'Sample',
        ),
      );
      await Future<void>.delayed(Duration.zero);
    },
  );

  test('leaving mid-upload: a late failure is equally silent', () async {
    final upload = _GatedUpload();
    final c = ProviderContainer(
      overrides: [
        filePickerProvider.overrideWithValue(_Picker()),
        notationEngineProvider.overrideWithValue(FakeNotationEngine()),
        scoreUploadServiceProvider.overrideWithValue(upload),
      ],
    );
    await _startSubmit(c);
    c.dispose();

    upload.gate.completeError(AuthException(AuthError.unavailable));
    await Future<void>.delayed(Duration.zero);
  });

  test('AlreadyExists reads as a fact, not a failure', () {
    // After an abandoned upload that landed, re-submitting is the EXPECTED
    // path: the server dedups on (owner, sha256), so the answer is a statement
    // about the library, not an error report.
    final msg = uploadErrorMessage(AuthException(AuthError.alreadyExists));
    expect(msg, contains('déjà dans votre bibliothèque'));
  });
}
