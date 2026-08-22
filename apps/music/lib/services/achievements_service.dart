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
import 'package:freezed_annotation/freezed_annotation.dart';
import 'package:grpc/grpc.dart';
import 'package:riverpod_annotation/riverpod_annotation.dart';

import '../courses/course_manifest.dart'
    show InlineText, parseInlineJson, resolveInline;
import '../src/grpc/score.pbgrpc.dart' as score;
import 'grpc_client.dart';
import 'rpc_deadlines.dart';

part 'achievements_service.freezed.dart';
part 'achievements_service.g.dart';

// --- Domain view models ------------------------------------------------------
//
// Freezed domain models mapped from the wire types (change: add-achievement-
// badges), so the notifier + widgets + their tests never depend on the generated
// proto classes.
//
// Nothing here enumerates badges: the app holds no list of keys, labels or
// metrics. The registry lives on the server and its identity travels with it as
// inline-localized maps, which is what lets a new badge ship without an app
// release.

/// One badge, projected against the signed-in user.
@freezed
abstract class AchievementBadgeView with _$AchievementBadgeView {
  const AchievementBadgeView._();

  const factory AchievementBadgeView({
    /// Stable registry key. Opaque to the app — used for identity, never for a
    /// `switch` on labels.
    required String key,
    required String family,
    required String metric,
    required int threshold,

    /// The graduated series this badge belongs to; empty for a standalone badge.
    @Default('') String track,

    /// The rung within [track]; 0 for a standalone badge.
    @Default(0) int tier,
    @Default(false) bool earned,

    /// The user's standing on [metric], already clamped to [threshold] by the
    /// server when earned — so an earned badge never renders as incomplete.
    @Default(0) int value,

    /// When it was earned; null while locked.
    DateTime? earnedAt,
    @Default(<String, String>{}) InlineText label,
    @Default(<String, String>{}) InlineText description,
  }) = _AchievementBadgeView;

  /// Whether this badge is one rung of a graduated series.
  bool get isTracked => track.isNotEmpty;

  /// Progress toward the threshold as a 0..1 fraction, for the locked tile's
  /// indicator. A degenerate threshold reads as complete rather than dividing by
  /// zero.
  double get progress {
    if (threshold <= 0) return 1;
    return (value / threshold).clamp(0.0, 1.0);
  }

  /// The label in [languageCode], falling back to English (design D6).
  String labelIn(String languageCode) => resolveInline(label, languageCode);

  /// The description in [languageCode], falling back to English.
  String descriptionIn(String languageCode) =>
      resolveInline(description, languageCode);
}

// --- Conversion --------------------------------------------------------------

AchievementBadgeView _toBadge(score.AchievementBadge b) => AchievementBadgeView(
  key: b.key,
  family: b.family,
  metric: b.metric,
  threshold: b.threshold.toInt(),
  track: b.track,
  tier: b.tier,
  earned: b.earned,
  value: b.value.toInt(),
  // `granted_at` is unix MILLIS, absent until the grant is recorded.
  earnedAt: b.hasGrantedAt()
      ? DateTime.fromMillisecondsSinceEpoch(b.grantedAt.toInt())
      : null,
  label: parseInlineJson(b.labelJson),
  description: parseInlineJson(b.descriptionJson),
);

// --- Service seam ------------------------------------------------------------

/// Seam over the backend `ScoreService`'s achievements surface (change:
/// add-achievement-badges). Bearer-authenticated; the production impl refreshes
/// transparently on `UNAUTHENTICATED`. Tests override the provider with a mockito
/// mock. Failures throw `AuthException`.
abstract class AchievementsService {
  /// The caller's standing on every badge in the server registry, in registry
  /// order. The read also awards anything newly due, server-side.
  Future<List<AchievementBadgeView>> getAchievements();
}

/// Production [AchievementsService] over the generated `ScoreServiceClient`.
/// Protected calls run through [AuthedRunner] so a stale access token is
/// refreshed once and the call retried transparently (mirrors
/// [GrpcCuratorRewardsService]).
class GrpcAchievementsService implements AchievementsService {
  GrpcAchievementsService({
    required ClientChannel channel,
    required AuthedRunner authed,
    RpcDeadlines deadlines = const RpcDeadlines(),
  }) : _client = score.ScoreServiceClient(channel, interceptors: [deadlines]),
       _authed = authed;

  final score.ScoreServiceClient _client;
  final AuthedRunner _authed;

  @override
  Future<List<AchievementBadgeView>> getAchievements() => _authed(
    (bearer) async => [
      for (final b in (await _client.getAchievements(
        score.GetAchievementsRequest(),
        options: bearerOptions(bearer),
      )).badges)
        _toBadge(b),
    ],
  );
}

/// Production achievements-service provider. Override in tests with a mock.
@Riverpod(keepAlive: true)
AchievementsService achievementsService(Ref ref) => GrpcAchievementsService(
  channel: ref.watch(cymbraChannelProvider),
  authed: ref.watch(authedRunnerProvider),
  deadlines: ref.watch(rpcDeadlinesProvider),
);
