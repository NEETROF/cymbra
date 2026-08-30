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
import '../state/session_notifier.dart';
import '../state/streak_notifier.dart';
import '../theme/cymbra_theme.dart';

/// Opens the streak sheet (change: make-streak-recovery-reachable).
///
/// The home the recovery offer did not have. The buy-back is use-it-or-lose-it
/// — resuming restarts the run, so the pre-break count is gone the moment the
/// player does the thing they opened the app to do — and until now the only way
/// to reach it was a modal that opened itself at launch. Declining that modal
/// therefore cost the option rather than the interruption.
Future<void> showStreakSheet(BuildContext context) =>
    showModalBottomSheet<void>(
      context: context,
      backgroundColor: CymbraColors.surfaceContainerHigh,
      showDragHandle: true,
      builder: (_) => const StreakSheet(),
    );

/// The current standing, and the one action available on it.
///
/// Deliberately not a history view: a heatmap already exists elsewhere, and the
/// question this answers is "where am I, and can I do anything about it".
class StreakSheet extends ConsumerWidget {
  const StreakSheet({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    // A guest has no server-owned standing. Saying so beats rendering a zero,
    // which reads like a run that was lost rather than one never started.
    if (!ref.watch(canUseOnlineServicesProvider)) {
      return _Frame(
        key: const Key('streak-sheet-guest'),
        children: [Text(l10n.streakSheetGuest, style: _bodyStyle)],
      );
    }

    final streak = ref.watch(streakProvider).valueOrNull ?? StreakView.none;
    return _Frame(
      key: const Key('streak-sheet'),
      children: [
        Row(
          children: [
            Icon(
              Icons.local_fire_department,
              size: 28,
              color: streak.hasStreak
                  ? CymbraColors.primary
                  : CymbraColors.onSurfaceVariant,
            ),
            const SizedBox(width: 8),
            Expanded(
              child: Text(
                l10n.streakSheetCurrent(streak.current),
                style: const TextStyle(
                  color: CymbraColors.onSurface,
                  fontSize: 20,
                  fontWeight: FontWeight.w700,
                ),
              ),
            ),
          ],
        ),
        const SizedBox(height: 4),
        Text(l10n.streakSheetLongest(streak.longest), style: _mutedStyle),
        if (streak.playedToday) ...[
          const SizedBox(height: 4),
          Text(l10n.streakSheetSecured, style: _mutedStyle),
        ] else if (streak.atRisk) ...[
          const SizedBox(height: 4),
          Text(l10n.streakSheetAtRisk, style: _mutedStyle),
        ],
        if (streak.hasRecoveryToShow) ...[
          const SizedBox(height: 20),
          _Recovery(streak: streak),
        ],
        const SizedBox(height: 20),
        Align(
          alignment: Alignment.centerRight,
          child: TextButton(
            key: const Key('streak-sheet-close'),
            onPressed: () => Navigator.of(context).maybePop(),
            child: Text(l10n.streakSheetClose),
          ),
        ),
      ],
    );
  }
}

/// The buy-back, or why it cannot be taken.
///
/// Shown for an unaffordable break too (design D5): the wire reports what the
/// recovery *would* cost even when it refuses it, and "you are N points short"
/// is actionable where silence is not. An intact streak and one past the grace
/// window report no cost at all and reach nothing here.
class _Recovery extends ConsumerWidget {
  const _Recovery({required this.streak});

  final StreakView streak;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final short = streak.unaffordable;
    return Column(
      key: const Key('streak-sheet-recovery'),
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        Text(
          l10n.streakSheetRecoverBody(streak.recoverableStreak),
          style: _bodyStyle,
        ),
        if (short) ...[
          const SizedBox(height: 4),
          Text(
            key: const Key('streak-sheet-short'),
            l10n.streakSheetShort(streak.recoverCost),
            style: const TextStyle(color: CymbraColors.error, fontSize: 13),
          ),
        ],
        const SizedBox(height: 12),
        FilledButton(
          key: const Key('streak-sheet-recover'),
          // Disabled rather than hidden when the balance is short: the offer is
          // real, the price is knowable, and hiding it would leave the player
          // unable to tell "too poor" from "too late".
          onPressed: short
              ? null
              : () {
                  // Fire and react to state — never await the notifier's return
                  // and branch on it (architecture rule 3). The outcome reaches
                  // the player through the listener's snackbars, as it already
                  // did from the dialog this replaces.
                  unawaited(ref.read(streakProvider.notifier).recover());
                  Navigator.of(context).maybePop();
                },
          child: Text(l10n.streakRecoverConfirm(streak.recoverCost)),
        ),
      ],
    );
  }
}

class _Frame extends StatelessWidget {
  const _Frame({required this.children, super.key});

  final List<Widget> children;

  @override
  Widget build(BuildContext context) => SafeArea(
    // Scrollable, not merely min-sized: the tallest variant — a break with the
    // shortfall line — overflows a short phone-landscape sheet, and the app is
    // landscape-locked where it matters most.
    child: SingleChildScrollView(
      padding: const EdgeInsets.fromLTRB(20, 4, 20, 20),
      child: Column(
        mainAxisSize: MainAxisSize.min,
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          Text(
            AppLocalizations.of(context).streakSheetTitle,
            style: const TextStyle(
              color: CymbraColors.onSurfaceVariant,
              fontSize: 12,
              fontWeight: FontWeight.w700,
              letterSpacing: 0.4,
            ),
          ),
          const SizedBox(height: 12),
          ...children,
        ],
      ),
    ),
  );
}

const _bodyStyle = TextStyle(color: CymbraColors.onSurface, fontSize: 14);
const _mutedStyle = TextStyle(
  color: CymbraColors.onSurfaceVariant,
  fontSize: 13,
);
