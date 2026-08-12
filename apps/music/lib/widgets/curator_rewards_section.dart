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
import '../screens/curator_activity_screen.dart';
import '../screens/soundfonts_screen.dart';
import '../services/curator_rewards_service.dart';
import '../state/curator_profile_notifier.dart';
import '../theme/cymbra_theme.dart';

/// The signed-in curator's rewards, rendered INLINE inside the user profile
/// (change: add-curation-rewards). Previously a standalone screen; now a section
/// of `ProfileScreen` so a user's standing lives with the rest of their profile.
/// Standing (level + lifetime points + progress), spendable balance with a reward
/// -shop entry, personal curator stats, and the recent-activity feed — all driven
/// through [curatorProfileProvider] (loading/data/error via `.when`). Returns a
/// `Column`, so it drops into the profile's scroll view (no Scaffold/ListView of
/// its own).
///
/// Badges are NOT here any more (change: add-achievement-badges): a grid of seven
/// curation milestones inside the curator section promised an achievement system
/// to users who never rate anything. They live in [AchievementsSection], which
/// spans every family.
class CuratorRewardsSection extends ConsumerWidget {
  const CuratorRewardsSection({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final async = ref.watch(curatorProfileProvider);
    return async.when(
      loading: () => const Padding(
        padding: EdgeInsets.symmetric(vertical: 24),
        child: Center(child: CircularProgressIndicator()),
      ),
      // A rewards read failure degrades quietly with a retry — it must never take
      // down the rest of the profile (activity, visibility).
      error: (_, _) => Padding(
        padding: const EdgeInsets.symmetric(vertical: 16),
        child: Row(
          children: [
            const Icon(Icons.cloud_off, color: CymbraColors.onSurfaceVariant),
            const SizedBox(width: 12),
            Expanded(
              child: Text(
                l10n.curatorLoadError,
                style: const TextStyle(color: CymbraColors.onSurfaceVariant),
              ),
            ),
            TextButton(
              onPressed: () =>
                  ref.read(curatorProfileProvider.notifier).refresh(),
              child: Text(l10n.curatorRetry),
            ),
          ],
        ),
      ),
      data: (r) => Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          // Mark the newest activity as seen once loaded, so the account-icon
          // notification dot clears (side effect isolated in its own widget).
          _CuratorSeenListener(rewards: r),
          _SectionTitle(l10n.curatorProfileTitle),
          const SizedBox(height: 8),
          _StandingCard(rewards: r),
          const SizedBox(height: 16),
          _BalanceCard(rewards: r),
          const SizedBox(height: 24),
          _SectionTitle(l10n.curatorStatsTitle),
          const SizedBox(height: 8),
          _StatsRow(rewards: r),
          const SizedBox(height: 16),
          // Recent activity is shown ON DEMAND (not inline) so the profile stays
          // uncluttered — a tappable "Activité récente ›" row opens the full list.
          _RecentActivityEntry(count: r.recent.length),
        ],
      ),
    );
  }
}

/// Isolates the "mark activity seen" side effect (architecture rule 4): once the
/// snapshot has a newest activity, record it so the account-icon dot clears.
class _CuratorSeenListener extends ConsumerStatefulWidget {
  const _CuratorSeenListener({required this.rewards});

  final CuratorRewardsView rewards;

  @override
  ConsumerState<_CuratorSeenListener> createState() =>
      _CuratorSeenListenerState();
}

class _CuratorSeenListenerState extends ConsumerState<_CuratorSeenListener> {
  @override
  void initState() {
    super.initState();
    final at = widget.rewards.newestActivityAt;
    if (at != null) {
      // Defer past the first frame — never mutate providers during build.
      WidgetsBinding.instance.addPostFrameCallback((_) {
        ref.read(curatorActivitySeenProvider.notifier).markSeen(at);
      });
    }
  }

  @override
  Widget build(BuildContext context) => const SizedBox.shrink();
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
            // No standalone shop (change: add-curation-rewards): redeemable sounds
            // appear directly in the SoundFont catalog with a purchase affordance.
            // This CTA just takes the user there to spend their balance.
            FilledButton.icon(
              icon: const Icon(Icons.library_music_outlined, size: 18),
              label: Text(l10n.curatorShopButton),
              onPressed: () => Navigator.of(context).push(
                MaterialPageRoute<void>(
                  builder: (_) => const SoundFontsScreen(),
                ),
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// A tappable "Activité récente ›" entry that opens the full activity list on
/// demand (change: add-curation-rewards) — keeps the profile uncluttered.
class _RecentActivityEntry extends StatelessWidget {
  const _RecentActivityEntry({required this.count});
  final int count;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Card(
      color: CymbraColors.surfaceContainerHigh,
      margin: EdgeInsets.zero,
      child: ListTile(
        key: const Key('curator-recent-entry'),
        leading: const Icon(
          Icons.history,
          color: CymbraColors.onSurfaceVariant,
        ),
        title: Text(
          l10n.curatorRecentTitle,
          style: const TextStyle(
            color: CymbraColors.onSurface,
            fontWeight: FontWeight.w700,
          ),
        ),
        subtitle: count == 0
            ? Text(
                l10n.curatorRecentEmpty,
                style: const TextStyle(color: CymbraColors.onSurfaceVariant),
              )
            : null,
        trailing: const Icon(
          Icons.chevron_right,
          color: CymbraColors.onSurfaceVariant,
        ),
        onTap: () => Navigator.of(context).push(
          MaterialPageRoute<void>(
            builder: (_) => const CuratorActivityScreen(),
          ),
        ),
      ),
    );
  }
}

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
