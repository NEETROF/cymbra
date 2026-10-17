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

import '../courses/course_manifest.dart';
import '../l10n/gen/app_localizations.dart';
import '../screens/open_score.dart';
import '../state/course_completion_notifier.dart';
import '../state/session_summary.dart';
import '../theme/cymbra_theme.dart';
import 'piece_run.dart';

/// The `piece` step of a lesson (change: add-music-piece-lessons): the piece,
/// the settings the step imposes as chips, the bar to clear, and the outcome
/// of the last run — not started, visited or passed. *Play* opens the full
/// player; coming back with a result opens the lesson's gate (via
/// [onAnswered]) without advancing, so the learner reads the outcome, tries
/// again, or moves on. A step never traps: a missing or offline piece opens
/// the gate too.
class PieceBlockView extends ConsumerStatefulWidget {
  const PieceBlockView({
    super.key,
    required this.block,
    required this.courseId,
    required this.lessonTitle,
    required this.onAnswered,
    this.stepIndex = 0,
    this.soleStep = true,
  });

  /// This step's index among the lesson's shown steps — its own standing is
  /// kept under it, apart from the lesson's other piece steps.
  final int stepIndex;

  /// Whether this is the lesson's only piece step: then the course-level
  /// standing (synced across devices) is this step's too.
  final bool soleStep;

  final PieceBlock block;
  final String courseId;
  final String lessonTitle;

  /// Opens the lesson's gate with this run's first-try flag (passed = flawless).
  final void Function({required bool flawless}) onAnswered;

  @override
  ConsumerState<PieceBlockView> createState() => _PieceBlockViewState();
}

class _PieceBlockViewState extends ConsumerState<PieceBlockView> {
  /// The last run's overall synchronisation, this session.
  double? _lastPct;
  bool _passedThisSession = false;
  PieceOpenOutcome? _unavailable;
  bool _opening = false;

  PieceBlock get block => widget.block;

  Future<void> _open() async {
    if (_opening) return;
    setState(() {
      _opening = true;
      _unavailable = null;
    });
    // The runner records the run, writes today's ledger and celebrates a
    // pass once the learner is back here.
    final outcome = await runPieceStep(
      context,
      ref,
      courseId: widget.courseId,
      lessonTitle: widget.lessonTitle,
      block: block,
      stepKey: pieceStepKey(widget.courseId, widget.stepIndex),
      onResult: _onResult,
    );
    if (!mounted) return;
    setState(() {
      _opening = false;
      _unavailable = outcome == PieceOpenOutcome.opened ? null : outcome;
    });
    // Unavailable is not the learner's fault: the gate opens, the step is
    // simply not passed.
    if (outcome != PieceOpenOutcome.opened) widget.onAnswered(flawless: false);
  }

  /// Each finished run of this lesson's player, passed or not: the step's
  /// own standing, and the lesson's gate.
  void _onResult(SessionResult result) {
    final passed = result.overallSyncPct >= block.passPct;
    if (mounted) {
      setState(() {
        _lastPct = result.overallSyncPct;
        _passedThisSession = _passedThisSession || passed;
      });
    }
    widget.onAnswered(flawless: passed);
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    final lang = Localizations.localeOf(context).languageCode;
    final completion = ref.watch(courseCompletionProvider);
    // This step's standing from earlier sessions, so a learner coming back
    // sees where they stand before pressing Play: its own record, or — for a
    // lesson's only piece step — the course-level one, synced across devices.
    // Never another step's (a chain's second step is not passed by the first).
    final key = pieceStepKey(widget.courseId, widget.stepIndex);
    final stepBest = completion.stepBest[key];
    final earlierBest =
        stepBest ??
        (widget.soleStep ? completion.bestPct[widget.courseId] : null);
    final earlierPassed =
        (stepBest != null && stepBest >= block.passPct) ||
        (widget.soleStep && completion.isPassed(widget.courseId));
    final passed = _passedThisSession || earlierPassed;
    final best = _lastPct ?? earlierBest;

    final title = pieceTitleOf(ref, block, l10n, lang);
    final (stateLabel, stateColor, stateIcon) = switch ((passed, best)) {
      (true, final b?) => (
        l10n.piecePassed(b.round()),
        CymbraColors.tertiary,
        Icons.check_circle,
      ),
      (true, null) => (
        l10n.piecePassed(block.passPct),
        CymbraColors.tertiary,
        Icons.check_circle,
      ),
      (false, final b?) => (
        l10n.pieceVisited(b.round()),
        CymbraColors.onSurfaceVariant,
        Icons.replay_circle_filled,
      ),
      (false, null) => (
        l10n.pieceNotStarted,
        CymbraColors.onSurfaceVariant,
        Icons.radio_button_unchecked,
      ),
    };

    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Text(
          title,
          key: const Key('piece-title'),
          style: const TextStyle(
            color: CymbraColors.onSurface,
            fontSize: 20,
            fontWeight: FontWeight.w800,
          ),
        ),
        if (resolveInline(block.prompt, lang).isNotEmpty) ...[
          const SizedBox(height: 6),
          Text(
            resolveInline(block.prompt, lang),
            style: const TextStyle(
              color: CymbraColors.onSurfaceVariant,
              fontSize: 14,
              height: 1.4,
            ),
          ),
        ],
        const SizedBox(height: 12),
        // The imposed settings, visible before the run so nothing surprises.
        Wrap(
          spacing: 8,
          runSpacing: 8,
          children: [
            _chip(_handsLabel(l10n), CymbraColors.handRight),
            _chip(
              l10n.pieceSpeed((block.speed * 100).round()),
              CymbraColors.secondary,
            ),
            _chip(
              block.waitMode ? l10n.pieceWaitOn : l10n.pieceWaitOff,
              CymbraColors.primary,
            ),
            _chip(
              block.metronome ? l10n.pieceMetronomeOn : l10n.pieceMetronomeOff,
              CymbraColors.outline,
            ),
            _chip(_aidLabel(l10n), CymbraColors.primary),
          ],
        ),
        const SizedBox(height: 14),
        Container(
          padding: const EdgeInsets.all(14),
          decoration: BoxDecoration(
            color: CymbraColors.surfaceContainer,
            borderRadius: BorderRadius.circular(12),
            border: Border.all(color: CymbraColors.outlineVariant),
          ),
          child: Row(
            children: [
              Icon(stateIcon, color: stateColor, size: 28),
              const SizedBox(width: 12),
              Expanded(
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      l10n.pieceTarget(block.passPct),
                      style: const TextStyle(
                        color: CymbraColors.onSurface,
                        fontWeight: FontWeight.w700,
                      ),
                    ),
                    const SizedBox(height: 2),
                    Text(
                      stateLabel,
                      key: const Key('piece-state'),
                      style: TextStyle(
                        color: stateColor,
                        fontSize: 13,
                        fontWeight: FontWeight.w600,
                      ),
                    ),
                  ],
                ),
              ),
            ],
          ),
        ),
        if (_unavailable case final u?) ...[
          const SizedBox(height: 10),
          Row(
            key: const Key('piece-unavailable'),
            children: [
              const Icon(
                Icons.cloud_off,
                size: 18,
                color: CymbraColors.onSurfaceVariant,
              ),
              const SizedBox(width: 8),
              Expanded(
                child: Text(
                  u == PieceOpenOutcome.offline
                      ? l10n.pieceOffline
                      : l10n.pieceMissing,
                  style: const TextStyle(color: CymbraColors.onSurfaceVariant),
                ),
              ),
            ],
          ),
        ],
        const SizedBox(height: 16),
        FilledButton.icon(
          key: Key(best == null ? 'piece-start' : 'piece-retry'),
          style: FilledButton.styleFrom(
            backgroundColor: CymbraColors.primaryContainer,
          ),
          onPressed: _opening ? null : _open,
          icon: Icon(best == null ? Icons.play_arrow : Icons.replay),
          label: Text(best == null ? l10n.pieceStart : l10n.pieceTryAgain),
        ),
      ],
    );
  }

  /// The piece's name: the step's own title, else the bundled entry's, else a
  /// generic catalogue label (the hit is only resolved when the run opens).

  String _handsLabel(AppLocalizations l10n) => switch (block.hands) {
    LessonHands.right => l10n.pieceHandsRight,
    LessonHands.left => l10n.pieceHandsLeft,
    LessonHands.both => l10n.pieceHandsBoth,
  };

  String _aidLabel(AppLocalizations l10n) => switch (block.readingAid) {
    LessonReadingAid.off => l10n.pieceAidOff,
    LessonReadingAid.name => l10n.pieceAidName,
    LessonReadingAid.nameAndRhythm => l10n.pieceAidNameRhythm,
  };

  Widget _chip(String label, Color color) => Container(
    padding: const EdgeInsets.symmetric(horizontal: 10, vertical: 5),
    decoration: BoxDecoration(
      color: CymbraColors.surfaceContainerHigh,
      borderRadius: BorderRadius.circular(12),
      border: Border.all(color: color.withValues(alpha: 0.6)),
    ),
    child: Row(
      mainAxisSize: MainAxisSize.min,
      children: [
        Container(
          width: 8,
          height: 8,
          decoration: BoxDecoration(color: color, shape: BoxShape.circle),
        ),
        const SizedBox(width: 6),
        Text(
          label,
          style: const TextStyle(
            color: CymbraColors.onSurface,
            fontSize: 12,
            fontWeight: FontWeight.w600,
          ),
        ),
      ],
    ),
  );
}
