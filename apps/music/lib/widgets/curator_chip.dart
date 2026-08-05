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
import '../state/curator_profile_notifier.dart';
import '../theme/cymbra_theme.dart';

/// The compact curator standing pill for the app bar (change: add-curation-
/// rewards): the level + lifetime points, with a notification dot when a deferred
/// honesty/adjustment award has landed since the profile was last opened.
///
/// Presentational only — it carries no tap handler. It is used as the **account
/// control's button** (`AccountMenu`): the pill replaces the plain person icon, so
/// tapping it opens the account menu (which routes to the profile, where the full
/// rewards live). It reads the reward providers directly (a value read), so it
/// stays live as level/points change.
class CuratorStandingPill extends ConsumerWidget {
  const CuratorStandingPill({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final rewards = ref.watch(curatorProfileProvider).valueOrNull;
    final hasUnseen = ref.watch(curatorHasUnseenAwardsProvider);

    final label = rewards == null
        ? l10n.curatorEntryTooltip
        : l10n.curatorChipLabel(rewards.level, rewards.lifetimePoints);

    return Container(
      key: const Key('curator-chip'),
      padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
      decoration: BoxDecoration(
        color: CymbraColors.surfaceContainerHigh,
        borderRadius: BorderRadius.circular(20),
      ),
      child: Row(
        mainAxisSize: MainAxisSize.min,
        children: [
          Stack(
            clipBehavior: Clip.none,
            children: [
              const Icon(
                Icons.workspace_premium,
                size: 18,
                color: CymbraColors.primary,
              ),
              if (hasUnseen)
                Positioned(
                  right: -2,
                  top: -2,
                  child: Container(
                    key: const Key('curator-chip-dot'),
                    width: 8,
                    height: 8,
                    decoration: const BoxDecoration(
                      color: CymbraColors.error,
                      shape: BoxShape.circle,
                    ),
                  ),
                ),
            ],
          ),
          const SizedBox(width: 6),
          Text(
            label,
            style: const TextStyle(
              color: CymbraColors.onSurface,
              fontSize: 12.5,
              fontWeight: FontWeight.w700,
            ),
          ),
        ],
      ),
    );
  }
}
