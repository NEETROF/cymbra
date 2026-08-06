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
import '../screens/community_screen.dart';
import '../state/global_leaderboard.dart';
import '../state/global_leaderboard_notifier.dart';
import '../state/leaderboard.dart';
import '../theme/cymbra_theme.dart';

/// The signed-in player's GLOBAL standing on their own profile (change:
/// add-global-leaderboard, task 4.2): their current-season rank and score on each
/// mode, and an entry into the Community/Leaderboards destination.
///
/// The server always returns a caller their OWN rank among the public entries, so
/// a **private or under-age** player sees their standing here exactly like anyone
/// else — they are simply not listed to others. The RPC is caller-scoped, so this
/// section is shown only on the own profile.
class GlobalStandingSection extends ConsumerWidget {
  const GlobalStandingSection({super.key});

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final theme = Theme.of(context);
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(l10n.globalStandingTitle, style: theme.textTheme.titleMedium),
        const SizedBox(height: 8),
        for (final mode in LeaderboardMode.values) ...[
          _ModeStanding(mode: mode),
          const SizedBox(height: 4),
        ],
        const SizedBox(height: 4),
        TextButton.icon(
          key: const Key('global-standing-open'),
          icon: const Icon(Icons.leaderboard_outlined, size: 18),
          label: Text(l10n.globalStandingOpen),
          onPressed: () => Navigator.of(context).push(
            MaterialPageRoute<void>(builder: (_) => const CommunityScreen()),
          ),
        ),
      ],
    );
  }
}

/// One mode's row: the player's current-season rank + score, or a nudge when they
/// have not scored this season yet.
class _ModeStanding extends ConsumerWidget {
  const _ModeStanding({required this.mode});

  final LeaderboardMode mode;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    // The empty season id = the CURRENT season.
    final async = ref.watch(globalLeaderboardProvider(mode, ''));
    final label = switch (mode) {
      LeaderboardMode.tempo => l10n.leaderboardModeFree,
      LeaderboardMode.reaction => l10n.leaderboardModeWait,
    };
    return async.when(
      // A failed read must not break the profile — the section stays quiet.
      loading: () => const SizedBox.shrink(),
      error: (_, _) => const SizedBox.shrink(),
      data: (board) => _row(l10n, label, board),
    );
  }

  Widget _row(AppLocalizations l10n, String label, GlobalLeaderboard board) {
    final own = board.own;
    return Row(
      children: [
        Expanded(
          child: Text(
            label,
            style: const TextStyle(
              fontSize: 13,
              color: CymbraColors.onSurfaceVariant,
            ),
          ),
        ),
        if (own == null)
          Flexible(
            child: Text(
              l10n.globalStandingNone,
              textAlign: TextAlign.end,
              style: const TextStyle(
                fontSize: 12,
                color: CymbraColors.onSurfaceVariant,
              ),
            ),
          )
        else ...[
          Text(
            l10n.leaderboardYourRank(own.rank, board.total),
            style: const TextStyle(
              fontSize: 13,
              fontWeight: FontWeight.w600,
              color: CymbraColors.onSurface,
            ),
          ),
          const SizedBox(width: 12),
          Text(
            l10n.globalLeaderboardScore(formatGlobalScore(own.score)),
            style: const TextStyle(
              fontSize: 13,
              fontWeight: FontWeight.w600,
              color: CymbraColors.primary,
            ),
          ),
        ],
      ],
    );
  }
}
