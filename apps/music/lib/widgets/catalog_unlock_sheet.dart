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

import 'dart:async';

import 'package:flutter/material.dart';
import 'package:flutter_riverpod/flutter_riverpod.dart';

import '../l10n/gen/app_localizations.dart';
import '../screens/plan_screen.dart';
import '../services/catalog_access_state.dart';
import '../state/catalog_daily_access_notifier.dart';
import '../state/score_catalog.dart';
import '../state/score_preview_playback.dart';
import '../theme/cymbra_theme.dart';

/// The unlock flow of a catalog piece refused by the daily quota (change:
/// add-score-daily-access-rewards, design D8): a bottom sheet naming the piece,
/// offering to **listen to the audio teaser** (never the MusicXML), to **unlock it
/// for today** with points after an explicit confirmation, and carrying the
/// subscription **upsell placeholder**. Confirming fires the unlock notifier;
/// the outcome arrives as state and is surfaced by `CatalogUnlockListener`.
Future<void> showCatalogUnlockSheet(
  BuildContext context,
  WidgetRef ref,
  CatalogEntry entry,
) => showModalBottomSheet<void>(
  context: context,
  backgroundColor: CymbraColors.surfaceContainerHigh,
  showDragHandle: true,
  isScrollControlled: true,
  builder: (_) => _CatalogUnlockSheet(entry: entry),
);

class _CatalogUnlockSheet extends ConsumerStatefulWidget {
  const _CatalogUnlockSheet({required this.entry});

  final CatalogEntry entry;

  @override
  ConsumerState<_CatalogUnlockSheet> createState() =>
      _CatalogUnlockSheetState();
}

class _CatalogUnlockSheetState extends ConsumerState<_CatalogUnlockSheet> {
  late final ScorePreviewPlayback _playback;

  @override
  void initState() {
    super.initState();
    // Resolved once so `dispose` never touches `ref` (riverpod_lint rule).
    _playback = ref.read(scorePreviewPlaybackProvider.notifier);
  }

  @override
  void dispose() {
    // Stop the teaser when the sheet goes away (fire-and-forget).
    unawaited(_playback.stop());
    super.dispose();
  }

  void _toggleAudition() {
    final catalogId = widget.entry.catalogId;
    if (catalogId == null) return;
    // Fire-and-observe: playing/loading/missing/failure are state; the failure
    // snackbar lives in the library listeners.
    unawaited(_playback.toggle(catalogId));
  }

  void _confirmUnlock() {
    // Fire-and-observe: the outcome is state, surfaced by the listener widget.
    unawaited(ref.read(catalogUnlockProvider.notifier).unlock(widget.entry));
    Navigator.of(context).pop();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final entry = widget.entry;
    final access = ref.watch(catalogDailyAccessProvider).valueOrNull;
    final busy = ref.watch(catalogUnlockProvider.select((s) => s.busy));
    final catalogId = entry.catalogId ?? '';
    final playback = ref.watch(scorePreviewPlaybackProvider);
    final playing = playback.playingId == catalogId;
    final loading = playback.loadingId == catalogId;
    final canListen = entry.hasPreview && !playback.missing.contains(catalogId);
    final cost = access?.daySlotCost ?? 0;
    final balance = access?.spendableBalance ?? 0;
    final affordable = access?.canAffordDaySlot ?? false;
    final missing = (cost - balance).clamp(0, cost);
    return SafeArea(
      child: Padding(
        padding: const EdgeInsets.fromLTRB(24, 0, 24, 24),
        child: Column(
          key: const Key('catalog-unlock-sheet'),
          mainAxisSize: MainAxisSize.min,
          crossAxisAlignment: CrossAxisAlignment.stretch,
          children: [
            Row(
              children: [
                const Icon(Icons.lock_outline, color: CymbraColors.primary),
                const SizedBox(width: 8),
                Expanded(
                  child: Text(
                    l10n.catalogUnlockTitle,
                    style: Theme.of(context).textTheme.titleLarge,
                  ),
                ),
              ],
            ),
            const SizedBox(height: 12),
            Text(entry.title, style: Theme.of(context).textTheme.titleMedium),
            if (entry.composer.isNotEmpty)
              Text(
                entry.composer,
                style: const TextStyle(color: CymbraColors.onSurfaceVariant),
              ),
            const SizedBox(height: 12),
            Text(l10n.catalogUnlockBody(access?.freeQuota ?? 0)),
            const SizedBox(height: 16),
            OutlinedButton.icon(
              key: const Key('catalog-unlock-listen'),
              onPressed: canListen ? _toggleAudition : null,
              icon: loading
                  ? const SizedBox.square(
                      dimension: 16,
                      child: CircularProgressIndicator(strokeWidth: 2),
                    )
                  : Icon(playing ? Icons.stop : Icons.play_arrow),
              label: Text(
                !canListen
                    ? l10n.catalogUnlockListenUnavailable
                    : playing || loading
                    ? l10n.catalogUnlockStop
                    : l10n.catalogUnlockListen,
              ),
            ),
            const SizedBox(height: 12),
            _BalanceLine(access: access, missing: missing),
            const SizedBox(height: 8),
            FilledButton.icon(
              key: const Key('catalog-unlock-confirm'),
              onPressed: affordable && !busy ? _confirmUnlock : null,
              icon: const Icon(Icons.lock_open),
              label: Text(l10n.catalogUnlockConfirm(cost)),
            ),
            if (access?.upsell ?? false) ...[
              const SizedBox(height: 12),
              // The subscription upsell (change: add-premium-subscription): a
              // real call to action to the plan screen, whose purchase entry is
              // decided server-side for this platform.
              OutlinedButton.icon(
                key: const Key('catalog-unlock-upsell'),
                onPressed: () {
                  Navigator.of(context).pop();
                  openPlanScreen(context);
                },
                icon: const Icon(Icons.workspace_premium, size: 18),
                label: Text(l10n.catalogUnlockUpsellCta),
              ),
            ],
            const SizedBox(height: 4),
            TextButton(
              key: const Key('catalog-unlock-dismiss'),
              onPressed: () => Navigator.of(context).pop(),
              child: Text(l10n.catalogUnlockDismiss),
            ),
          ],
        ),
      ),
    );
  }
}

class _BalanceLine extends StatelessWidget {
  const _BalanceLine({required this.access, required this.missing});

  final CatalogAccessState? access;
  final int missing;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    if (access == null) return const SizedBox.shrink();
    return Column(
      children: [
        Text(
          l10n.catalogUnlockBalance(access!.spendableBalance),
          key: const Key('catalog-unlock-balance'),
          style: const TextStyle(color: CymbraColors.onSurfaceVariant),
        ),
        if (!access!.canAffordDaySlot)
          Text(
            l10n.catalogUnlockShort(missing),
            key: const Key('catalog-unlock-short'),
            style: const TextStyle(color: CymbraColors.onSurfaceVariant),
          ),
      ],
    );
  }
}
