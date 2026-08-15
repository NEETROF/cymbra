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
import '../state/catalog_daily_access_notifier.dart';
import '../theme/cymbra_theme.dart';

/// The hub/library header chip of the freemium daily quota (change:
/// add-score-daily-access-rewards, design D8): "N free opens left · resets in
/// Xh Ymin". Renders nothing when the gate is off for the caller (signed out,
/// exempt, subscriber, no backend gate) — the common case today.
class CatalogAccessChip extends ConsumerWidget {
  const CatalogAccessChip({super.key, this.now});

  /// Injectable clock (tests); defaults to `DateTime.now()`.
  final DateTime Function()? now;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final access = ref.watch(catalogDailyAccessProvider).valueOrNull;
    if (access == null || !access.enabled) return const SizedBox.shrink();
    final l10n = AppLocalizations.of(context);
    final left = access.untilReset((now ?? DateTime.now)());
    final hours = left.inHours;
    final minutes = left.inMinutes.remainder(60);
    final exhausted = access.freeLeft == 0;
    return Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 4),
      child: Row(
        key: const Key('catalog-access-chip'),
        children: [
          Icon(
            exhausted ? Icons.lock_clock : Icons.today,
            size: 16,
            color: exhausted
                ? CymbraColors.primary
                : CymbraColors.onSurfaceVariant,
          ),
          const SizedBox(width: 6),
          Expanded(
            child: Text(
              '${l10n.catalogAccessChip(access.freeLeft)} · '
              '${l10n.catalogAccessResetsIn(hours, minutes)}',
              style: const TextStyle(
                color: CymbraColors.onSurfaceVariant,
                fontSize: 12,
              ),
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
            ),
          ),
        ],
      ),
    );
  }
}

/// The per-card daily-access mark (design D8): a check pill when the piece is
/// already open for the caller today (re-opening is free), a lock + cost pill
/// when the quota is exhausted and the piece is not open. Nothing when the gate
/// is off. Sits on the cover overlay — the bottom-left slot is taken by the
/// offline/status tags.
class CatalogAccessMark extends ConsumerWidget {
  const CatalogAccessMark({super.key, required this.catalogId});

  final String catalogId;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final access = ref.watch(catalogDailyAccessProvider).valueOrNull;
    if (access == null || !access.enabled) return const SizedBox.shrink();
    final l10n = AppLocalizations.of(context);
    if (access.isOpenToday(catalogId)) {
      return _Pill(
        key: const Key('catalog-access-open-today'),
        icon: Icons.check_circle,
        label: l10n.catalogAccessOpenedToday,
        color: CymbraColors.primary,
      );
    }
    if (access.freeLeft == 0) {
      return _Pill(
        key: const Key('catalog-access-locked'),
        icon: Icons.lock_outline,
        label: l10n.rewardShopCost(access.daySlotCost),
        color: CymbraColors.onSurfaceVariant,
      );
    }
    return const SizedBox.shrink();
  }
}

class _Pill extends StatelessWidget {
  const _Pill({
    super.key,
    required this.icon,
    required this.label,
    required this.color,
  });

  final IconData icon;
  final String label;
  final Color color;

  @override
  Widget build(BuildContext context) {
    return DecoratedBox(
      decoration: BoxDecoration(
        color: Colors.black.withValues(alpha: 0.55),
        borderRadius: BorderRadius.circular(999),
      ),
      child: Padding(
        padding: const EdgeInsets.symmetric(horizontal: 8, vertical: 4),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(icon, size: 12, color: color),
            const SizedBox(width: 4),
            Text(
              label,
              style: TextStyle(
                color: color,
                fontSize: 10,
                fontWeight: FontWeight.w600,
              ),
            ),
          ],
        ),
      ),
    );
  }
}
