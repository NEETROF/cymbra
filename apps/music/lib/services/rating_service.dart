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

import 'package:flutter_riverpod/flutter_riverpod.dart';
import 'package:grpc/grpc.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../src/grpc/score.pbgrpc.dart' as score;
import 'grpc_client.dart';

part 'rating_service.g.dart';

/// The swipe verdict for a score rating (change: add-app-score-rating). Left =
/// dislike (a negative verdict), right = like, up = love. Also the granularity a
/// tap-button carries; a star rating derives its verdict from the star value.
enum RatingVerdict {
  dislike,
  like,
  love;

  /// The wire string the backend expects (matches the proto `verdict`).
  String get wire => name;

  /// Parse a wire string back; null for anything unknown (a rating read back from
  /// a newer server must never crash the app).
  static RatingVerdict? fromWire(String? wire) => switch (wire) {
    'dislike' => RatingVerdict.dislike,
    'like' => RatingVerdict.like,
    'love' => RatingVerdict.love,
    _ => null,
  };
}

/// The verdict a 1–5 star value implies (design D2): 5 → love, 3–4 → like,
/// 1–2 → dislike. Shared by every star surface — the deck's star sheet and the
/// post-play prompt — so stars and swipes can never fold onto different scales.
RatingVerdict verdictForStars(int stars) {
  if (stars >= 5) return RatingVerdict.love;
  if (stars >= 3) return RatingVerdict.like;
  return RatingVerdict.dislike;
}

/// The caller's own rating of one score (change: add-post-play-rating-prompt).
/// [verdict] and [stars] are null when [rated] is false; [stars] is also null for
/// a swipe-only rating.
class MyRating {
  const MyRating({required this.rated, this.verdict, this.stars});

  /// The un-rated answer — also what a fail-closed read (unknown/rejected score)
  /// reports, deliberately indistinguishable from a genuine "never rated".
  static const MyRating none = MyRating(rated: false);

  final bool rated;
  final RatingVerdict? verdict;
  final int? stars;
}

/// The per-score aggregate returned when a rating is submitted, so the UI can
/// reflect the fresh average/counts without a second round-trip.
class RatingAggregate {
  const RatingAggregate({
    required this.average,
    required this.count,
    required this.dislikeCount,
    required this.likeCount,
    required this.loveCount,
    this.pointsAwarded = 0,
  });

  /// Average effective value on the 1–5 scale (0 when there are no ratings).
  final double average;
  final int count;
  final int dislikeCount;
  final int likeCount;
  final int loveCount;

  /// Curator coverage points earned by THIS rating (change: add-curation-rewards),
  /// so the deck can show an immediate "+N" cue. 0 when nothing was awarded
  /// (daily cap hit, not engaged, already rated, or a fully-covered score).
  final int pointsAwarded;
}

/// Seam over the backend `ScoreService`'s rating surface — submit/update the
/// caller's rating of an `accepted` catalog score. Every call is bearer-
/// authenticated; the production impl refreshes transparently on
/// `UNAUTHENTICATED`. Tests override the provider with an in-memory fake.
/// Failures throw `AuthException`.
abstract class RatingService {
  /// Submit (or update) the caller's rating of the catalog score [catalogId],
  /// carrying a swipe [verdict] and an optional 1–5 [stars] value. Returns the
  /// freshly-recomputed per-score aggregate.
  Future<RatingAggregate> submit({
    required String catalogId,
    required RatingVerdict verdict,
    int? stars,
  });

  /// The caller's OWN rating of [catalogId] (change: add-post-play-rating-prompt),
  /// so the player can suppress the post-play prompt for a score already rated on
  /// any device. Fail-closed server-side: an unknown or `rejected` score answers
  /// "not rated" rather than erroring.
  Future<MyRating> myRating({required String catalogId});
}

/// Production [RatingService] over the generated `ScoreServiceClient`. Protected
/// calls run through [authedCall] so a stale access token is refreshed once and
/// the call retried transparently (mirrors [GrpcCatalogService]).
class GrpcRatingService implements RatingService {
  GrpcRatingService({
    required ClientChannel channel,
    required AuthedRunner authed,
  }) : _client = score.ScoreServiceClient(channel),
       _authed = authed;

  final score.ScoreServiceClient _client;
  final AuthedRunner _authed;

  @override
  Future<RatingAggregate> submit({
    required String catalogId,
    required RatingVerdict verdict,
    int? stars,
  }) => _authed((bearer) async {
    final resp = await _client.submitScoreRating(
      score.SubmitScoreRatingRequest(
        catalogId: catalogId,
        verdict: verdict.wire,
        stars: stars,
      ),
      options: bearerOptions(bearer),
    );
    return RatingAggregate(
      average: resp.ratingAvg,
      count: resp.ratingCount,
      dislikeCount: resp.dislikeCount,
      likeCount: resp.likeCount,
      loveCount: resp.loveCount,
      pointsAwarded: resp.pointsAwarded,
    );
  });

  @override
  Future<MyRating> myRating({required String catalogId}) => _authed((
    bearer,
  ) async {
    final resp = await _client.getMyScoreRating(
      score.GetMyScoreRatingRequest(catalogId: catalogId),
      options: bearerOptions(bearer),
    );
    if (!resp.rated) return MyRating.none;
    return MyRating(
      rated: true,
      verdict: RatingVerdict.fromWire(resp.hasVerdict() ? resp.verdict : null),
      stars: resp.hasStars() ? resp.stars : null,
    );
  });
}

/// Production rating-service provider. Override in tests with a fake.
@Riverpod(keepAlive: true)
RatingService ratingService(Ref ref) => GrpcRatingService(
  channel: ref.watch(cymbraChannelProvider),
  authed: ref.watch(authedRunnerProvider),
);
