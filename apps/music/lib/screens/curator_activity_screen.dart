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

/// The curator's recent points activity, shown ON DEMAND (change: add-curation-
/// rewards) — the profile only surfaces an "Activité récente ›" entry, and the
/// full list opens here, so the profile stays uncluttered. Each entry states its
/// amount and, for a deferred honesty/adjustment award, its source (consensus vs
/// moderator). Driven through [curatorProfileProvider].
class CuratorActivityScreen extends ConsumerWidget {
  const CuratorActivityScreen({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final async = ref.watch(curatorProfileProvider);
    return Scaffold(
      backgroundColor: CymbraColors.background,
      appBar: AppBar(
        title: Text(l10n.curatorRecentTitle),
        backgroundColor: CymbraColors.surfaceContainerLowest,
      ),
      body: SafeArea(
        child: async.when(
          loading: () => const Center(child: CircularProgressIndicator()),
          error: (_, _) => Center(
            child: Text(
              l10n.curatorLoadError,
              style: const TextStyle(color: CymbraColors.onSurfaceVariant),
            ),
          ),
          data: (r) => r.recent.isEmpty
              ? Center(
                  child: Text(
                    l10n.curatorRecentEmpty,
                    style: const TextStyle(
                      color: CymbraColors.onSurfaceVariant,
                    ),
                  ),
                )
              : ListView(
                  padding: const EdgeInsets.fromLTRB(12, 12, 12, 32),
                  children: [
                    for (final a in r.recent) _ActivityRow(activity: a),
                  ],
                ),
        ),
      ),
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
  // The play awards (change: add-play-rewards) — the ledger now has more than
  // one source of income, and the feed shows all of it.
  'performance' => Icons.piano_outlined,
  'practice' => Icons.event_available_outlined,
  'redeem' => Icons.card_giftcard,
  _ => Icons.stars,
};

String _kindLabel(AppLocalizations l10n, String kind) => switch (kind) {
  'coverage' => l10n.curatorActivityCoverage,
  'honesty' => l10n.curatorActivityHonesty,
  'adjustment' => l10n.curatorActivityAdjustment,
  'performance' => l10n.curatorActivityPerformance,
  'practice' => l10n.curatorActivityPractice,
  'redeem' => l10n.curatorActivityRedeem,
  // An award kind a newer server knows and this build does not: the raw key
  // would be an untranslated technical string in the UI, so fall back to the
  // neutral "points earned" wording instead.
  _ => l10n.curatorActivityOther,
};

String? _sourceLabel(AppLocalizations l10n, String? source) => switch (source) {
  'consensus' => l10n.curatorActivitySourceConsensus,
  'moderator' => l10n.curatorActivitySourceModerator,
  _ => null,
};
