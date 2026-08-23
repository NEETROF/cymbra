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

import '../src/grpc/score.pbgrpc.dart' as score;
import 'grpc_client.dart';
import 'rpc_deadlines.dart';

part 'curator_rewards_service.freezed.dart';
part 'curator_rewards_service.g.dart';

// --- Domain view models ------------------------------------------------------
//
// Freezed domain models mapped from the wire types (change: add-curation-
// rewards), so the notifier + widgets + their tests never depend on the
// generated proto classes. The service converts proto → view.

/// One badge in the full grid: earned or locked (with its milestone hint).
@freezed
abstract class CuratorBadgeView with _$CuratorBadgeView {
  const factory CuratorBadgeView({
    required String key,
    required String metric,
    required int threshold,
    required bool earned,
  }) = _CuratorBadgeView;
}

/// One recent points event for the profile activity feed. [source] is set for
/// honesty/adjustment settlements (`consensus` / `moderator`); [catalogId] /
/// [rewardKey] are set for the kinds they apply to.
@freezed
abstract class RewardActivityView with _$RewardActivityView {
  const factory RewardActivityView({
    required String kind,
    required int amount,
    required DateTime createdAt,
    String? catalogId,
    String? rewardKey,
    String? source,
  }) = _RewardActivityView;
}

/// The curator's rewards snapshot: standing (level/points), curator stats, the
/// full badge grid, and the recent-activity feed.
@freezed
abstract class CuratorRewardsView with _$CuratorRewardsView {
  const CuratorRewardsView._();

  const factory CuratorRewardsView({
    required int lifetimePoints,
    required int spendableBalance,
    required int level,
    required int levelFloor,
    required int nextLevelAt,
    required int totalRatings,
    required int coverageContribution,
    required double alignmentRate,
    required List<CuratorBadgeView> badges,
    required List<RewardActivityView> recent,
  }) = _CuratorRewardsView;

  /// Progress toward the next level as a 0..1 fraction of the current band
  /// (`levelFloor` → `nextLevelAt`). 1.0 when the band is degenerate (top level).
  double get levelProgress {
    final span = nextLevelAt - levelFloor;
    if (span <= 0) return 1;
    final into = (lifetimePoints - levelFloor).clamp(0, span);
    return into / span;
  }

  /// The newest activity timestamp, or null when the feed is empty (used to
  /// decide whether there are unseen honesty/adjustment awards).
  DateTime? get newestActivityAt =>
      recent.isEmpty ? null : recent.first.createdAt;
}

/// A reward-shop item — a priced (or coming-soon) SoundFont from the catalog.
@freezed
abstract class RewardShopItemView with _$RewardShopItemView {
  const factory RewardShopItemView({
    required String key,
    required String label,
    required String instrument,
    required String license,
    required String attribution,
    required int pointCost,
    required bool redeemable,
    required bool owned,
  }) = _RewardShopItemView;
}

/// The outcome of a redeem: whether the reward is now owned and the fresh balance.
@freezed
abstract class RedeemResultView with _$RedeemResultView {
  const factory RedeemResultView({
    required bool owned,
    required int newBalance,
  }) = _RedeemResultView;
}

// --- Conversion --------------------------------------------------------------

CuratorBadgeView _toBadge(score.CuratorBadge b) => CuratorBadgeView(
  key: b.key,
  metric: b.metric,
  threshold: b.threshold.toInt(),
  earned: b.earned,
);

RewardActivityView _toActivity(score.RewardActivity a) => RewardActivityView(
  kind: a.kind,
  amount: a.amount,
  // `created_at` is unix MILLIS (backend `extract(epoch …) * 1000`).
  createdAt: DateTime.fromMillisecondsSinceEpoch(a.createdAt.toInt()),
  catalogId: a.hasCatalogId() ? a.catalogId : null,
  rewardKey: a.hasRewardKey() ? a.rewardKey : null,
  source: a.hasSource() ? a.source : null,
);

CuratorRewardsView _toRewards(score.CuratorRewards r) => CuratorRewardsView(
  lifetimePoints: r.lifetimePoints.toInt(),
  spendableBalance: r.spendableBalance.toInt(),
  level: r.level.toInt(),
  levelFloor: r.levelFloor.toInt(),
  nextLevelAt: r.nextLevelAt.toInt(),
  totalRatings: r.totalRatings.toInt(),
  coverageContribution: r.coverageContribution.toInt(),
  alignmentRate: r.alignmentRate,
  badges: [for (final b in r.badges) _toBadge(b)],
  recent: [for (final a in r.recent) _toActivity(a)],
);

RewardShopItemView _toShopItem(score.RewardShopItem i) => RewardShopItemView(
  key: i.key,
  label: i.label,
  instrument: i.instrument,
  license: i.license,
  attribution: i.attribution,
  pointCost: i.pointCost,
  redeemable: i.redeemable,
  owned: i.owned,
);

// --- Service seam ------------------------------------------------------------

/// Seam over the backend `ScoreService`'s curation-rewards surface (change:
/// add-curation-rewards): read the caller's rewards snapshot, list the reward
/// shop, and redeem a reward. Every call is bearer-authenticated; the production
/// impl refreshes transparently on `UNAUTHENTICATED`. Tests override the provider
/// with a mockito mock. Failures throw `AuthException`.
abstract class CuratorRewardsService {
  /// The caller's rewards snapshot (standing, stats, badges, recent activity).
  Future<CuratorRewardsView> getRewards();

  /// The reward shop items (priced / coming-soon SoundFonts), with owned/
  /// redeemable flags relative to the caller.
  Future<List<RewardShopItemView>> listShop();

  /// Redeem the reward keyed by [rewardKey], returning the fresh ownership +
  /// balance.
  Future<RedeemResultView> redeem(String rewardKey);
}

/// Production [CuratorRewardsService] over the generated `ScoreServiceClient`.
/// Protected calls run through [AuthedRunner] so a stale access token is
/// refreshed once and the call retried transparently (mirrors [GrpcRatingService]).
class GrpcCuratorRewardsService implements CuratorRewardsService {
  GrpcCuratorRewardsService({
    required ClientChannel channel,
    required AuthedRunner authed,
    RpcDeadlines deadlines = const RpcDeadlines(),
  }) : _client = score.ScoreServiceClient(channel, interceptors: [deadlines]),
       _authed = authed;

  final score.ScoreServiceClient _client;
  final AuthedRunner _authed;

  @override
  Future<CuratorRewardsView> getRewards() => _authed(
    (bearer) async => _toRewards(
      await _client.getCuratorRewards(
        score.GetCuratorRewardsRequest(),
        options: bearerOptions(bearer),
      ),
    ),
  );

  @override
  Future<List<RewardShopItemView>> listShop() => _authed(
    (bearer) async => [
      for (final i in (await _client.listRewardShop(
        score.ListRewardShopRequest(),
        options: bearerOptions(bearer),
      )).items)
        _toShopItem(i),
    ],
  );

  @override
  Future<RedeemResultView> redeem(String rewardKey) => _authed((bearer) async {
    final resp = await _client.redeemReward(
      score.RedeemRewardRequest(rewardKey: rewardKey),
      options: bearerOptions(bearer),
    );
    return RedeemResultView(
      owned: resp.owned,
      newBalance: resp.newBalance.toInt(),
    );
  });
}

/// Production curator-rewards-service provider. Override in tests with a mock.
@Riverpod(keepAlive: true)
CuratorRewardsService curatorRewardsService(Ref ref) =>
    GrpcCuratorRewardsService(
      channel: ref.watch(cymbraChannelProvider),
      authed: ref.watch(authedRunnerProvider),
      deadlines: ref.watch(rpcDeadlinesProvider),
    );
