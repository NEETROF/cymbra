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
import '../state/reward_shop_notifier.dart';
import '../theme/cymbra_theme.dart';
import '../widgets/app_snackbar.dart';
import '../widgets/reward_celebration.dart';

/// The reward shop (change: add-curation-rewards): lists the priced / coming-soon
/// SoundFonts a curator can unlock with their points. A redeem calls the notifier
/// (which redeems then refreshes both the shop and, reactively, the balance);
/// unaffordable or coming-soon items are greyed and non-redeemable.
class RewardShopScreen extends ConsumerWidget {
  const RewardShopScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    return Scaffold(
      backgroundColor: CymbraColors.background,
      appBar: AppBar(
        title: Text(l10n.rewardShopTitle),
        backgroundColor: CymbraColors.surfaceContainerLowest,
      ),
      // The shop's side effects (celebration on redeem, error snackbar) live in a
      // dedicated listener widget near the top of the subtree (architecture rule 4).
      body: const _RewardShopListeners(
        child: SafeArea(child: _RewardShopBody()),
      ),
    );
  }
}

/// Isolates the shop's `ref.listen` side effects so they are not scattered in the
/// build methods below.
class _RewardShopListeners extends ConsumerWidget {
  const _RewardShopListeners({required this.child});

  final Widget child;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    ref.listen(rewardShopProvider.select((s) => s.valueOrNull?.redeemSeq), (
      prev,
      next,
    ) {
      if (next == null || next == prev) return;
      final state = ref.read(rewardShopProvider).valueOrNull;
      if (state == null) return;
      if (state.redeemError) {
        showAppSnackBar(
          ScaffoldMessenger.of(context),
          l10n.rewardShopRedeemError,
        );
        return;
      }
      final label = state.lastRedeemedLabel;
      if (label != null) {
        showRewardCelebration(
          context,
          title: l10n.rewardCelebrationRedeemedTitle,
          message: l10n.rewardShopRedeemed(label),
          icon: Icons.music_note,
        );
      }
    });
    return child;
  }
}

class _RewardShopBody extends ConsumerWidget {
  const _RewardShopBody();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final shop = ref.watch(rewardShopProvider);
    // The spendable balance drives affordability; still loading → treat as 0.
    final balance =
        ref.watch(curatorProfileProvider).valueOrNull?.spendableBalance ?? 0;

    return shop.when(
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (_, _) => _Message(
        icon: Icons.cloud_off,
        title: l10n.rewardShopLoadError,
        action: FilledButton(
          onPressed: () => ref.invalidate(rewardShopProvider),
          child: Text(l10n.curatorRetry),
        ),
      ),
      data: (state) {
        if (state.items.isEmpty) {
          return _Message(
            icon: Icons.card_giftcard,
            title: l10n.rewardShopEmpty,
          );
        }
        return Column(
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 12, 16, 4),
              child: Align(
                alignment: Alignment.centerLeft,
                child: Text(
                  l10n.rewardShopBalance(balance),
                  style: const TextStyle(
                    color: CymbraColors.onSurfaceVariant,
                    fontSize: 13,
                    fontWeight: FontWeight.w600,
                  ),
                ),
              ),
            ),
            Expanded(
              child: ListView(
                padding: const EdgeInsets.only(bottom: 24),
                children: [
                  for (final item in state.items)
                    _RewardCard(
                      item: item,
                      balance: balance,
                      onRedeem: () => ref
                          .read(rewardShopProvider.notifier)
                          .redeem(item.key),
                    ),
                ],
              ),
            ),
          ],
        );
      },
    );
  }
}

/// One reward row: label + licence/attribution, a cost pill, and a trailing
/// affordance (owned / coming soon / not enough points / redeem).
class _RewardCard extends StatelessWidget {
  const _RewardCard({
    required this.item,
    required this.balance,
    required this.onRedeem,
  });

  final RewardShopItemView item;
  final int balance;
  final VoidCallback onRedeem;

  String? get _subtitle {
    final parts = [
      if (item.license.isNotEmpty) item.license,
      if (item.attribution.isNotEmpty) item.attribution,
    ];
    return parts.isEmpty ? null : parts.join(' · ');
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final affordable = balance >= item.pointCost;
    // An unaffordable redeemable item shows its "not enough points" hint on the
    // subtitle line, so the trailing stays a single button (no overflow in the
    // height-constrained ListTile trailing).
    final showInsufficient = !item.owned && item.redeemable && !affordable;
    final subtitle = [
      ?_subtitle,
      if (showInsufficient) l10n.rewardShopInsufficient,
    ].join(' · ');
    return Card(
      color: CymbraColors.surfaceContainerHigh,
      margin: const EdgeInsets.symmetric(horizontal: 12, vertical: 4),
      child: ListTile(
        leading: const Icon(Icons.piano, color: CymbraColors.onSurfaceVariant),
        title: Text(
          item.label,
          style: const TextStyle(color: CymbraColors.onSurface),
        ),
        subtitle: subtitle.isEmpty
            ? null
            : Text(
                subtitle,
                style: TextStyle(
                  color: showInsufficient
                      ? CymbraColors.error
                      : CymbraColors.onSurfaceVariant,
                  fontSize: 12,
                ),
              ),
        trailing: _trailing(context, l10n, affordable),
      ),
    );
  }

  Widget _trailing(
    BuildContext context,
    AppLocalizations l10n,
    bool affordable,
  ) {
    if (item.owned) {
      return _Tag(label: l10n.rewardShopOwned, color: CymbraColors.primary);
    }
    if (!item.redeemable) {
      return _Tag(
        label: l10n.rewardShopComingSoon,
        color: CymbraColors.onSurfaceVariant,
      );
    }
    return FilledButton(
      key: Key('reward-redeem-${item.key}'),
      // Unaffordable → disabled (greyed); the hint is on the subtitle line.
      onPressed: affordable ? onRedeem : null,
      child: Text(l10n.rewardShopCost(item.pointCost)),
    );
  }
}

/// A small status pill (owned / coming soon).
class _Tag extends StatelessWidget {
  const _Tag({required this.label, required this.color});

  final String label;
  final Color color;

  @override
  Widget build(BuildContext context) => Container(
    padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
    decoration: BoxDecoration(
      color: color.withValues(alpha: 0.15),
      borderRadius: BorderRadius.circular(10),
    ),
    child: Text(
      label,
      style: TextStyle(color: color, fontSize: 12, fontWeight: FontWeight.w600),
    ),
  );
}

/// A centered icon + title (+ optional action) for the empty / error states.
class _Message extends StatelessWidget {
  const _Message({required this.icon, required this.title, this.action});

  final IconData icon;
  final String title;
  final Widget? action;

  @override
  Widget build(BuildContext context) => Center(
    child: Padding(
      padding: const EdgeInsets.all(32),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Icon(icon, size: 56, color: CymbraColors.onSurfaceVariant),
          const SizedBox(height: 20),
          Text(
            title,
            textAlign: TextAlign.center,
            style: const TextStyle(
              color: CymbraColors.onSurface,
              fontSize: 18,
              fontWeight: FontWeight.w700,
            ),
          ),
          if (action != null) ...[const SizedBox(height: 24), action!],
        ],
      ),
    ),
  );
}
