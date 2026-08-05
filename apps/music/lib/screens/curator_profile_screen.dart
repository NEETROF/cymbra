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

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../services/curator_rewards_service.dart';
import '../state/curator_profile_notifier.dart';
import '../theme/cymbra_theme.dart';
import 'reward_shop_screen.dart';

/// The full-screen curator profile (change: add-curation-rewards): standing
/// (level + lifetime points + progress), spendable balance with an entry into the
/// reward shop, the full badge grid (earned + locked with milestone hints),
/// personal curator stats, and the recent-activity feed. Driven entirely through
/// [curatorProfileProvider] (loading/data/error via `.when`).
class CuratorProfileScreen extends ConsumerWidget {
  const CuratorProfileScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    return Scaffold(
      backgroundColor: CymbraColors.background,
      appBar: AppBar(
        title: Text(l10n.curatorProfileTitle),
        backgroundColor: CymbraColors.surfaceContainerLowest,
      ),
      // Marking recent activity as "seen" is a side effect → a dedicated listener
      // widget near the top of the subtree (architecture rule 4).
      body: const _CuratorProfileListeners(
        child: SafeArea(child: _CuratorProfileBody()),
      ),
    );
  }
}

/// Isolates the profile's `ref.listen` side effect: once the snapshot loads, mark
/// its newest activity as seen so the notification dot clears.
class _CuratorProfileListeners extends ConsumerWidget {
  const _CuratorProfileListeners({required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    ref.listen(curatorProfileProvider, (_, next) {
      final at = next.valueOrNull?.newestActivityAt;
      if (at != null) {
        ref.read(curatorActivitySeenProvider.notifier).markSeen(at);
      }
    });
    return child;
  }
}

class _CuratorProfileBody extends ConsumerWidget {
  const _CuratorProfileBody();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final async = ref.watch(curatorProfileProvider);
    return async.when(
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (_, _) => Center(
        child: Padding(
          padding: const EdgeInsets.all(32),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              const Icon(
                Icons.cloud_off,
                size: 56,
                color: CymbraColors.onSurfaceVariant,
              ),
              const SizedBox(height: 20),
              Text(
                l10n.curatorLoadError,
                textAlign: TextAlign.center,
                style: const TextStyle(
                  color: CymbraColors.onSurface,
                  fontSize: 18,
                  fontWeight: FontWeight.w700,
                ),
              ),
              const SizedBox(height: 24),
              FilledButton(
                onPressed: () =>
                    ref.read(curatorProfileProvider.notifier).refresh(),
                child: Text(l10n.curatorRetry),
              ),
            ],
          ),
        ),
      ),
      data: (r) => RefreshIndicator(
        onRefresh: () => ref.read(curatorProfileProvider.notifier).refresh(),
        child: ListView(
          padding: const EdgeInsets.fromLTRB(16, 16, 16, 32),
          children: [
            _StandingCard(rewards: r),
            const SizedBox(height: 16),
            _BalanceCard(rewards: r),
            const SizedBox(height: 24),
            _SectionTitle(l10n.curatorBadgesTitle),
            const SizedBox(height: 8),
            _BadgeGrid(badges: r.badges),
            const SizedBox(height: 24),
            _SectionTitle(l10n.curatorStatsTitle),
            const SizedBox(height: 8),
            _StatsRow(rewards: r),
            const SizedBox(height: 24),
            _SectionTitle(l10n.curatorRecentTitle),
            const SizedBox(height: 8),
            _RecentActivity(activity: r.recent),
          ],
        ),
      ),
    );
  }
}

class _SectionTitle extends StatelessWidget {
  const _SectionTitle(this.text);
  final String text;

  @override
  Widget build(BuildContext context) => Text(
    text,
    style: const TextStyle(
      color: CymbraColors.primary,
      fontSize: 15,
      fontWeight: FontWeight.w800,
      letterSpacing: 0.5,
    ),
  );
}

/// Standing: level, lifetime points, and the progress bar to the next level.
class _StandingCard extends StatelessWidget {
  const _StandingCard({required this.rewards});
  final CuratorRewardsView rewards;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final atMax = rewards.nextLevelAt <= rewards.levelFloor;
    return Card(
      color: CymbraColors.surfaceContainerHigh,
      child: Padding(
        padding: const EdgeInsets.all(20),
        child: Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            Row(
              children: [
                const Icon(
                  Icons.workspace_premium,
                  color: CymbraColors.primary,
                  size: 28,
                ),
                const SizedBox(width: 10),
                Text(
                  l10n.curatorLevel(rewards.level),
                  style: const TextStyle(
                    color: CymbraColors.onSurface,
                    fontSize: 22,
                    fontWeight: FontWeight.w800,
                  ),
                ),
                const Spacer(),
                Text(
                  l10n.curatorPointsValue(rewards.lifetimePoints),
                  style: const TextStyle(
                    color: CymbraColors.primary,
                    fontSize: 16,
                    fontWeight: FontWeight.w700,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 4),
            Text(
              l10n.curatorLifetimeLabel,
              style: const TextStyle(
                color: CymbraColors.onSurfaceVariant,
                fontSize: 12,
              ),
            ),
            const SizedBox(height: 14),
            ClipRRect(
              borderRadius: BorderRadius.circular(6),
              child: LinearProgressIndicator(
                value: rewards.levelProgress,
                minHeight: 9,
                backgroundColor: CymbraColors.background,
                valueColor: const AlwaysStoppedAnimation(CymbraColors.primary),
              ),
            ),
            const SizedBox(height: 6),
            Text(
              atMax
                  ? l10n.curatorAtMaxLevel
                  : l10n.curatorProgressLabel(
                      rewards.lifetimePoints,
                      rewards.nextLevelAt,
                    ),
              style: const TextStyle(
                color: CymbraColors.onSurfaceVariant,
                fontSize: 12,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// Spendable balance + an entry into the reward shop.
class _BalanceCard extends StatelessWidget {
  const _BalanceCard({required this.rewards});
  final CuratorRewardsView rewards;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Card(
      color: CymbraColors.surfaceContainerHigh,
      child: Padding(
        padding: const EdgeInsets.fromLTRB(20, 16, 16, 16),
        child: Row(
          children: [
            Expanded(
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  Text(
                    l10n.curatorPointsValue(rewards.spendableBalance),
                    style: const TextStyle(
                      color: CymbraColors.onSurface,
                      fontSize: 20,
                      fontWeight: FontWeight.w800,
                    ),
                  ),
                  Text(
                    l10n.curatorBalanceLabel,
                    style: const TextStyle(
                      color: CymbraColors.onSurfaceVariant,
                      fontSize: 12,
                    ),
                  ),
                ],
              ),
            ),
            FilledButton.icon(
              icon: const Icon(Icons.card_giftcard, size: 18),
              label: Text(l10n.curatorShopButton),
              onPressed: () => Navigator.of(context).push(
                MaterialPageRoute<void>(
                  builder: (_) => const RewardShopScreen(),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// The full badge grid — all badges, earned or locked (with a milestone hint).
class _BadgeGrid extends StatelessWidget {
  const _BadgeGrid({required this.badges});
  final List<CuratorBadgeView> badges;

  @override
  Widget build(BuildContext context) => GridView.builder(
    shrinkWrap: true,
    physics: const NeverScrollableScrollPhysics(),
    gridDelegate: const SliverGridDelegateWithMaxCrossAxisExtent(
      maxCrossAxisExtent: 180,
      mainAxisSpacing: 10,
      crossAxisSpacing: 10,
      childAspectRatio: 1.25,
    ),
    itemCount: badges.length,
    itemBuilder: (context, i) => _BadgeTile(badge: badges[i]),
  );
}

class _BadgeTile extends StatelessWidget {
  const _BadgeTile({required this.badge});
  final CuratorBadgeView badge;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final earned = badge.earned;
    final color = earned ? CymbraColors.primary : CymbraColors.outline;
    return Container(
      padding: const EdgeInsets.all(10),
      decoration: BoxDecoration(
        color: CymbraColors.surfaceContainerHigh,
        borderRadius: BorderRadius.circular(12),
        border: Border.all(
          color: earned
              ? CymbraColors.primary.withValues(alpha: 0.5)
              : Colors.transparent,
        ),
      ),
      child: Column(
        mainAxisAlignment: MainAxisAlignment.center,
        children: [
          Icon(
            earned ? Icons.military_tech : Icons.lock_outline,
            color: color,
            size: 30,
          ),
          const SizedBox(height: 6),
          Text(
            _badgeLabel(l10n, badge.key),
            textAlign: TextAlign.center,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
            style: TextStyle(
              color: earned
                  ? CymbraColors.onSurface
                  : CymbraColors.onSurfaceVariant,
              fontSize: 12.5,
              fontWeight: FontWeight.w700,
            ),
          ),
          const SizedBox(height: 2),
          Text(
            earned
                ? l10n.curatorBadgeEarned
                : l10n.curatorBadgeMilestone(
                    badge.threshold,
                    _metricLabel(l10n, badge.metric),
                  ),
            textAlign: TextAlign.center,
            maxLines: 2,
            overflow: TextOverflow.ellipsis,
            style: const TextStyle(
              color: CymbraColors.onSurfaceVariant,
              fontSize: 10.5,
            ),
          ),
        ],
      ),
    );
  }
}

/// Localized badge label for a known [key], falling back to the raw key.
String _badgeLabel(AppLocalizations l10n, String key) => switch (key) {
  'first_note' => l10n.curatorBadgeFirstNote,
  'curator_1' => l10n.curatorBadgeCurator1,
  'curator_2' => l10n.curatorBadgeCurator2,
  'curator_3' => l10n.curatorBadgeCurator3,
  'sharp_ear_1' => l10n.curatorBadgeSharpEar1,
  'sharp_ear_2' => l10n.curatorBadgeSharpEar2,
  'trailblazer' => l10n.curatorBadgeTrailblazer,
  _ => key,
};

/// Localized name for a badge metric, for the locked milestone hint.
String _metricLabel(AppLocalizations l10n, String metric) => switch (metric) {
  'rating_count' => l10n.curatorMetricRatingCount,
  'aligned_count' => l10n.curatorMetricAlignedCount,
  'first_rater_count' => l10n.curatorMetricFirstRaterCount,
  _ => metric,
};

/// Personal stats: total ratings, coverage points contributed, alignment rate.
class _StatsRow extends StatelessWidget {
  const _StatsRow({required this.rewards});
  final CuratorRewardsView rewards;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Row(
      children: [
        _StatBox(
          label: l10n.curatorStatRatings,
          value: '${rewards.totalRatings}',
        ),
        const SizedBox(width: 8),
        _StatBox(
          label: l10n.curatorStatCoverage,
          value: '${rewards.coverageContribution}',
        ),
        const SizedBox(width: 8),
        _StatBox(
          label: l10n.curatorStatAlignment,
          value: '${(rewards.alignmentRate * 100).round()}%',
        ),
      ],
    );
  }
}

class _StatBox extends StatelessWidget {
  const _StatBox({required this.label, required this.value});
  final String label;
  final String value;

  @override
  Widget build(BuildContext context) => Expanded(
    child: Container(
      padding: const EdgeInsets.symmetric(vertical: 14, horizontal: 8),
      decoration: BoxDecoration(
        color: CymbraColors.surfaceContainerHigh,
        borderRadius: BorderRadius.circular(12),
      ),
      child: Column(
        children: [
          Text(
            value,
            style: const TextStyle(
              color: CymbraColors.primary,
              fontSize: 20,
              fontWeight: FontWeight.w800,
            ),
          ),
          const SizedBox(height: 4),
          Text(
            label,
            textAlign: TextAlign.center,
            style: const TextStyle(
              color: CymbraColors.onSurfaceVariant,
              fontSize: 11,
            ),
          ),
        ],
      ),
    ),
  );
}

/// The recent-activity feed. Each entry states its amount and, for a deferred
/// honesty/adjustment award, its source (consensus vs moderator).
class _RecentActivity extends StatelessWidget {
  const _RecentActivity({required this.activity});
  final List<RewardActivityView> activity;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    if (activity.isEmpty) {
      return Text(
        l10n.curatorRecentEmpty,
        style: const TextStyle(color: CymbraColors.onSurfaceVariant),
      );
    }
    return Column(
      children: [for (final a in activity) _ActivityRow(activity: a)],
    );
  }
}

class _ActivityRow extends StatelessWidget {
  const _ActivityRow({required this.activity});
  final RewardActivityView activity;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final positive = activity.amount >= 0;
    final amountText = positive ? '+${activity.amount}' : '${activity.amount}';
    return Card(
      color: CymbraColors.surfaceContainerHigh,
      margin: const EdgeInsets.symmetric(vertical: 4),
      child: ListTile(
        dense: true,
        leading: Icon(_kindIcon(activity.kind), color: CymbraColors.primary),
        title: Text(
          _kindLabel(l10n, activity.kind),
          style: const TextStyle(color: CymbraColors.onSurface, fontSize: 14),
        ),
        subtitle: _sourceLabel(l10n, activity.source) == null
            ? null
            : Text(
                _sourceLabel(l10n, activity.source)!,
                style: const TextStyle(
                  color: CymbraColors.onSurfaceVariant,
                  fontSize: 12,
                ),
              ),
        trailing: Text(
          amountText,
          style: TextStyle(
            color: positive ? CymbraColors.primary : CymbraColors.error,
            fontSize: 15,
            fontWeight: FontWeight.w800,
          ),
        ),
      ),
    );
  }
}

IconData _kindIcon(String kind) => switch (kind) {
  'coverage' => Icons.explore_outlined,
  'honesty' => Icons.verified_outlined,
  'adjustment' => Icons.tune,
  'redeem' => Icons.card_giftcard,
  _ => Icons.stars,
};

String _kindLabel(AppLocalizations l10n, String kind) => switch (kind) {
  'coverage' => l10n.curatorActivityCoverage,
  'honesty' => l10n.curatorActivityHonesty,
  'adjustment' => l10n.curatorActivityAdjustment,
  'redeem' => l10n.curatorActivityRedeem,
  _ => kind,
};

String? _sourceLabel(AppLocalizations l10n, String? source) => switch (source) {
  'consensus' => l10n.curatorActivitySourceConsensus,
  'moderator' => l10n.curatorActivitySourceModerator,
  _ => null,
};
