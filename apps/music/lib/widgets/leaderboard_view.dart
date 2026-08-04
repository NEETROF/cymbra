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
import '../state/leaderboard.dart';
import '../state/leaderboard_notifier.dart';
import '../theme/cymbra_theme.dart';

/// A piece's leaderboards (change: add-play-leaderboards): a tempo/reaction toggle
/// over the ranked public entries, always showing the viewer their own rank +
/// personal best (even when private) and highlighting their own row when present.
///
/// Driven entirely through the injectable [leaderboardProvider], so it renders in
/// a test with a fake board — no native library, no live backend.
class LeaderboardView extends ConsumerStatefulWidget {
  const LeaderboardView({
    super.key,
    required this.scoreId,
    required this.title,
    this.initialMode = LeaderboardMode.tempo,
  });

  final String scoreId;
  final String title;
  final LeaderboardMode initialMode;

  @override
  ConsumerState<LeaderboardView> createState() => _LeaderboardViewState();
}

class _LeaderboardViewState extends ConsumerState<LeaderboardView> {
  late LeaderboardMode _mode = widget.initialMode;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final async = ref.watch(leaderboardProvider(widget.scoreId, _mode));

    return Column(
      mainAxisSize: MainAxisSize.min,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        // The title is optional — when embedded (e.g. the pre-play modal already
        // shows the piece name in its header), pass an empty title to skip it.
        if (widget.title.isNotEmpty) ...[
          Text(
            widget.title,
            style: const TextStyle(
              fontSize: 13,
              color: CymbraColors.onSurfaceVariant,
            ),
            textAlign: TextAlign.center,
          ),
          const SizedBox(height: 8),
        ],
        _modeToggle(l10n),
        const SizedBox(height: 12),
        Flexible(
          child: async.when(
            loading: () => const Padding(
              padding: EdgeInsets.all(24),
              child: Center(child: CircularProgressIndicator()),
            ),
            error: (_, _) => _errorState(l10n),
            data: (board) => _board(l10n, board),
          ),
        ),
      ],
    );
  }

  Widget _modeToggle(AppLocalizations l10n) => Center(
    child: SegmentedButton<LeaderboardMode>(
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

  Widget _errorState(AppLocalizations l10n) => Padding(
    padding: const EdgeInsets.all(20),
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
              ref.invalidate(leaderboardProvider(widget.scoreId, _mode)),
          child: Text(l10n.leaderboardRetry),
        ),
      ],
    ),
  );

  Widget _board(AppLocalizations l10n, Leaderboard board) {
    if (board.entries.isEmpty && board.own == null) {
      return Padding(
        padding: const EdgeInsets.all(24),
        child: Text(
          l10n.leaderboardEmpty,
          textAlign: TextAlign.center,
          style: const TextStyle(color: CymbraColors.onSurfaceVariant),
        ),
      );
    }
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        if (board.own != null) ...[
          _ownStanding(l10n, board.own!, board.total),
          const SizedBox(height: 8),
        ],
        Flexible(
          child: ListView.separated(
            shrinkWrap: true,
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

  /// The viewer's own rank + personal best — always shown (even when the viewer
  /// is private and therefore not listed to others).
  Widget _ownStanding(AppLocalizations l10n, LeaderboardEntry own, int total) =>
      Container(
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 10),
        decoration: BoxDecoration(
          color: CymbraColors.primaryContainer,
          borderRadius: BorderRadius.circular(10),
        ),
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
              l10n.leaderboardBest(own.subscore.round()),
              style: const TextStyle(
                fontSize: 15,
                fontWeight: FontWeight.w600,
                color: CymbraColors.primary,
              ),
            ),
          ],
        ),
      );

  Widget _entryRow(
    AppLocalizations l10n,
    LeaderboardEntry e, {
    required bool highlight,
  }) {
    final name = e.label ?? l10n.leaderboardAnonymous;
    return Container(
      color: highlight ? CymbraColors.primaryContainer : null,
      padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 8),
      child: Row(
        children: [
          SizedBox(
            width: 34,
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
            child: Text(
              highlight ? '$name (${l10n.leaderboardYou})' : name,
              maxLines: 1,
              overflow: TextOverflow.ellipsis,
              style: TextStyle(
                fontSize: 14,
                fontWeight: highlight ? FontWeight.w600 : FontWeight.w400,
                color: CymbraColors.onSurface,
              ),
            ),
          ),
          Text(
            '${e.subscore.round()}%',
            style: const TextStyle(
              fontSize: 14,
              fontWeight: FontWeight.w500,
              color: CymbraColors.primary,
            ),
          ),
        ],
      ),
    );
  }
}

/// Open a piece's leaderboards as a modal dialog (change: add-play-leaderboards).
/// Reachable from the score, the profile, and the end-of-session summary.
Future<void> showLeaderboard(
  BuildContext context, {
  required String scoreId,
  required String title,
  LeaderboardMode initialMode = LeaderboardMode.tempo,
}) {
  final l10n = AppLocalizations.of(context);
  final maxHeight = MediaQuery.of(context).size.height * 0.85;
  return showDialog<void>(
    context: context,
    builder: (context) => Dialog(
      backgroundColor: CymbraColors.surfaceContainerLow,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      insetPadding: const EdgeInsets.symmetric(horizontal: 24, vertical: 24),
      child: ConstrainedBox(
        constraints: BoxConstraints(maxWidth: 420, maxHeight: maxHeight),
        child: Padding(
          padding: const EdgeInsets.fromLTRB(20, 8, 20, 16),
          // Fill a STABLE height (max + Expanded) so switching modes — or the
          // loading→data / empty→populated swaps — never resizes the dialog; the
          // list scrolls within, matching the inline pre-play board.
          child: Column(
            mainAxisSize: MainAxisSize.max,
            children: [
              Row(
                children: [
                  Expanded(
                    child: Text(
                      l10n.leaderboardTitle,
                      style: const TextStyle(
                        fontSize: 16,
                        fontWeight: FontWeight.w600,
                        color: CymbraColors.onSurface,
                      ),
                    ),
                  ),
                  IconButton(
                    visualDensity: VisualDensity.compact,
                    icon: const Icon(
                      Icons.close,
                      color: CymbraColors.onSurface,
                    ),
                    onPressed: () => Navigator.of(context).pop(),
                  ),
                ],
              ),
              Expanded(
                child: LeaderboardView(
                  scoreId: scoreId,
                  title: title,
                  initialMode: initialMode,
                ),
              ),
            ],
          ),
        ),
      ),
    ),
  );
}
