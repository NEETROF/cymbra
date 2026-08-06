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
import '../state/global_leaderboard.dart';
import '../state/global_leaderboard_notifier.dart';
import '../state/leaderboard.dart';
import '../theme/cymbra_theme.dart';
import 'profile_screen.dart';

/// The Community / Leaderboards **destination** (change: add-global-leaderboard):
/// the global tempo and reaction boards with a mode toggle and a season selector
/// (the live season plus the snapshotted past ones).
///
/// Viewing is open to any signed-in player — including a private or under-age one,
/// who is simply not *listed* to others but always sees their own rank and score.
/// Driven entirely through the injectable [globalLeaderboardProvider] /
/// [globalSeasonsProvider], so it renders in a test with a fake board — no native
/// library, no live backend.
class CommunityScreen extends ConsumerStatefulWidget {
  const CommunityScreen({super.key, this.initialMode = LeaderboardMode.tempo});

  final LeaderboardMode initialMode;

  @override
  ConsumerState<CommunityScreen> createState() => _CommunityScreenState();
}

class _CommunityScreenState extends ConsumerState<CommunityScreen> {
  late LeaderboardMode _mode = widget.initialMode;

  /// The selected season, or the empty string for the CURRENT one (the family key
  /// the notifier expects).
  String _seasonId = '';

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Scaffold(
      backgroundColor: CymbraColors.background,
      appBar: AppBar(
        title: Text(l10n.communityTitle),
        backgroundColor: CymbraColors.surfaceContainerLowest,
      ),
      body: SafeArea(
        child: Padding(
          padding: const EdgeInsets.fromLTRB(16, 12, 16, 0),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.stretch,
            children: [
              _modeToggle(l10n),
              const SizedBox(height: 12),
              _seasonSelector(l10n),
              const SizedBox(height: 12),
              Expanded(child: _board(l10n)),
            ],
          ),
        ),
      ),
    );
  }

  Widget _modeToggle(AppLocalizations l10n) => Center(
    child: SegmentedButton<LeaderboardMode>(
      key: const Key('community-mode'),
      segments: [
        ButtonSegment(
          value: LeaderboardMode.tempo,
          label: Text(l10n.leaderboardModeFree),
        ),
        ButtonSegment(
          value: LeaderboardMode.reaction,
          label: Text(l10n.leaderboardModeWait),
        ),
      ],
      selected: {_mode},
      showSelectedIcon: false,
      onSelectionChanged: (s) => setState(() => _mode = s.first),
    ),
  );

  /// The season selector: the live season plus every snapshotted past one. While
  /// the season list is loading (or if it fails), the screen simply stays on the
  /// current season rather than blocking the board.
  Widget _seasonSelector(AppLocalizations l10n) {
    final seasons = ref.watch(globalSeasonsProvider);
    final past = seasons.maybeWhen(
      data: (s) => s.pastSeasonIds,
      orElse: () => const <String>[],
    );
    if (past.isEmpty) return const SizedBox.shrink();
    return Row(
      mainAxisAlignment: MainAxisAlignment.center,
      children: [
        Text(
          l10n.communitySeasonLabel,
          style: const TextStyle(
            fontSize: 13,
            color: CymbraColors.onSurfaceVariant,
          ),
        ),
        const SizedBox(width: 8),
        DropdownButton<String>(
          key: const Key('community-season'),
          value: _seasonId,
          items: [
            DropdownMenuItem(
              value: '',
              child: Text(l10n.communitySeasonCurrent),
            ),
            for (final id in past)
              DropdownMenuItem(
                value: id,
                child: Text(l10n.communitySeasonPast(id)),
              ),
          ],
          onChanged: (value) => setState(() => _seasonId = value ?? ''),
        ),
      ],
    );
  }

  Widget _board(AppLocalizations l10n) {
    final async = ref.watch(globalLeaderboardProvider(_mode, _seasonId));
    return async.when(
      loading: () => const Center(child: CircularProgressIndicator()),
      error: (_, _) => _errorState(l10n),
      data: (board) => _content(l10n, board),
    );
  }

  Widget _errorState(AppLocalizations l10n) => Center(
    child: Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        Text(
          l10n.leaderboardError,
          textAlign: TextAlign.center,
          style: const TextStyle(color: CymbraColors.onSurfaceVariant),
        ),
        const SizedBox(height: 8),
        TextButton(
          onPressed: () =>
              ref.invalidate(globalLeaderboardProvider(_mode, _seasonId)),
          child: Text(l10n.leaderboardRetry),
        ),
      ],
    ),
  );

  Widget _content(AppLocalizations l10n, GlobalLeaderboard board) {
    if (board.entries.isEmpty && board.own == null) {
      return Center(
        child: Padding(
          padding: const EdgeInsets.all(24),
          child: Text(
            l10n.globalLeaderboardEmpty,
            textAlign: TextAlign.center,
            style: const TextStyle(color: CymbraColors.onSurfaceVariant),
          ),
        ),
      );
    }
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // The viewer's own rank + score — always shown, even when they are
        // private/under-age and therefore not listed to others.
        if (board.own != null) ...[
          _ownStanding(l10n, board.own!, board.total),
          const SizedBox(height: 8),
        ],
        Expanded(
          child: ListView.separated(
            itemCount: board.entries.length,
            separatorBuilder: (_, _) => const Divider(
              height: 1,
              color: CymbraColors.surfaceContainerLow,
            ),
            itemBuilder: (context, i) {
              final e = board.entries[i];
              return _entryRow(l10n, e, highlight: board.isViewer(e));
            },
          ),
        ),
      ],
    );
  }

  Widget _ownStanding(
    AppLocalizations l10n,
    GlobalLeaderboardEntry own,
    int total,
  ) => Material(
    key: const Key('community-own-standing'),
    color: CymbraColors.primaryContainer,
    borderRadius: BorderRadius.circular(10),
    clipBehavior: Clip.antiAlias,
    child: Padding(
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
      child: Row(
        children: [
          Expanded(
            child: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              children: [
                Text(
                  l10n.leaderboardYourStanding,
                  style: const TextStyle(
                    fontSize: 11,
                    color: CymbraColors.onSurfaceVariant,
                  ),
                ),
                Text(
                  l10n.leaderboardYourRank(own.rank, total),
                  style: const TextStyle(
                    fontSize: 15,
                    fontWeight: FontWeight.w600,
                    color: CymbraColors.onSurface,
                  ),
                ),
              ],
            ),
          ),
          Text(
            l10n.globalLeaderboardScore(formatGlobalScore(own.score)),
            style: const TextStyle(
              fontSize: 15,
              fontWeight: FontWeight.w600,
              color: CymbraColors.primary,
            ),
          ),
        ],
      ),
    ),
  );

  Widget _entryRow(
    AppLocalizations l10n,
    GlobalLeaderboardEntry e, {
    required bool highlight,
  }) {
    final name = e.label ?? l10n.leaderboardAnonymous;
    return Material(
      color: highlight ? CymbraColors.primaryContainer : Colors.transparent,
      child: InkWell(
        // Listed players are public + eligible, so their profile read succeeds.
        onTap: () => Navigator.of(context).push(
          MaterialPageRoute<void>(
            builder: (_) => ProfileScreen(userId: e.userId),
          ),
        ),
        child: Padding(
          padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
          child: Row(
            children: [
              SizedBox(
                width: 38,
                child: Text(
                  '#${e.rank}',
                  style: const TextStyle(
                    fontSize: 13,
                    fontWeight: FontWeight.w600,
                    color: CymbraColors.onSurfaceVariant,
                  ),
                ),
              ),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      highlight ? '$name (${l10n.leaderboardYou})' : name,
                      maxLines: 1,
                      overflow: TextOverflow.ellipsis,
                      style: TextStyle(
                        fontSize: 14,
                        fontWeight: highlight
                            ? FontWeight.w600
                            : FontWeight.w400,
                        color: CymbraColors.onSurface,
                      ),
                    ),
                    Text(
                      l10n.globalLeaderboardPieces(e.contributingPieces),
                      style: const TextStyle(
                        fontSize: 11,
                        color: CymbraColors.onSurfaceVariant,
                      ),
                    ),
                  ],
                ),
              ),
              Text(
                l10n.globalLeaderboardScore(formatGlobalScore(e.score)),
                style: const TextStyle(
                  fontSize: 14,
                  fontWeight: FontWeight.w500,
                  color: CymbraColors.primary,
                ),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

/// Render a global season score for display: one decimal, so a close race stays
/// readable without exposing the raw floating-point value.
String formatGlobalScore(double score) => score.toStringAsFixed(1);
