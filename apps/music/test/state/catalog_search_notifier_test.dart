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
import 'package:music/services/preferences_service.dart';
import 'package:music/services/score_upload_service.dart';
import 'package:music/state/catalog_search_notifier.dart';
import 'package:music/state/contributed_scores.dart';
import 'package:music/state/drums_access.dart';
import 'package:music/state/instrument_context.dart';
import 'package:music/state/score_catalog.dart';
import 'package:music/state/session_notifier.dart';

import '../support/prefs_fakes.dart';

/// In-memory [CatalogService] mirroring the backend's substring/filter search.
class _FakeCatalog implements CatalogService {
  @override
  Future<Uint8List> getOfflineCacheKey() async => Uint8List(0);
  @override
  Future<CatalogAccessState?> dailyAccess() async => null;
  @override
  Future<CatalogAccessState?> unlockForToday(String catalogId) async => null;
  _FakeCatalog(this.rows);
  final List<CatalogHit> rows;
  final Set<String> saved = {};
  final List<String> saveCalls = [];
  final List<String> removeCalls = [];
  // Records the last call's facet arguments (for assertions).
  ScoreInstrument? lastInstrument;
  bool instrumentEverSet = false;
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
    lastInstrument = filters.instrument;
    instrumentEverSet = instrumentEverSet || filters.instrument != null;
    lastMaxNoteValue = filters.maxNoteValue;
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
      final facetOk = filters.maxNoteValue == null;
      return queryOk && authorOk && levelOk && facetOk;
    }).toList();
    final page = matched.skip(offset).take(limit).toList();
    return CatalogSearchPage(
      hits: page,
      nextOffset: offset + page.length,
      total: matched.length,
    );
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
  Future<ScoreBytesResult> fetchScoreBytes(
    String catalogId, {
    String? ifNoneMatch,
  }) async => ScoreBytesResult(data: Uint8List(0), etag: '', unchanged: false);
  @override
  Future<Uint8List> ratingPreviewBytes(String catalogId) async => Uint8List(0);
  @override
  Future<CatalogSearchPage> ratingDeck({
    int limit = 20,
    int offset = 0,
  }) async => const CatalogSearchPage(hits: [], nextOffset: 0, total: 0);
}

class _FakeUpload implements ScoreUploadService {
  @override
  Future<void> propose({
    required String scoreId,
    required String license,
    required bool attestation,
    String attribution = '',
    String? resubmissionNote,
  }) async {}

  _FakeUpload(this.mine);
  final List<ContributedScore> mine;

  @override
  Future<List<ContributedScore>> listMyScores() async => mine;
  @override
  Future<void> deleteScore(String id) async {}
  @override
  Future<void> setFavorite(String id, bool favorite) async {}

  @override
  Future<ScoreBytesResult> fetchScoreBytes(
    String id, {
    String? ifNoneMatch,
  }) async => ScoreBytesResult(data: Uint8List(0), etag: '', unchanged: false);
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

ContributedScore _upload(
  String id,
  String title,
  String composer, {
  ScoreInstrument? instrument,
}) => ContributedScore(
  id: id,
  level: PracticeLevel.beginner,
  createdAt: DateTime.utc(2026, 5, 1),
  measureCount: 4,
  timeSig: '4/4',
  keyFifths: 0,
  title: title,
  composer: composer,
  instrument: instrument,
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
    c.read(catalogSearchProvider.notifier).setMyScoresOnly(true);
    final s = await _settled(c);
    expect(s.isMyUploads, isTrue);
    expect(_ids(s), ['u1', 'u2']);
    // The author filter narrows the uploads too.
    c.read(catalogSearchProvider.notifier).setAuthor('Me');
    final s2 = await _settled(c);
    expect(_ids(s2), ['u1']);
  });

  test('the instrument filter narrows the user uploads too', () async {
    // The defect: the uploads are filtered in the CLIENT (they are not part of
    // the catalog query), and the instrument had no local twin — so picking
    // "drums" kept a piano upload at the top of the list, which reads as a
    // broken filter rather than as two sources.
    final c = _container(
      _FakeCatalog(_corpus()),
      uploads: [
        _upload(
          'u1',
          'My Groove',
          'Me',
          instrument: ScoreInstrument.percussion,
        ),
        _upload('u2', 'My Waltz', 'Me', instrument: ScoreInstrument.keyboard),
        _upload('u3', 'Unknown', 'Me'),
      ],
    );
    await _settled(c);
    final notifier = c.read(catalogSearchProvider.notifier)
      ..setMyScoresOnly(true)
      ..setInstrument(ScoreInstrument.percussion);
    expect(_ids(await _settled(c)), ['u1']);

    notifier.setInstrument(ScoreInstrument.keyboard);
    expect(_ids(await _settled(c)), ['u2']);

    // No filter: everything comes back, unrecorded instrument included.
    notifier.setInstrument(null);
    expect(_ids(await _settled(c)), ['u1', 'u2', 'u3']);
  });

  test('a new upload refreshes the hub without a manual reload', () async {
    // Mutable uploads list shared with the fake service; invalidating the
    // uploads provider (as the upload flow does) must re-run the search.
    final uploads = <ContributedScore>[];
    final c = _container(_FakeCatalog(_corpus()), uploads: uploads);
    final s = await _settled(c);
    expect(_ids(s), ['c1', 'c2', 'c3']); // no uploads yet

    uploads.add(_upload('u1', 'Fresh Upload', 'Me'));
    c.invalidate(myUploadsProvider);
    final s2 = await _settled(c);
    // The upload now leads the mixed results, catalog still following.
    expect(_ids(s2), ['u1', 'c1', 'c2', 'c3']);
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
    'catalog search is NOT pinned to an instrument and applies facet filters',
    () async {
      final catalog = _FakeCatalog(_corpus());
      final c = _container(catalog);
      await _settled(c);
      // No instrument constraint of the hub's own (change: add-drums-access):
      // the backend already withholds what the caller may not see.
      expect(catalog.instrumentEverSet, isFalse);
      expect(catalog.lastInstrument, isNull);

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

  test('the instrument filter maps into the catalog request', () async {
    final catalog = _FakeCatalog(_corpus());
    final c = _container(catalog);
    await _settled(c);

    c
        .read(catalogSearchProvider.notifier)
        .setInstrument(ScoreInstrument.percussion);
    await _settled(c);
    expect(catalog.lastInstrument, ScoreInstrument.percussion);
    expect(c.read(catalogSearchProvider).hasAdvancedFilters, isTrue);

    // Back to "any": the constraint is dropped, not sent as keyboard.
    c.read(catalogSearchProvider.notifier).setInstrument(null);
    await _settled(c);
    expect(catalog.lastInstrument, isNull);

    // Reset-all clears it too.
    c
        .read(catalogSearchProvider.notifier)
        .setInstrument(ScoreInstrument.keyboard);
    await _settled(c);
    expect(catalog.lastInstrument, ScoreInstrument.keyboard);
    c.read(catalogSearchProvider.notifier).clearAdvancedFilters();
    await _settled(c);
    expect(catalog.lastInstrument, isNull);
    expect(c.read(catalogSearchProvider).hasAdvancedFilters, isFalse);
  });

  group('instrument-context seeding (change: add-instrument-context)', () {
    // A container where the context apparatus is live: drums visible, a real
    // context notifier over an in-memory store. The context is primed BEFORE
    // the hub notifier first builds — modelling a stored choice.
    ProviderContainer contextContainer(
      _FakeCatalog catalog, {
      AppInstrument context = AppInstrument.keyboard,
    }) {
      final c = ProviderContainer(
        overrides: [
          catalogServiceProvider.overrideWithValue(catalog),
          scoreUploadServiceProvider.overrideWithValue(_FakeUpload(const [])),
          canUseOnlineServicesProvider.overrideWithValue(true),
          preferencesServiceProvider.overrideWithValue(
            FakePreferencesService(),
          ),
          drumsEnabledProvider.overrideWithValue(true),
        ],
      );
      addTearDown(c.dispose);
      if (context != AppInstrument.keyboard) {
        c.read(instrumentContextProvider.notifier).select(context);
      }
      final sub = c.listen(catalogSearchProvider, (_, _) {});
      addTearDown(sub.close);
      return c;
    }

    test('the drums context seeds the percussion filter', () async {
      final catalog = _FakeCatalog(_corpus());
      final c = contextContainer(catalog, context: AppInstrument.drums);
      expect(
        c.read(catalogSearchProvider).instrument,
        ScoreInstrument.percussion,
      );
      await _settled(c);
      expect(catalog.lastInstrument, ScoreInstrument.percussion);
    });

    test('the keyboard context seeds the unconstrained browse', () async {
      final catalog = _FakeCatalog(_corpus());
      final c = contextContainer(catalog);
      expect(c.read(catalogSearchProvider).instrument, isNull);
      await _settled(c);
      expect(catalog.instrumentEverSet, isFalse);
    });

    test('adjusting the filter never writes back into the context', () async {
      final c = contextContainer(
        _FakeCatalog(_corpus()),
        context: AppInstrument.drums,
      );
      await _settled(c);
      // The drummer browses the whole catalog for a change…
      c.read(catalogSearchProvider.notifier).setInstrument(null);
      await _settled(c);
      expect(c.read(catalogSearchProvider).instrument, isNull);
      // …and the durable context is untouched: the filter is session state.
      expect(c.read(instrumentContextProvider).context, AppInstrument.drums);
    });

    test('an explicit context switch re-seeds, overriding an adjusted '
        'filter — in both directions', () async {
      final catalog = _FakeCatalog(_corpus());
      final c = contextContainer(catalog);
      await _settled(c);
      // The user adjusted the filter by hand…
      c
          .read(catalogSearchProvider.notifier)
          .setInstrument(ScoreInstrument.keyboard);
      await _settled(c);

      // …then switches context: the durable act outranks the session state.
      c.read(instrumentContextProvider.notifier).select(AppInstrument.drums);
      final s = await _settled(c);
      expect(s.instrument, ScoreInstrument.percussion);
      expect(catalog.lastInstrument, ScoreInstrument.percussion);

      // And back: keyboard's starting value is the unconstrained browse.
      c.read(instrumentContextProvider.notifier).select(AppInstrument.keyboard);
      final s2 = await _settled(c);
      expect(s2.instrument, isNull);
      expect(catalog.lastInstrument, isNull);
    });
  });
}
