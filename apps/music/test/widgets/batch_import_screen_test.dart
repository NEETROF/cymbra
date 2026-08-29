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
import 'package:flutter_localizations/flutter_localizations.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:flutter_test/flutter_test.dart';
import 'package:music/l10n/gen/app_localizations.dart';
import 'package:music/screens/batch_import_screen.dart';
import 'package:music/services/auth_service.dart';
import 'package:music/services/file_picker_service.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/state/batch_import_notifier.dart';
import 'package:music/state/score_catalog.dart';

/// A hand fake (not mockito) because the batch drives it many times with
/// per-filename behaviour, which reads better as data than as stubs.
class _FakeUpload implements ScoreUploadService {
  _FakeUpload({this.failures = const {}, this.allowance});

  final Map<String, Object> failures;
  final UploadAllowance? allowance;

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
    final failure = failures[filename];
    if (failure != null) throw failure;
    return ContributedScore(
      id: filename,
      level: level,
      createdAt: DateTime.utc(2026),
      measureCount: 4,
      timeSig: '4/4',
      keyFifths: 0,
      title: filename,
    );
  }

  @override
  Future<UploadAllowance> uploadAllowance() async =>
      allowance ??
      const UploadAllowance(
        remaining: 100,
        max: 100,
        windowDays: 7,
        upgradeRaisesLimit: false,
      );

  @override
  Future<List<ContributedScore>> listMyScores() async => const [];

  @override
  Future<void> propose({
    required String scoreId,
    required String license,
    required bool attestation,
    String attribution = '',
    String? resubmissionNote,
  }) async {}

  @override
  Future<void> deleteScore(String id) async {}

  @override
  Future<void> setFavorite(String id, bool favorite) async {}

  @override
  Future<ScoreBytesResult> fetchScoreBytes(String id, {String? ifNoneMatch}) =>
      throw UnimplementedError();
}

/// The setup form is taller than the test viewport, so the action button is not
/// built until it scrolls into range.
Future<void> _scrollToImport(WidgetTester tester) => tester.scrollUntilVisible(
  find.widgetWithText(FilledButton, 'Import'),
  200,
  scrollable: find.byType(Scrollable).first,
);

PickedScoreFile _file(String name) =>
    PickedScoreFile(name: name, bytes: Uint8List.fromList(const [1, 2, 3]));

Future<ProviderContainer> _pump(
  WidgetTester tester, {
  required List<PickedScoreFile> files,
  _FakeUpload? upload,
}) async {
  final container = ProviderContainer(
    overrides: [
      scoreUploadServiceProvider.overrideWithValue(upload ?? _FakeUpload()),
    ],
  );
  addTearDown(container.dispose);
  container.read(batchImportNotifierProvider.notifier).setFiles(files);
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: const MaterialApp(
        locale: Locale('en'),
        localizationsDelegates: [
          AppLocalizations.delegate,
          GlobalMaterialLocalizations.delegate,
          GlobalWidgetsLocalizations.delegate,
          GlobalCupertinoLocalizations.delegate,
        ],
        supportedLocales: AppLocalizations.supportedLocales,
        home: BatchImportScreen(),
      ),
    ),
  );
  await tester.pumpAndSettle();
  return container;
}

void main() {
  testWidgets('the import button stays disabled until the batch is attested', (
    tester,
  ) async {
    final c = await _pump(tester, files: [_file('a.musicxml')]);
    // The form is longer than the viewport; the action sits at its end.
    await _scrollToImport(tester);
    FilledButton importButton() => tester.widget<FilledButton>(
      find.widgetWithText(FilledButton, 'Import'),
    );
    expect(importButton().onPressed, isNull);

    final n = c.read(batchImportNotifierProvider.notifier);
    n.setRightsBasis(RightsBasis.author);
    n.setRightsAck(true);
    await tester.pumpAndSettle();
    expect(importButton().onPressed, isNull, reason: 'no difficulty yet');

    n.setLevel(PracticeLevel.beginner);
    await tester.pumpAndSettle();
    await _scrollToImport(tester);
    expect(importButton().onPressed, isNotNull);
  });

  testWidgets('a mixed batch shows one localized outcome per file', (
    tester,
  ) async {
    final upload = _FakeUpload(
      failures: {
        'dup.musicxml': AuthException(AuthError.alreadyExists),
        'bad.musicxml': AuthException(AuthError.invalidArgument),
        'over.musicxml': AuthException(AuthError.rateLimited),
        'boom.musicxml': StateError('transport'),
      },
    );
    final c = await _pump(
      tester,
      files: [
        _file('ok.musicxml'),
        _file('dup.musicxml'),
        _file('bad.musicxml'),
        _file('over.musicxml'),
        _file('boom.musicxml'),
      ],
      upload: upload,
    );
    final n = c.read(batchImportNotifierProvider.notifier);
    n.setRightsBasis(RightsBasis.privateUse);
    n.setRightsAck(true);
    n.setLevel(PracticeLevel.beginner);
    await tester.pumpAndSettle();
    await _scrollToImport(tester);
    await tester.tap(find.widgetWithText(FilledButton, 'Import'));
    await tester.pumpAndSettle();

    // The summary counts only what actually landed.
    expect(find.text('1 of 5 imported'), findsOneWidget);
    // Every file is listed with its own localized, non-technical outcome.
    expect(find.text('Imported'), findsOneWidget);
    expect(find.text('Already in your library'), findsOneWidget);
    expect(find.text('Unreadable score file'), findsOneWidget);
    expect(find.text('Import quota reached'), findsOneWidget);
    expect(find.text('Could not be imported — try again'), findsOneWidget);
    // No raw exception text ever reaches the UI.
    expect(find.textContaining('StateError'), findsNothing);
  });

  testWidgets('a selection larger than the allowance warns before importing', (
    tester,
  ) async {
    final upload = _FakeUpload(
      allowance: const UploadAllowance(
        remaining: 2,
        max: 5,
        windowDays: 7,
        upgradeRaisesLimit: true,
      ),
    );
    await _pump(
      tester,
      files: [_file('a.musicxml'), _file('b.musicxml'), _file('c.musicxml')],
      upload: upload,
    );
    expect(
      find.text(
        'Your plan allows 2 more imports: 1 of these files cannot be imported now.',
      ),
      findsOneWidget,
    );
    expect(find.text('A higher plan raises this limit.'), findsOneWidget);
  });
}
