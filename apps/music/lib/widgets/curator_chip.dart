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
import '../services/streak_service.dart';
import '../state/curator_profile_notifier.dart';
import '../state/streak_notifier.dart';
import '../theme/cymbra_theme.dart';
import 'streak_sheet.dart';

/// The compact curator standing pill for the app bar (change: add-curation-
/// rewards): the level + lifetime points, plus the practice-streak flame
/// (change: add-practice-streak), with a notification dot when a deferred
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
          const _StreakSegment(),
        ],
      ),
    );
  }
}

/// The practice-streak segment of the standing pill (change: add-practice-streak,
/// task 5.1): a flame + the day count.
///
/// Always present, never hidden: at zero it renders **muted**, which is the hint
/// that a streak is there to be started. Hiding it would make the streak invisible
/// to exactly the users who have not begun one.
class _StreakSegment extends ConsumerWidget {
  const _StreakSegment();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    // While the standing loads (or if it failed), show the muted zero state
    // rather than a spinner inside a 20px-tall pill.
    final streak = ref.watch(streakProvider).valueOrNull ?? StreakView.none;
    final days = streak.current;
    final active = days > 0;
    return Semantics(
      label: l10n.streakChipTooltip(days),
      button: true,
      child: Tooltip(
        message: l10n.streakChipTooltip(days),
        // The way in to the streak (change: make-streak-recovery-reachable).
        // It was a label; the recovery offer had no home, so it had to open
        // itself at launch. Tapping opens the sheet **whether or not** a
        // recovery is available (design D1): a control that only sometimes
        // responds teaches players not to try it.
        child: InkWell(
          key: const Key('curator-chip-streak-tap'),
          onTap: () => unawaited(showStreakSheet(context)),
          borderRadius: BorderRadius.circular(12),
          child: Row(
            key: const Key('curator-chip-streak'),
            mainAxisSize: MainAxisSize.min,
            children: [
              const SizedBox(width: 8),
              Icon(
                Icons.local_fire_department,
                size: 16,
                // Muted at zero; lit — and warmer still once today is secured —
                // when there is a run to protect.
                color: active
                    ? (streak.playedToday
                          ? CymbraColors.primary
                          : CymbraColors.onSurface)
                    : CymbraColors.onSurfaceVariant,
              ),
              const SizedBox(width: 3),
              Text(
                l10n.streakChipDays(days),
                style: TextStyle(
                  color: active
                      ? CymbraColors.onSurface
                      : CymbraColors.onSurfaceVariant,
                  fontSize: 12.5,
                  fontWeight: active ? FontWeight.w700 : FontWeight.w500,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}
