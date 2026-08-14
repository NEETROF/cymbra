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
import '../state/performance_scoring_core.dart';
import '../state/play_reward_cue.dart';
import '../state/post_play_rating_notifier.dart';
import '../state/session_summary.dart';
import '../theme/cymbra_theme.dart';
import '../theme/scoring_style.dart';
import 'leaderboard_view.dart';
import 'post_play_rating.dart';

/// What the player chose to do from the summary modal.
enum SummaryAction { replay, retry, close }

/// Shows the end-of-session summary for [result]. Returns the chosen
/// [SummaryAction] (or [SummaryAction.close] if dismissed). The caller handles
/// `replay` (it holds the score context the replay needs).
Future<SummaryAction> showSessionSummary(
  BuildContext context,
  SessionResult result,
) async {
  final action = await showDialog<SummaryAction>(
    context: context,
    // The player must make an explicit choice (see mistakes / retry / quit) —
    // no dismiss by tapping outside or the back button.
    barrierDismissible: false,
    builder: (context) =>
        PopScope(canPop: false, child: _SummaryDialog(result: result)),
  );
  return action ?? SummaryAction.close;
}

class _SummaryDialog extends StatelessWidget {
  const _SummaryDialog({required this.result});

  final SessionResult result;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final tier = (result.overallSyncPct ~/ 20).clamp(0, 4);
    final accent = _tierColor(tier);

    // Cap the height so a short phone-landscape viewport never pushes the action
    // buttons off-screen: the stats scroll, the buttons stay pinned below.
    final maxHeight = MediaQuery.of(context).size.height * 0.92;

    return Dialog(
      backgroundColor: CymbraColors.surfaceContainerLow,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      insetPadding: const EdgeInsets.symmetric(horizontal: 24, vertical: 16),
      child: ConstrainedBox(
        constraints: BoxConstraints(maxWidth: 420, maxHeight: maxHeight),
        child: Padding(
          padding: const EdgeInsets.fromLTRB(20, 8, 20, 16),
          child: Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              // Close cross (top-right) — same as Quit: leaves play mode.
              Align(
                alignment: Alignment.centerRight,
                child: IconButton(
                  visualDensity: VisualDensity.compact,
                  icon: const Icon(Icons.close, color: CymbraColors.onSurface),
                  onPressed: () =>
                      Navigator.of(context).pop(SummaryAction.close),
                ),
              ),
              Flexible(
                child: SingleChildScrollView(
                  child: Column(
                    mainAxisSize: MainAxisSize.min,
                    children: [
                      Text(
                        '${result.title} · ${_handsLabel(l10n, result.hands)}',
                        style: const TextStyle(
                          fontSize: 13,
                          color: CymbraColors.onSurfaceVariant,
                        ),
                        textAlign: TextAlign.center,
                      ),
                      Text(
                        '${result.overallSyncPct.round()}%',
                        style: TextStyle(
                          fontSize: 44,
                          fontWeight: FontWeight.w500,
                          color: accent,
                          height: 1.2,
                        ),
                      ),
                      Text(
                        l10n.summaryOverall,
                        style: const TextStyle(
                          fontSize: 12,
                          color: CymbraColors.outline,
                        ),
                      ),
                      // What this run earned (change: add-play-rewards) — shown
                      // right under the score, where the work happened. Renders
                      // nothing at all when the run earned nothing.
                      const _PointsCue(),
                      const SizedBox(height: 16),
                      _subScores(l10n),
                      const SizedBox(height: 16),
                      _dimensionBar(
                        l10n.summaryTiming,
                        result.timing,
                        CymbraColors.secondary,
                      ),
                      _dimensionBar(
                        l10n.summaryCorrect,
                        result.correctness,
                        CymbraColors.tertiary,
                      ),
                      _dimensionBar(
                        l10n.summarySustain,
                        result.sustain,
                        CymbraColors.handLeft,
                      ),
                      const SizedBox(height: 12),
                      _verdictRow(l10n),
                      // Post-session standing on this piece for the run's mode(s),
                      // with a link to the full board (change: add-play-
                      // leaderboards). A private/under-age player still sees their
                      // own rank + PB here (the server always returns own standing).
                      _PostSessionStanding(result: result),
                    ],
                  ),
                ),
              ),
              // Rate the piece just played (change: add-post-play-rating-prompt).
              // Pinned with the actions rather than inside the scroll area, so it
              // is never scrolled out of sight — but it is NOT a fourth choice:
              // it never dismisses the modal, and ignoring it costs nothing.
              const _SummaryRating(),
              const SizedBox(height: 16),
              _actions(context, l10n),
            ],
          ),
        ),
      ),
    );
  }

  Widget _subScores(AppLocalizations l10n) {
    Widget box(
      String label,
      String sub,
      double? pct,
      Color color,
      String? detail,
    ) => Expanded(
      child: Container(
        padding: const EdgeInsets.all(8),
        decoration: BoxDecoration(
          color: CymbraColors.background,
          borderRadius: BorderRadius.circular(10),
        ),
        child: Column(
          children: [
            Text(
              label,
              style: const TextStyle(fontSize: 11, color: CymbraColors.outline),
            ),
            Text(
              pct == null ? '—' : '${pct.round()}%',
              style: TextStyle(
                fontSize: 18,
                fontWeight: FontWeight.w500,
                color: color,
              ),
            ),
            Text(
              sub,
              style: const TextStyle(fontSize: 10, color: CymbraColors.outline),
            ),
            if (detail != null)
              Text(
                detail,
                style: TextStyle(
                  fontSize: 11,
                  fontWeight: FontWeight.w500,
                  color: color,
                ),
              ),
          ],
        ),
      ),
    );

    // A pure run shows only its relevant sub-score; a mixed run shows both.
    final showTempo = result.freeSyncPct != null;
    final showReaction = result.waitSyncPct != null;
    return Row(
      children: [
        if (showTempo)
          box(
            l10n.summaryTempo,
            l10n.summaryTempoSub,
            result.freeSyncPct,
            CymbraColors.handRight,
            _tempoTendency(l10n),
          ),
        if (showTempo && showReaction) const SizedBox(width: 8),
        if (showReaction)
          box(
            l10n.summaryReaction,
            l10n.summaryReactionSub,
            result.waitSyncPct,
            CymbraColors.primary,
            _reactionTendency(l10n),
          ),
      ],
    );
  }

  /// The tempo box's average early/late tendency (rushes vs. drags), or null.
  String? _tempoTendency(AppLocalizations l10n) {
    final off = result.avgFreeOffsetMs;
    if (off == null) return null;
    final ms = off.abs().round();
    return off < 0 ? l10n.summaryAvgEarly(ms) : l10n.summaryAvgLate(ms);
  }

  /// The reaction box's average reaction time, or null.
  String? _reactionTendency(AppLocalizations l10n) {
    final r = result.avgReactionMs;
    return r == null ? null : l10n.summaryAvgReaction(r.round());
  }

  Widget _dimensionBar(String label, double value, Color color) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 4),
    child: Row(
      children: [
        SizedBox(
          width: 96,
          child: Text(
            label,
            style: const TextStyle(
              fontSize: 12,
              color: CymbraColors.onSurfaceVariant,
            ),
          ),
        ),
        Expanded(
          child: ClipRRect(
            borderRadius: BorderRadius.circular(5),
            child: LinearProgressIndicator(
              value: value.clamp(0.0, 1.0),
              minHeight: 7,
              backgroundColor: CymbraColors.background,
              valueColor: AlwaysStoppedAnimation(color),
            ),
          ),
        ),
      ],
    ),
  );

  Widget _verdictRow(AppLocalizations l10n) {
    final perfect = result.verdictCounts[TimingVerdict.perfect] ?? 0;
    final off =
        (result.verdictCounts[TimingVerdict.good] ?? 0) +
        (result.verdictCounts[TimingVerdict.early] ?? 0) +
        (result.verdictCounts[TimingVerdict.late] ?? 0);
    final missed = result.verdictCounts[TimingVerdict.missed] ?? 0;
    return DefaultTextStyle(
      style: const TextStyle(
        fontSize: 12,
        color: CymbraColors.onSurfaceVariant,
      ),
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Text(l10n.summaryPerfect(perfect)),
          Text(l10n.summaryOff(off)),
          Text(l10n.summaryMissed(missed)),
          Text(l10n.summaryBestCombo(result.bestCombo)),
        ],
      ),
    );
  }

  Widget _actions(BuildContext context, AppLocalizations l10n) => Column(
    children: [
      // Primary action: see the mistakes on the score. Quit is the close cross.
      SizedBox(
        width: double.infinity,
        child: FilledButton(
          style: FilledButton.styleFrom(
            backgroundColor: CymbraColors.primaryContainer,
          ),
          onPressed: () => Navigator.of(context).pop(SummaryAction.replay),
          child: Text(l10n.summaryReplay),
        ),
      ),
      const SizedBox(height: 8),
      // Sole secondary action. Drilling a passage is no longer offered here: the
      // range is picked in the play screen itself (long-press the rewind), so the
      // summary stays about the run that just ended.
      SizedBox(
        width: double.infinity,
        child: OutlinedButton(
          onPressed: () => Navigator.of(context).pop(SummaryAction.retry),
          child: Text(
            l10n.summaryRetry,
            maxLines: 1,
            overflow: TextOverflow.ellipsis,
          ),
        ),
      ),
    ],
  );

  static Color _tierColor(int tier) => switch (tier) {
    0 => CymbraColors.error,
    1 => CymbraColors.handLeft,
    2 => CymbraColors.secondary,
    3 => CymbraColors.tertiary,
    _ => CymbraColors.primary,
  };

  static String _handsLabel(AppLocalizations l10n, String hands) =>
      l10n.summaryHands(hands);
}

/// The "+N points" cue for the run just finished (change: add-play-rewards).
///
/// The amount is not known when the modal opens: the run is captured into the
/// durable outbox and the server decides what it is worth, so this appears the
/// moment the ack lands — which may be before the modal opened or a beat after.
/// It renders **nothing** when the run earned nothing (below the quality floor,
/// a piece that has already paid out, the daily cap reached) and nothing at all
/// offline, so the summary is byte-for-byte its old self in those cases.
class _PointsCue extends ConsumerWidget {
  const _PointsCue();

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final points = ref.watch(playRewardCueProvider.select((s) => s.points));
    if (points <= 0) return const SizedBox.shrink();
    final l10n = AppLocalizations.of(context);
    // Gamified accent: the flawless feedback-tier colour, matching the reward
    // celebration — not a plain success green.
    final accent = 4.tierColor;
    return Padding(
      padding: const EdgeInsets.only(top: 8),
      child: Container(
        key: const Key('summary-points-cue'),
        padding: const EdgeInsets.symmetric(horizontal: 12, vertical: 5),
        decoration: BoxDecoration(
          color: accent.withValues(alpha: 0.15),
          borderRadius: BorderRadius.circular(999),
        ),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            Icon(Icons.stars_rounded, size: 15, color: accent),
            const SizedBox(width: 6),
            Text(
              l10n.playPointsCue(points),
              style: TextStyle(
                fontSize: 13,
                fontWeight: FontWeight.w700,
                color: accent,
              ),
            ),
          ],
        ),
      ),
    );
  }
}

/// The summary's rating affordance (change: add-post-play-rating-prompt): the
/// shared star row, mounted only when the played score is eligible (signed in, a
/// public-catalog score, not already rated, not already offered). A finished run
/// always satisfies the played-enough term, hence `reachedEnd: true`. Renders
/// nothing otherwise, so the modal is byte-for-byte its old self for a bundled
/// score or a guest.
class _SummaryRating extends ConsumerStatefulWidget {
  const _SummaryRating();

  @override
  ConsumerState<_SummaryRating> createState() => _SummaryRatingState();
}

class _SummaryRatingState extends ConsumerState<_SummaryRating> {
  /// Eligibility is **latched** so a rating read that resolves after the modal
  /// opened can still turn the row on, without it flickering off again the moment
  /// the answer changes the provider's mind.
  bool _shown = false;

  /// Set once the player has answered — rated or refused. The row then goes away
  /// for good; the outcome is confirmed by a toast, so nothing has to stay on
  /// screen to report it, and the statistics get their height back.
  bool _answered = false;

  @override
  Widget build(BuildContext context) {
    if (_answered) return const SizedBox.shrink();
    _shown |= ref.watch(postPlayRatingEligibleProvider(reachedEnd: true));
    if (!_shown) return const SizedBox.shrink();
    return Padding(
      padding: const EdgeInsets.only(top: 8),
      child: PostPlayRatingRow(
        compact: true,
        onAnswered: () => setState(() => _answered = true),
      ),
    );
  }
}

/// The post-session standing block (change: add-play-leaderboards): for each mode
/// the run produced, the player's rank + personal best on that piece, plus a link
/// to open the full board. Reads through [leaderboardProvider], so a test drives
/// it with a fake board. A piece with no shared board (a user upload) simply shows
/// no standing lines; the link still opens the (empty) board dialog.
class _PostSessionStanding extends ConsumerWidget {
  const _PostSessionStanding({required this.result});

  final SessionResult result;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final modes = <LeaderboardMode>[
      if (result.freeSyncPct != null) LeaderboardMode.tempo,
      if (result.waitSyncPct != null) LeaderboardMode.reaction,
    ];
    if (modes.isEmpty) return const SizedBox.shrink();
    return Column(
      mainAxisSize: MainAxisSize.min,
      children: [
        const SizedBox(height: 12),
        for (final mode in modes)
          _StandingLine(pieceId: result.pieceId, mode: mode),
        TextButton.icon(
          icon: const Icon(Icons.emoji_events, size: 18),
          label: Text(l10n.leaderboardOpenFull),
          onPressed: () => showLeaderboard(
            context,
            scoreId: result.pieceId,
            title: result.title,
            initialMode: modes.first,
          ),
        ),
      ],
    );
  }
}

/// One mode's standing line in the summary — "Tempo · Rank #3 of 10 · Best 82%".
/// Renders nothing until the board loads and the viewer has an own standing on it.
class _StandingLine extends ConsumerWidget {
  const _StandingLine({required this.pieceId, required this.mode});

  final String pieceId;
  final LeaderboardMode mode;

  @override
  Widget build(BuildContext context, WidgetRef ref) {
    final l10n = AppLocalizations.of(context);
    final board = ref.watch(leaderboardProvider(pieceId, mode));
    final own = board.valueOrNull?.own;
    if (own == null) return const SizedBox.shrink();
    final total = board.valueOrNull?.total ?? 0;
    final modeLabel = mode == LeaderboardMode.tempo
        ? l10n.leaderboardModeFree
        : l10n.leaderboardModeWait;
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 2),
      child: Text(
        '$modeLabel · ${l10n.leaderboardYourRank(own.rank, total)} · ${l10n.leaderboardBest(own.subscore.round())}',
        style: const TextStyle(
          fontSize: 12,
          color: CymbraColors.onSurfaceVariant,
        ),
        textAlign: TextAlign.center,
      ),
    );
  }
}
