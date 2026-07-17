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
import 'package:music/services/catalog_service.dart';
import 'package:music/services/midi_service.dart';
import 'package:music/services/notation_engine.dart';
import 'package:music/services/score_asset_source.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';

import '../support/fakes.dart';
import '../support/localized.dart';
import '../support/notation_fakes.dart';

class _FakeCatalog implements CatalogService {
  _FakeCatalog(this.saved);
  final List<CatalogHit> saved;
  final List<String> removed = [];

  @override
  Future<List<CatalogHit>> listSaved() async =>
      saved.where((h) => !removed.contains(h.id)).toList();
  @override
  Future<void> remove(String catalogId) async => removed.add(catalogId);
  @override
  Future<void> save(String catalogId) async {}
  @override
  Future<CatalogSearchPage> search({
    String query = '',
    String? author,
    PracticeLevel? level,
    CatalogFilters filters = const CatalogFilters(),
    int limit = 20,
    int offset = 0,
  }) async => const CatalogSearchPage(hits: [], nextOffset: 0);
  @override
  Future<Uint8List> fetchBytes(String catalogId) async => Uint8List(0);
}

class _FakeUpload implements ScoreUploadService {
  _FakeUpload(this.mine);
  final List<ContributedScore> mine;
  final List<String> favoritedOff = [];

  @override
  Future<List<ContributedScore>> listMyScores() async => mine;
  @override
  Future<void> deleteScore(String id) async {}
  @override
  Future<void> setFavorite(String id, bool favorite) async {
    if (!favorite) favoritedOff.add(id);
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

CatalogHit _saved(String id, String title) => CatalogHit(
  id: id,
  title: title,
  composer: 'Composer',
  level: PracticeLevel.beginner,
  license: 'CC-BY-4.0',
  source: 'pdmx',
);

ContributedScore _upload(String id, String title, {bool favorite = true}) =>
    ContributedScore(
      id: id,
      level: PracticeLevel.intermediate,
      createdAt: DateTime.utc(2026, 5, 1),
      measureCount: 8,
      timeSig: '4/4',
      keyFifths: 0,
      title: title,
      composer: 'Me',
      favorite: favorite,
    );

const _bundled = [
  CatalogEntry(
    id: 'b1',
    title: 'Bundled Piece',
    composer: 'X',
    assetPath: 'assets/scores/beginner/b1.musicxml',
    level: PracticeLevel.beginner,
  ),
];

ProviderContainer _container(
  _FakeCatalog catalog,
  _FakeUpload upload, {
  bool signedIn = true,
}) {
  final c = ProviderContainer(
    overrides: [
      scoreCatalogProvider.overrideWithValue(_bundled),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(FakeNotationEngine()),
      midiServiceProvider.overrideWithValue(FakeMidiService()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
      catalogServiceProvider.overrideWithValue(catalog),
      scoreUploadServiceProvider.overrideWithValue(upload),
      canUseOnlineServicesProvider.overrideWithValue(signedIn),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

Future<void> _pump(WidgetTester tester, ProviderContainer c) async {
  await tester.binding.setSurfaceSize(const Size(1400, 900));
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: c,
      child: localizedApp(const LibraryScreen(), locale: const Locale('en')),
    ),
  );
  for (var i = 0; i < 8; i++) {
    await tester.pump(const Duration(milliseconds: 20));
  }
}

Future<void> _teardown(WidgetTester tester) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
}

void main() {
  testWidgets('signed in: favorites by level, no bundled, no non-favorites', (
    tester,
  ) async {
    final c = _container(
      _FakeCatalog([_saved('c1', 'Saved Piece')]),
      _FakeUpload([
        _upload('u1', 'Fav Upload'),
        _upload('u2', 'Hidden Upload', favorite: false),
      ]),
    );
    await _pump(tester, c);

    expect(find.text('Saved Piece'), findsOneWidget); // saved catalog
    expect(find.text('Fav Upload'), findsOneWidget); // favorited upload
    expect(find.text('Hidden Upload'), findsNothing); // not favorited
    expect(
      find.text('Bundled Piece'),
      findsNothing,
    ); // no bundled when signed in
    // Level headers (beginner for the saved, intermediate for the upload).
    expect(find.text('Beginner'), findsOneWidget);
    expect(find.text('Intermediate'), findsOneWidget);
    await _teardown(tester);
  });

  testWidgets('signed in with no favorites shows the hub call-to-action', (
    tester,
  ) async {
    final c = _container(_FakeCatalog(const []), _FakeUpload(const []));
    await _pump(tester, c);

    expect(find.text('No favorites yet'), findsOneWidget);
    expect(
      find.widgetWithText(FilledButton, 'Browse the Score Hub'),
      findsOneWidget,
    );
    await _teardown(tester);
  });

  testWidgets('signed out shows the bundled demo catalog', (tester) async {
    final c = _container(
      _FakeCatalog(const []),
      _FakeUpload(const []),
      signedIn: false,
    );
    await _pump(tester, c);
    expect(find.text('Bundled Piece'), findsOneWidget);
    await _teardown(tester);
  });

  testWidgets('heart un-favorites an upload (keeps it)', (tester) async {
    final upload = _FakeUpload([_upload('u1', 'Fav Upload')]);
    final c = _container(_FakeCatalog(const []), upload);
    await _pump(tester, c);
    expect(find.text('Fav Upload'), findsOneWidget);

    await tester.tap(find.byIcon(Icons.favorite));
    for (var i = 0; i < 8; i++) {
      await tester.pump(const Duration(milliseconds: 20));
    }
    expect(upload.favoritedOff, ['u1']);
    await _teardown(tester);
  });

  testWidgets('heart removes a saved catalog score from the library', (
    tester,
  ) async {
    final catalog = _FakeCatalog([_saved('c1', 'Saved Piece')]);
    final c = _container(catalog, _FakeUpload(const []));
    await _pump(tester, c);
    expect(find.text('Saved Piece'), findsOneWidget);

    await tester.tap(find.byIcon(Icons.favorite));
    for (var i = 0; i < 8; i++) {
      await tester.pump(const Duration(milliseconds: 20));
    }
    expect(catalog.removed, ['c1']);
    expect(find.text('Saved Piece'), findsNothing);
    await _teardown(tester);
  });
}
