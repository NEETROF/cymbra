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
import 'package:music/screens/score_hub_screen.dart';
import 'package:music/services/auth_service.dart';
import 'package:music/services/catalog_service.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';

import '../support/localized.dart';

class _FakeCatalog implements CatalogService {
  _FakeCatalog(this.rows, {this.fetchError});
  final List<CatalogHit> rows;

  /// When set, [fetchBytes] throws it — exercises the hub's pre-flight guard and
  /// the typed error → localized message mapping.
  final Object? fetchError;
  final Set<String> saved = {};
  final List<String> saveCalls = [];
  int? lastMaxNoteValue;

  @override
  Future<CatalogSearchPage> search({
    String query = '',
    String? author,
    PracticeLevel? level,
    CatalogFilters filters = const CatalogFilters(),
    int limit = 20,
    int offset = 0,
  }) async {
    lastMaxNoteValue = filters.maxNoteValue;
    final page = rows.skip(offset).take(limit).toList();
    return CatalogSearchPage(
      hits: page,
      nextOffset: offset + page.length,
      total: rows.length,
    );
  }

  @override
  Future<void> save(String catalogId) async {
    saved.add(catalogId);
    saveCalls.add(catalogId);
  }

  @override
  Future<void> remove(String catalogId) async => saved.remove(catalogId);

  @override
  Future<List<CatalogHit>> listSaved() async =>
      rows.where((h) => saved.contains(h.id)).toList();

  @override
  Future<Uint8List> fetchBytes(String catalogId) async {
    final err = fetchError;
    if (err != null) throw err;
    return Uint8List(0);
  }

  @override
  Future<Uint8List> ratingPreviewBytes(String catalogId) async => Uint8List(0);

  @override
  Future<CatalogSearchPage> ratingDeck({
    int limit = 20,
    int offset = 0,
  }) async => const CatalogSearchPage(hits: [], nextOffset: 0, total: 0);
}

class _FakeUpload implements ScoreUploadService {
  _FakeUpload(this.mine);
  final List<ContributedScore> mine;
  final List<(String, bool)> favoriteCalls = [];
  final List<({String id, String? note})> proposeCalls = [];

  @override
  Future<List<ContributedScore>> listMyScores() async => mine;
  @override
  Future<void> deleteScore(String id) async {}
  @override
  Future<void> setFavorite(String id, bool favorite) async =>
      favoriteCalls.add((id, favorite));
  @override
  Future<void> propose({
    required String scoreId,
    required String license,
    required bool attestation,
    String attribution = '',
    String? resubmissionNote,
  }) async => proposeCalls.add((id: scoreId, note: resubmissionNote));

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

CatalogHit _hit(String id, String title) => CatalogHit(
  id: id,
  title: title,
  composer: 'Composer',
  level: PracticeLevel.beginner,
  license: 'CC-BY-4.0',
  source: 'pdmx',
);

ProviderContainer _container(
  _FakeCatalog catalog, {
  List<ContributedScore> uploads = const [],
  _FakeUpload? uploadFake,
}) {
  final c = ProviderContainer(
    overrides: [
      catalogServiceProvider.overrideWithValue(catalog),
      scoreUploadServiceProvider.overrideWithValue(
        uploadFake ?? _FakeUpload(uploads),
      ),
      canUseOnlineServicesProvider.overrideWithValue(true),
    ],
  );
  addTearDown(c.dispose);
  return c;
}

Future<void> _pump(WidgetTester tester, ProviderContainer c) async {
  await tester.binding.setSurfaceSize(const Size(900, 1200));
  await tester.pumpWidget(
    UncontrolledProviderScope(
      container: c,
      child: localizedApp(const ScoreHubScreen(), locale: const Locale('en')),
    ),
  );
  for (var i = 0; i < 10; i++) {
    await tester.pump(const Duration(milliseconds: 40));
  }
}

Future<void> _teardown(WidgetTester tester, ProviderContainer c) async {
  await tester.pumpWidget(const SizedBox());
  await tester.pump();
}

void main() {
  testWidgets('shows catalog results with an add-to-library action', (
    tester,
  ) async {
    final catalog = _FakeCatalog([_hit('c1', 'Clair de Lune')]);
    final c = _container(catalog);
    await _pump(tester, c);

    expect(find.text('Clair de Lune'), findsOneWidget);
    // Catalog results offer an add-to-library toggle.
    expect(find.byIcon(Icons.favorite_border), findsOneWidget);

    await tester.tap(find.byIcon(Icons.favorite_border));
    await tester.pump();
    await tester.pump();
    expect(catalog.saveCalls, ['c1']);
    await _teardown(tester, c);
  });

  testWidgets('"mes partitions" chip switches to uploads and hides add', (
    tester,
  ) async {
    final catalog = _FakeCatalog([_hit('c1', 'Clair de Lune')]);
    final c = _container(
      catalog,
      uploads: [
        ContributedScore(
          id: 'u1',
          level: PracticeLevel.beginner,
          createdAt: DateTime.utc(2026, 5, 1),
          measureCount: 4,
          timeSig: '4/4',
          keyFifths: 0,
          title: 'My Upload',
          composer: 'Me',
        ),
      ],
    );
    await _pump(tester, c);
    expect(find.text('Clair de Lune'), findsOneWidget);

    // Activate the "My scores" quick-filter.
    await tester.tap(find.widgetWithText(FilterChip, 'My scores'));
    for (var i = 0; i < 10; i++) {
      await tester.pump(const Duration(milliseconds: 40));
    }

    // Now the user's upload shows, the catalog result is gone, and uploads
    // offer no add-to-library toggle (already owned).
    expect(find.text('My Upload'), findsOneWidget);
    expect(find.text('Clair de Lune'), findsNothing);
    expect(find.byIcon(Icons.favorite_border), findsNothing);
    await _teardown(tester, c);
  });

  testWidgets('uploads are mixed into the catalog results by default', (
    tester,
  ) async {
    final c = _container(
      _FakeCatalog([_hit('c1', 'Clair de Lune')]),
      uploads: [
        ContributedScore(
          id: 'u1',
          level: PracticeLevel.beginner,
          createdAt: DateTime.utc(2026, 5, 1),
          measureCount: 4,
          timeSig: '4/4',
          keyFifths: 0,
          title: 'My Upload',
          composer: 'Me',
        ),
      ],
    );
    await _pump(tester, c);
    // "Mes partitions" unchecked → the upload and the catalog result both show.
    expect(find.text('My Upload'), findsOneWidget);
    expect(find.text('Clair de Lune'), findsOneWidget);
    await _teardown(tester, c);
  });

  testWidgets('empty catalog search shows the no-results state', (
    tester,
  ) async {
    final c = _container(_FakeCatalog(const []));
    await _pump(tester, c);
    expect(find.text('No scores match your search.'), findsOneWidget);
    await _teardown(tester, c);
  });

  testWidgets('"mes partitions" upload can be un-favorited from the hub', (
    tester,
  ) async {
    final upload = _FakeUpload([
      ContributedScore(
        id: 'u1',
        level: PracticeLevel.beginner,
        createdAt: DateTime.utc(2026, 5, 1),
        measureCount: 4,
        timeSig: '4/4',
        keyFifths: 0,
        title: 'My Upload',
        composer: 'Me',
      ),
    ]);
    final c = _container(_FakeCatalog(const []), uploadFake: upload);
    await _pump(tester, c);
    await tester.tap(find.widgetWithText(FilterChip, 'My scores'));
    for (var i = 0; i < 10; i++) {
      await tester.pump(const Duration(milliseconds: 40));
    }
    expect(find.text('My Upload'), findsOneWidget);
    // The upload is favorited → a filled heart; tapping it un-favorites.
    await tester.tap(find.byIcon(Icons.favorite));
    for (var i = 0; i < 10; i++) {
      await tester.pump(const Duration(milliseconds: 40));
    }
    expect(upload.favoriteCalls, [('u1', false)]);
    await _teardown(tester, c);
  });

  testWidgets('advanced-filters drawer applies a facet filter', (tester) async {
    final catalog = _FakeCatalog([_hit('c1', 'Clair de Lune')]);
    final c = _container(catalog);
    await _pump(tester, c);

    // Open the end-drawer via the app-bar tune action.
    await tester.tap(find.byIcon(Icons.tune));
    await tester.pumpAndSettle();
    expect(find.text('Advanced filters'), findsOneWidget);

    // Pick a rhythmic-granularity option → the notifier re-queries with it.
    await tester.tap(find.text('≤ Eighth'));
    for (var i = 0; i < 10; i++) {
      await tester.pump(const Duration(milliseconds: 40));
    }
    expect(catalog.lastMaxNoteValue, 8);
    await _teardown(tester, c);
  });

  testWidgets('result count shows the server total, not the loaded page', (
    tester,
  ) async {
    // 25 rows match but the first page loads only 20 (the page size): the count
    // label must read the server total (25), not the number loaded in memory.
    final catalog = _FakeCatalog([
      for (var i = 0; i < 25; i++) _hit('c$i', 'Score $i'),
    ]);
    final c = _container(catalog);
    await _pump(tester, c);

    expect(find.text('25 scores'), findsOneWidget);
    expect(find.text('20 scores'), findsNothing);
    await _teardown(tester, c);
  });

  testWidgets('tapping a score that fails to load shows a snackbar, no player', (
    tester,
  ) async {
    // Pre-flight guard: the score can't be fetched, so the player is never
    // opened — the user stays on the hub and gets a localized snackbar (never a
    // raw error).
    final catalog = _FakeCatalog([
      _hit('c1', 'Clair de Lune'),
    ], fetchError: StateError('boom'));
    final c = _container(catalog);
    await _pump(tester, c);

    await tester.tap(find.text('Clair de Lune'));
    await tester.pump(); // show the progress dialog
    await tester.pump(); // pre-flight load resolves (fetch throws)
    await tester.pump(
      const Duration(milliseconds: 700),
    ); // dialog out, snackbar in

    expect(find.text('Could not load this score.'), findsOneWidget);
    expect(find.textContaining('boom'), findsNothing); // no raw error leaked
    // Still on the hub — the player was not pushed.
    expect(find.text('Clair de Lune'), findsOneWidget);

    await tester.pump(
      const Duration(seconds: 5),
    ); // let the snackbar auto-dismiss
    await _teardown(tester, c);
  });

  testWidgets('a not-found score shows a specific localized snackbar', (
    tester,
  ) async {
    // A backend NOT_FOUND surfaces as a specific — but localized — message,
    // never the raw gRPC/exception text.
    final catalog = _FakeCatalog([
      _hit('c1', 'Clair de Lune'),
    ], fetchError: const AuthException(AuthError.notFound));
    final c = _container(catalog);
    await _pump(tester, c);

    await tester.tap(find.text('Clair de Lune'));
    await tester.pump();
    await tester.pump();
    await tester.pump(const Duration(milliseconds: 700));

    expect(find.text('This score no longer exists.'), findsOneWidget);
    expect(find.text('Could not load this score.'), findsNothing);

    await tester.pump(const Duration(seconds: 5));
    await _teardown(tester, c);
  });
}
