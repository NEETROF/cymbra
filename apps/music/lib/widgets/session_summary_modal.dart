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

import '../l10n/gen/app_localizations.dart';
import '../state/performance_scoring_core.dart';
import '../state/session_summary.dart';
import '../theme/cymbra_theme.dart';

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
    barrierDismissible: true,
    builder: (context) => _SummaryDialog(result: result),
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

    return Dialog(
      backgroundColor: CymbraColors.surfaceContainerLow,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 420),
        child: Padding(
          padding: const EdgeInsets.all(20),
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

  Widget _actions(BuildContext context, AppLocalizations l10n) => Row(
    children: [
      Expanded(
        child: FilledButton(
          style: FilledButton.styleFrom(
            backgroundColor: CymbraColors.primaryContainer,
          ),
          onPressed: () => Navigator.of(context).pop(SummaryAction.replay),
          child: Text(l10n.summaryReplay),
        ),
      ),
      const SizedBox(width: 8),
      Expanded(
        child: OutlinedButton(
          onPressed: () => Navigator.of(context).pop(SummaryAction.retry),
          child: Text(l10n.summaryRetry),
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
