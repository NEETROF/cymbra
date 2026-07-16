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
import 'package:music/services/catalog_service.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/state/catalog_search_notifier.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';

/// In-memory [CatalogService] mirroring the backend's substring/filter search.
class _FakeCatalog implements CatalogService {
  _FakeCatalog(this.rows);
  final List<CatalogHit> rows;
  final Set<String> saved = {};
  final List<String> saveCalls = [];
  final List<String> removeCalls = [];
  // Records the last call's facet arguments (for assertions).
  bool? lastIsPiano;
  int? lastMaxNoteValue;

  @override
  Future<CatalogSearchPage> search({
    String query = '',
    String? author,
    PracticeLevel? level,
    bool? isPiano,
    int? maxNoteValue,
    bool? hasChords,
    bool? hasTuplets,
    bool? hasDotted,
    int? maxAmbitusSemitones,
    int? minBpm,
    int? maxBpm,
    int limit = 20,
    int offset = 0,
  }) async {
    lastIsPiano = isPiano;
    lastMaxNoteValue = maxNoteValue;
    final q = query.toLowerCase();
    final a = author?.toLowerCase();
    final matched = rows.where((h) {
      final t = (h.title ?? '').toLowerCase();
      final c = (h.composer ?? '').toLowerCase();
      final queryOk = q.isEmpty || t.contains(q) || c.contains(q);
      final authorOk = a == null || c.contains(a);
      final levelOk = level == null || h.level == level;
      // The corpus rows carry no facet data here; a set facet filter simply
      // narrows to nothing (mirrors "unknown facet excluded").
      final facetOk = maxNoteValue == null;
      return queryOk && authorOk && levelOk && facetOk;
    }).toList();
    final page = matched.skip(offset).take(limit).toList();
    return CatalogSearchPage(hits: page, nextOffset: offset + page.length);
  }

  @override
  Future<void> save(String catalogId) async {
    saved.add(catalogId);
    saveCalls.add(catalogId);
  }

  @override
  Future<void> remove(String catalogId) async {
    saved.remove(catalogId);
    removeCalls.add(catalogId);
  }

  @override
  Future<List<CatalogHit>> listSaved() async =>
      rows.where((h) => saved.contains(h.id)).toList();

  @override
  Future<Uint8List> fetchBytes(String catalogId) async => Uint8List(0);
}

class _FakeUpload implements ScoreUploadService {
  _FakeUpload(this.mine);
  final List<ContributedScore> mine;

  @override
  Future<List<ContributedScore>> listMyScores() async => mine;
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

CatalogHit _hit(
  String id,
  String title,
  String composer,
  PracticeLevel level,
) => CatalogHit(
  id: id,
  title: title,
  composer: composer,
  level: level,
  license: 'CC-BY-4.0',
  source: 'pdmx',
);

List<CatalogHit> _corpus() => [
  _hit('c1', 'Clair de Lune', 'Claude Debussy', PracticeLevel.intermediate),
  _hit('c2', 'Gymnopedie', 'Erik Satie', PracticeLevel.beginner),
  _hit('c3', 'Prelude', 'Claude Debussy', PracticeLevel.advanced),
];

ContributedScore _upload(String id, String title, String composer) =>
    ContributedScore(
      id: id,
      level: PracticeLevel.beginner,
      createdAt: DateTime.utc(2026, 5, 1),
      measureCount: 4,
      timeSig: '4/4',
      keyFifths: 0,
      title: title,
      composer: composer,
    );

ProviderContainer _container(
  _FakeCatalog catalog, {
  List<ContributedScore> uploads = const [],
}) {
  final c = ProviderContainer(
    overrides: [
      catalogServiceProvider.overrideWithValue(catalog),
      scoreUploadServiceProvider.overrideWithValue(_FakeUpload(uploads)),
      canUseOnlineServicesProvider.overrideWithValue(true),
    ],
  );
  // Hold a subscription so the autoDispose notifier is not torn down between
  // reads (its state, and the initial load, must survive across the test).
  final sub = c.listen(catalogSearchProvider, (_, _) {});
  addTearDown(sub.close);
  addTearDown(c.dispose);
  return c;
}

/// Wait until the notifier settles (initial or filter-triggered load done).
/// The leading delay clears the 300ms search debounce so a pending debounced
/// reload has started before we sample `loading`.
Future<CatalogSearchState> _settled(ProviderContainer c) async {
  await Future<void>.delayed(const Duration(milliseconds: 340));
  for (var i = 0; i < 30; i++) {
    await Future<void>.delayed(const Duration(milliseconds: 20));
    final s = c.read(catalogSearchProvider);
    if (!s.loading && !s.loadingMore) return s;
  }
  return c.read(catalogSearchProvider);
}

List<String> _ids(CatalogSearchState s) => [
  for (final e in s.entries) e.catalogId ?? e.contributedId ?? e.id,
];

void main() {
  test('initial browse loads the whole corpus', () async {
    final c = _container(_FakeCatalog(_corpus()));
    final s = await _settled(c);
    expect(_ids(s), ['c1', 'c2', 'c3']);
  });

  test('author and difficulty filters compose', () async {
    final c = _container(_FakeCatalog(_corpus()));
    await _settled(c);
    c.read(catalogSearchProvider.notifier).setAuthor('Debussy');
    c.read(catalogSearchProvider.notifier).setLevel(PracticeLevel.advanced);
    final s = await _settled(c);
    expect(_ids(s), ['c3']); // the advanced Debussy work only
  });

  test(
    'load more appends the next page without dropping earlier ones',
    () async {
      // 22 rows → page size 20, so a second page of 2 remains.
      final rows = [
        for (var i = 0; i < 22; i++)
          _hit('id$i', 'Piece $i', 'Composer', PracticeLevel.beginner),
      ];
      final c = _container(_FakeCatalog(rows));
      final first = await _settled(c);
      expect(first.entries.length, 20);
      expect(first.hasMore, isTrue);
      await c.read(catalogSearchProvider.notifier).loadMore();
      final s = await _settled(c);
      expect(s.entries.length, 22);
      expect(s.hasMore, isFalse);
    },
  );

  test('"mes partitions" source shows the user uploads, filtered', () async {
    final c = _container(
      _FakeCatalog(_corpus()),
      uploads: [
        _upload('u1', 'My Nocturne', 'Me'),
        _upload('u2', 'Study', 'Other'),
      ],
    );
    await _settled(c);
    c.read(catalogSearchProvider.notifier).setSource(CatalogSource.myUploads);
    final s = await _settled(c);
    expect(s.isMyUploads, isTrue);
    expect(_ids(s), ['u1', 'u2']);
    // The author filter narrows the uploads too.
    c.read(catalogSearchProvider.notifier).setAuthor('Me');
    final s2 = await _settled(c);
    expect(_ids(s2), ['u1']);
  });

  test('toggleSave saves then removes through the service', () async {
    final catalog = _FakeCatalog(_corpus());
    final c = _container(catalog);
    final s = await _settled(c);
    final entry = s.entries.firstWhere((e) => e.catalogId == 'c1');

    await c.read(catalogSearchProvider.notifier).toggleSave(entry);
    expect(catalog.saveCalls, ['c1']);
    expect(c.read(catalogSearchProvider).isSaved(entry), isTrue);

    await c.read(catalogSearchProvider.notifier).toggleSave(entry);
    expect(catalog.removeCalls, ['c1']);
    expect(c.read(catalogSearchProvider).isSaved(entry), isFalse);
  });

  test(
    'catalog search pins is_piano and applies advanced facet filters',
    () async {
      final catalog = _FakeCatalog(_corpus());
      final c = _container(catalog);
      await _settled(c);
      // The production catalog call always constrains to piano.
      expect(catalog.lastIsPiano, isTrue);

      // Applying a rhythmic-granularity filter re-queries with it.
      c.read(catalogSearchProvider.notifier).setMaxNoteValue(8);
      final s = await _settled(c);
      expect(catalog.lastMaxNoteValue, 8);
      expect(c.read(catalogSearchProvider).hasAdvancedFilters, isTrue);
      expect(s.entries, isEmpty); // facet-less fake rows are excluded

      // Clearing the advanced filters restores the results.
      c.read(catalogSearchProvider.notifier).clearAdvancedFilters();
      final s2 = await _settled(c);
      expect(catalog.lastMaxNoteValue, isNull);
      expect(s2.entries, isNotEmpty);
    },
  );
}
