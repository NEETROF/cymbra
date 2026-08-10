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

import 'package:music/services/catalog_service.dart';
import 'package:music/services/rating_service.dart';
import 'package:music/state/score_catalog.dart';

/// A recorded rating submission (for assertions).
typedef Submission = ({String catalogId, RatingVerdict verdict, int? stars});

/// In-memory [RatingService]: records every submit and returns a canned
/// aggregate. Set [fail] to simulate a persistence failure.
class FakeRatingService implements RatingService {
  FakeRatingService({this.fail = false});
  final bool fail;
  final List<Submission> submissions = [];

  @override
  Future<RatingAggregate> submit({
    required String catalogId,
    required RatingVerdict verdict,
    int? stars,
  }) async {
    if (fail) throw Exception('network down');
    submissions.add((catalogId: catalogId, verdict: verdict, stars: stars));
    return const RatingAggregate(
      average: 3,
      count: 1,
      dislikeCount: 0,
      likeCount: 1,
      loveCount: 0,
    );
  }

  /// Mirrors the server: what this fake has recorded IS what the caller rated.
  /// Deliberately unaffected by [fail], which models a failing *write* — a test
  /// that needs a failing read overrides the service with a mock instead.
  @override
  Future<MyRating> myRating({required String catalogId}) async {
    for (final s in submissions.reversed) {
      if (s.catalogId == catalogId) {
        return MyRating(rated: true, verdict: s.verdict, stars: s.stars);
      }
    }
    return MyRating.none;
  }
}

/// In-memory [CatalogService] whose search returns a fixed page of `accepted`
/// hits (the backend gate guarantees only validated scores reach the app). Only
/// `search` is used by the rating deck; the rest are inert.
class FakeDeckCatalogService implements CatalogService {
  @override
  Future<Uint8List> getOfflineCacheKey() async => Uint8List(0);
  FakeDeckCatalogService(this.rows);
  final List<CatalogHit> rows;

  CatalogSearchPage _page(int limit, int offset) {
    final page = rows.skip(offset).take(limit).toList();
    return CatalogSearchPage(
      hits: page,
      nextOffset: offset + page.length,
      total: rows.length,
    );
  }

  @override
  Future<CatalogSearchPage> search({
    String query = '',
    String? author,
    PracticeLevel? level,
    CatalogFilters filters = const CatalogFilters(),
    int limit = 20,
    int offset = 0,
  }) async => _page(limit, offset);

  @override
  Future<CatalogSearchPage> ratingDeck({
    int limit = 20,
    int offset = 0,
  }) async => _page(limit, offset);

  @override
  Future<void> save(String catalogId) async {}
  @override
  Future<void> remove(String catalogId) async {}
  @override
  Future<List<CatalogHit>> listSaved() async => const [];
  @override
  Future<ScoreBytesResult> fetchScoreBytes(
    String catalogId, {
    String? ifNoneMatch,
  }) async => ScoreBytesResult(data: Uint8List(0), etag: '', unchanged: false);
  @override
  Future<Uint8List> ratingPreviewBytes(String catalogId) async => Uint8List(0);
}

/// A minimal catalog hit for the deck tests. Defaults to `accepted`; pass
/// `moderationStatus: 'pending'` to model a not-yet-validated candidate (change:
/// rate-pending-scores).
CatalogHit deckHit(String id, {String moderationStatus = 'accepted'}) =>
    CatalogHit(
      id: id,
      title: 'Piece $id',
      composer: 'Composer',
      level: PracticeLevel.beginner,
      license: 'CC-BY-4.0',
      source: 'pdmx',
      moderationStatus: moderationStatus,
    );

/// `n` accepted hits (`c0`, `c1`, …).
List<CatalogHit> deckCorpus([int n = 3]) => [
  for (var i = 0; i < n; i++) deckHit('c$i'),
];
