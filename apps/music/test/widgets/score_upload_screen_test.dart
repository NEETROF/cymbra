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

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/screens/score_upload_screen.dart';
import 'package:music/services/auth_service.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/file_picker_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/src/rust/api/musicxml.dart' show ValidationOutcome;
import 'package:music/state/score_catalog.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';

class _FakePicker implements FilePickerService {
  _FakePicker(this.next);
  final PickedScoreFile? next;
  @override
  Future<PickedScoreFile?> pickScore() async => next;
}

class _FakeUpload implements ScoreUploadService {
  final List<({PracticeLevel level, RightsBasis basis})> uploads = [];
  Object? uploadError;

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
    uploads.add((level: level, basis: rightsBasis));
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
  Future<void> setFavorite(String id, bool favorite) async {}

  @override
  Future<Uint8List> fetchBytes(String id) async => Uint8List(0);
}

PickedScoreFile _validFile() => PickedScoreFile(
  name: 'song.musicxml',
  bytes: Uint8List.fromList(const [1, 2, 3]),
);

ProviderContainer _container({
  PickedScoreFile? pick,
  _FakeUpload? upload,
  FakeNotationEngine? engine,
}) {
  final c = ProviderContainer(
    overrides: [
      filePickerProvider.overrideWithValue(_FakePicker(pick)),
      notationEngineProvider.overrideWithValue(engine ?? FakeNotationEngine()),
      scoreUploadServiceProvider.overrideWithValue(upload ?? _FakeUpload()),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

Future<void> _pump(WidgetTester tester, ProviderContainer container) async {
  await tester.binding.setSurfaceSize(const Size(900, 1400));
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(const ScoreUploadScreen()),
    ),
  );
}

/// The verify step drives a Ticker, so `pumpAndSettle` never settles — pump a
/// fixed number of frames instead.
Future<void> _pumpFrames(WidgetTester tester, [int n = 8]) async {
  for (var i = 0; i < n; i++) {
    await tester.pump(const Duration(milliseconds: 50));
  }
}

/// Unmounts the tree so the verify step's Ticker is disposed before the test ends.
Future<void> _teardown(WidgetTester tester) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
}

bool _enabled(WidgetTester tester, String label) =>
    tester.widget<TextButton>(find.widgetWithText(TextButton, label)).enabled;

void main() {
  testWidgets('Verify is gated on a validated file AND the rights attestation', (
    tester,
  ) async {
    final container = _container(pick: _validFile());
    await _pump(tester, container);

    // Nothing picked yet: the forward action is present but disabled.
    expect(find.text('Vérifier'), findsOneWidget);
    expect(_enabled(tester, 'Vérifier'), isFalse);

    // Pick + validate.
    await tester.tap(find.text('Choisir un fichier'));
    await _pumpFrames(tester);
    expect(find.textContaining('est valide'), findsOneWidget);
    // A validated file alone is not enough — the attestation is still missing.
    expect(_enabled(tester, 'Vérifier'), isFalse);

    // Declare a basis and tick the confirmation.
    await tester.tap(find.text('Domaine public / licence libre'));
    await tester.pump();
    await tester.tap(find.byType(CheckboxListTile));
    await tester.pump();
    expect(_enabled(tester, 'Vérifier'), isTrue);
    await _teardown(tester);
  });

  testWidgets('full flow: upload → tempo-locked verify → confirm → submit', (
    tester,
  ) async {
    final upload = _FakeUpload();
    final container = _container(pick: _validFile(), upload: upload);
    await _pump(tester, container);

    // Upload step: pick + attest.
    await tester.tap(find.text('Choisir un fichier'));
    await _pumpFrames(tester);
    await tester.tap(find.text('J\'en suis l\'auteur'));
    await tester.pump();
    await tester.tap(find.byType(CheckboxListTile));
    await tester.pump();

    // → Verify: read-only metadata, and NO practice controls (no tempo slider).
    await tester.tap(find.text('Vérifier'));
    await _pumpFrames(tester);
    expect(find.text('Informations détectées (lecture seule)'), findsOneWidget);
    expect(find.byType(Slider), findsNothing);

    // → Confirm: difficulty gate.
    await tester.tap(find.text('Continuer'));
    await _pumpFrames(tester);
    expect(find.text('Niveau de difficulté'), findsOneWidget);
    // 'Envoyer' is disabled until a level is chosen.
    expect(_enabled(tester, 'Envoyer'), isFalse);

    await tester.tap(find.text('Intermediate')); // localized label (en default)
    await tester.pump();
    expect(_enabled(tester, 'Envoyer'), isTrue);

    await tester.tap(find.text('Envoyer'));
    await _pumpFrames(tester);

    // Submitted with the chosen level/basis; success screen shown.
    expect(upload.uploads.single.level, PracticeLevel.intermediate);
    expect(upload.uploads.single.basis, RightsBasis.author);
    expect(find.text('Partition ajoutée à vos contributions.'), findsOneWidget);
    await _teardown(tester);
  });

  testWidgets('the AppBar back arrow steps back through the wizard', (
    tester,
  ) async {
    final container = _container(pick: _validFile());
    await _pump(tester, container);

    // Drive to the confirm step.
    await tester.tap(find.text('Choisir un fichier'));
    await _pumpFrames(tester);
    await tester.tap(find.text('J\'en suis l\'auteur'));
    await tester.pump();
    await tester.tap(find.byType(CheckboxListTile));
    await tester.pump();
    await tester.tap(find.text('Vérifier'));
    await _pumpFrames(tester);
    await tester.tap(find.text('Continuer'));
    await _pumpFrames(tester);
    expect(find.text('Niveau de difficulté'), findsOneWidget);

    // Back: confirm → verify.
    await tester.tap(find.byIcon(Icons.arrow_back));
    await _pumpFrames(tester);
    expect(find.text('Informations détectées (lecture seule)'), findsOneWidget);

    // Back: verify → upload.
    await tester.tap(find.byIcon(Icons.arrow_back));
    await _pumpFrames(tester);
    expect(find.textContaining('Formats acceptés'), findsOneWidget);
    await _teardown(tester);
  });

  testWidgets('a rejected file shows the reject banner and blocks Verify', (
    tester,
  ) async {
    final engine = FakeNotationEngine(
      validateOutcome: ValidationOutcome(rejectCode: 'no_notes'),
    );
    final container = _container(pick: _validFile(), engine: engine);
    await _pump(tester, container);

    await tester.tap(find.text('Choisir un fichier'));
    await _pumpFrames(tester);

    // No "valide" banner, no attestation controls, forward stays disabled.
    expect(find.textContaining('est valide'), findsNothing);
    expect(find.byType(CheckboxListTile), findsNothing);
    expect(_enabled(tester, 'Vérifier'), isFalse);
    await _teardown(tester);
  });

  testWidgets('a submit failure surfaces a friendly banner and keeps inputs', (
    tester,
  ) async {
    final upload = _FakeUpload()
      ..uploadError = const AuthException(
        AuthError.rateLimited,
        'quota exceeded',
      );
    final container = _container(pick: _validFile(), upload: upload);
    await _pump(tester, container);

    await tester.tap(find.text('Choisir un fichier'));
    await _pumpFrames(tester);
    await tester.tap(find.text('J\'en suis l\'auteur'));
    await tester.pump();
    await tester.tap(find.byType(CheckboxListTile));
    await tester.pump();
    await tester.tap(find.text('Vérifier'));
    await _pumpFrames(tester);
    await tester.tap(find.text('Continuer'));
    await _pumpFrames(tester);
    await tester.tap(find.text('Beginner'));
    await tester.pump();
    await tester.tap(find.text('Envoyer'));
    await _pumpFrames(tester);

    // Friendly FR message (never the raw exception); still on the confirm step.
    expect(
      find.textContaining('quota'),
      findsOneWidget,
      reason: 'the rate-limit message mentions the quota',
    );
    expect(find.textContaining('AuthException'), findsNothing);
    expect(find.text('Niveau de difficulté'), findsOneWidget);
    await _teardown(tester);
  });
}
