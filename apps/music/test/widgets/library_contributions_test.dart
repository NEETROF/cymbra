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
import 'package:music/screens/library_screen.dart';
import 'package:music/services/audio_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';

class _FakeUpload implements ScoreUploadService {
  _FakeUpload(this.mine);
  final List<ContributedScore> mine;
  final List<String> deleted = [];

  @override
  Future<List<ContributedScore>> listMyScores() async => mine;

  @override
  Future<void> deleteScore(String id) async => deleted.add(id);

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

  @override
  Future<Uint8List> fetchBytes(String id) async => Uint8List(0);
}

const _bundled = [
  CatalogEntry(
    id: 'b1',
    title: 'Bundled Piece',
    composer: 'Composer A',
    assetPath: 'assets/scores/beginner/b1.musicxml',
    level: PracticeLevel.beginner,
  ),
];

ContributedScore _mine() => ContributedScore(
  id: 'sc-1',
  level: PracticeLevel.intermediate,
  createdAt: DateTime.utc(2026, 3, 4),
  measureCount: 8,
  timeSig: '4/4',
  keyFifths: 0,
  title: 'My Upload',
  composer: 'Me',
);

ProviderContainer _container(_FakeUpload upload, {bool signedIn = true}) {
  final c = ProviderContainer(
    overrides: [
      scoreCatalogProvider.overrideWithValue(_bundled),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(FakeNotationEngine()),
      midiServiceProvider.overrideWithValue(FakeMidiService()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
      scoreUploadServiceProvider.overrideWithValue(upload),
      canUseOnlineServicesProvider.overrideWithValue(signedIn),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

Future<void> _pump(WidgetTester tester, ProviderContainer container) async {
  await tester.binding.setSurfaceSize(const Size(1400, 900));
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: container,
      child: localizedApp(const LibraryScreen()),
    ),
  );
  // Resolve the async `myContributedScoresProvider`.
  for (var i = 0; i < 6; i++) {
    await tester.pump(const Duration(milliseconds: 20));
  }
}

Future<void> _teardown(WidgetTester tester, ProviderContainer container) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
  container.dispose();
}

void main() {
  testWidgets('shows a contributions section with an owner-only delete', (
    tester,
  ) async {
    final upload = _FakeUpload([_mine()]);
    final container = _container(upload);
    await _pump(tester, container);

    expect(find.text('MES CONTRIBUTIONS'), findsOneWidget);
    expect(find.text('My Upload'), findsOneWidget);
    // Owner-only delete on the contributed tile; the bundled tile has none.
    expect(find.byIcon(Icons.delete_outline), findsOneWidget);
    await _teardown(tester, container);
  });

  testWidgets('no contributions section when signed out', (tester) async {
    final upload = _FakeUpload([_mine()]);
    final container = _container(upload, signedIn: false);
    await _pump(tester, container);

    expect(find.text('MES CONTRIBUTIONS'), findsNothing);
    expect(find.text('My Upload'), findsNothing);
    await _teardown(tester, container);
  });

  testWidgets('confirming delete calls deleteScore with the score id', (
    tester,
  ) async {
    final upload = _FakeUpload([_mine()]);
    final container = _container(upload);
    await _pump(tester, container);

    await tester.tap(find.byIcon(Icons.delete_outline));
    await tester.pumpAndSettle();
    expect(find.text('Supprimer cette partition ?'), findsOneWidget);

    await tester.tap(find.widgetWithText(FilledButton, 'Supprimer'));
    await tester.pump();
    await tester.pump();

    expect(upload.deleted, ['sc-1']);
    await _teardown(tester, container);
  });

  testWidgets('cancelling delete does not call deleteScore', (tester) async {
    final upload = _FakeUpload([_mine()]);
    final container = _container(upload);
    await _pump(tester, container);

    await tester.tap(find.byIcon(Icons.delete_outline));
    await tester.pumpAndSettle();
    await tester.tap(find.widgetWithText(TextButton, 'Annuler'));
    await tester.pumpAndSettle();

    expect(upload.deleted, isEmpty);
    await _teardown(tester, container);
  });
}
