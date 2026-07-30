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
  });

  /// Average effective value on the 1–5 scale (0 when there are no ratings).
  final double average;
  final int count;
  final int dislikeCount;
  final int likeCount;
  final int loveCount;
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
    );
  });
}

/// Production rating-service provider. Override in tests with a fake.
@Riverpod(keepAlive: true)
RatingService ratingService(Ref ref) => GrpcRatingService(
  channel: ref.watch(cymbraChannelProvider),
  authed: ref.watch(authedRunnerProvider),
);
