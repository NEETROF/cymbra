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
import '../screens/curator_profile_screen.dart';
import '../state/curator_profile_notifier.dart';
import '../theme/cymbra_theme.dart';

/// A compact persistent chip for the app bars (change: add-curation-rewards):
/// shows the curator's level + lifetime points, with a notification dot when a
/// deferred honesty/adjustment award has landed since the profile was last
/// opened. Tapping it opens the full curator profile.
///
/// It reads the curator profile notifier directly (a value read, not a
/// side-effectful service call), so it stays live as points/level change.
class CuratorChip extends ConsumerWidget {
  const CuratorChip({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final rewards = ref.watch(curatorProfileProvider).valueOrNull;
    final hasUnseen = ref.watch(curatorHasUnseenAwardsProvider);

    final label = rewards == null
        ? l10n.curatorEntryTooltip
        : l10n.curatorChipLabel(rewards.level, rewards.lifetimePoints);

    return Padding(
      key: const Key('curator-chip'),
      padding: const EdgeInsets.symmetric(horizontal: 4, vertical: 8),
      child: Tooltip(
        message: l10n.curatorEntryTooltip,
        child: Material(
          color: CymbraColors.surfaceContainerHigh,
          borderRadius: BorderRadius.circular(20),
          child: InkWell(
            borderRadius: BorderRadius.circular(20),
            onTap: () => _open(context),
            child: Padding(
              padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 6),
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
            ),
          ),
        ),
      ),
    );
  }

  static void _open(BuildContext context) => Navigator.of(
    context,
  ).push(MaterialPageRoute<void>(builder: (_) => const CuratorProfileScreen()));
}
