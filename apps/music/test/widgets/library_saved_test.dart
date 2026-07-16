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
  _FakeCatalog(this.savedHits);
  final List<CatalogHit> savedHits;
  final List<String> removed = [];

  @override
  Future<List<CatalogHit>> listSaved() async =>
      savedHits.where((h) => !removed.contains(h.id)).toList();

  @override
  Future<void> remove(String catalogId) async => removed.add(catalogId);

  @override
  Future<void> save(String catalogId) async {}

  @override
  Future<CatalogSearchPage> search({
    String query = '',
    String? author,
    PracticeLevel? level,
    int limit = 20,
    int offset = 0,
  }) async => const CatalogSearchPage(hits: [], nextOffset: 0);

  @override
  Future<Uint8List> fetchBytes(String catalogId) async => Uint8List(0);
}

class _FakeUpload implements ScoreUploadService {
  @override
  Future<List<ContributedScore>> listMyScores() async => const [];
  @override
  Future<void> deleteScore(String id) async {}
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

CatalogHit _saved() => const CatalogHit(
  id: 'c1',
  title: 'Saved Piece',
  composer: 'Composer A',
  level: PracticeLevel.beginner,
  license: 'CC-BY-4.0',
  source: 'pdmx',
);

ProviderContainer _container(_FakeCatalog catalog, {bool signedIn = true}) {
  final c = ProviderContainer(
    overrides: [
      scoreCatalogProvider.overrideWithValue(const []),
      scoreAssetSourceProvider.overrideWithValue(FakeScoreAssetSource()),
      notationEngineProvider.overrideWithValue(FakeNotationEngine()),
      midiServiceProvider.overrideWithValue(FakeMidiService()),
      scoreSourceProvider.overrideWithValue(FakeScoreSource()),
      audioServiceProvider.overrideWithValue(RecordingAudioService()),
      scoreUploadServiceProvider.overrideWithValue(_FakeUpload()),
      catalogServiceProvider.overrideWithValue(catalog),
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
  testWidgets('shows the saved section + hub entry point when signed in', (
    tester,
  ) async {
    final c = _container(_FakeCatalog([_saved()]));
    await _pump(tester, c);

    expect(find.text('SAVED FROM THE HUB'), findsOneWidget);
    expect(find.text('Saved Piece'), findsOneWidget);
    // The Score Hub app-bar entry point is present.
    expect(find.byIcon(Icons.search), findsOneWidget);
    await _teardown(tester);
  });

  testWidgets('no saved section or hub entry point when signed out', (
    tester,
  ) async {
    final c = _container(_FakeCatalog([_saved()]), signedIn: false);
    await _pump(tester, c);

    expect(find.text('SAVED FROM THE HUB'), findsNothing);
    expect(find.text('Saved Piece'), findsNothing);
    expect(find.byIcon(Icons.search), findsNothing);
    await _teardown(tester);
  });

  testWidgets('no saved section when the saved list is empty', (tester) async {
    final c = _container(_FakeCatalog(const []));
    await _pump(tester, c);
    expect(find.text('SAVED FROM THE HUB'), findsNothing);
    await _teardown(tester);
  });

  testWidgets('remove-from-library calls remove and drops the tile', (
    tester,
  ) async {
    final catalog = _FakeCatalog([_saved()]);
    final c = _container(catalog);
    await _pump(tester, c);
    expect(find.text('Saved Piece'), findsOneWidget);

    await tester.tap(find.byIcon(Icons.bookmark_remove_outlined));
    for (var i = 0; i < 8; i++) {
      await tester.pump(const Duration(milliseconds: 20));
    }

    expect(catalog.removed, ['c1']);
    expect(find.text('Saved Piece'), findsNothing);
    await _teardown(tester);
  });
}
