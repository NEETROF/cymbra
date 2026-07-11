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

/// How a judged note is classified for the replay highlight.
enum ReplayMark { correct, mistimed, shortSustain, missed, wrong }

/// Classifies a [NoteJudgment] for the replay: correct notes are left un-marked,
/// everything else is highlighted by the kind of mistake.
ReplayMark markFor(NoteJudgment j) {
  if (j.wrong) return ReplayMark.wrong;
  if (j.verdict == TimingVerdict.missed) return ReplayMark.missed;
  if (j.verdict == TimingVerdict.early || j.verdict == TimingVerdict.late) {
    return ReplayMark.mistimed;
  }
  if (j.sustainRatio < 0.5) return ReplayMark.shortSustain;
  return ReplayMark.correct;
}

/// Opens the mistake replay for [result] over the horizontal score. Driven
/// entirely by the recorded per-note judgments — no live input.
Future<void> showMistakeReplay(BuildContext context, SessionResult result) =>
    showDialog<void>(
      context: context,
      builder: (context) => _ReplayDialog(result: result),
    );

class _ReplayDialog extends StatefulWidget {
  const _ReplayDialog({required this.result});

  final SessionResult result;

  @override
  State<_ReplayDialog> createState() => _ReplayDialogState();
}

class _ReplayDialogState extends State<_ReplayDialog>
    with SingleTickerProviderStateMixin {
  late final AnimationController _scrub = AnimationController(
    vsync: this,
    duration: const Duration(seconds: 6),
  )..forward();

  @override
  void dispose() {
    _scrub.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    return Dialog.fullscreen(
      backgroundColor: CymbraColors.background,
      child: Column(
        children: [
          Padding(
            padding: const EdgeInsets.all(12),
            child: Row(
              children: [
                Text(
                  l10n.replayTitle,
                  style: const TextStyle(
                    fontSize: 16,
                    fontWeight: FontWeight.w500,
                    color: CymbraColors.onSurface,
                  ),
                ),
                const Spacer(),
                IconButton(
                  icon: const Icon(Icons.close, color: CymbraColors.onSurface),
                  onPressed: () => Navigator.of(context).pop(),
                ),
              ],
            ),
          ),
          Expanded(
            child: AnimatedBuilder(
              animation: _scrub,
              builder: (context, _) => CustomPaint(
                size: Size.infinite,
                painter: MistakeReplayPainter(
                  judgments: widget.result.notes,
                  progress: _scrub.value,
                ),
              ),
            ),
          ),
          _legend(l10n),
        ],
      ),
    );
  }

  Widget _legend(AppLocalizations l10n) => Padding(
    padding: const EdgeInsets.all(12),
    child: Wrap(
      spacing: 16,
      runSpacing: 8,
      children: [
        _legendItem(CymbraColors.error, l10n.replayMissed),
        _legendItem(CymbraColors.handLeft, l10n.replayMistimed),
        _legendItem(CymbraColors.primary, l10n.replayShortSustain),
        _legendItem(
          CymbraColors.error.withValues(alpha: 0.5),
          l10n.replayWrong,
        ),
      ],
    ),
  );

  Widget _legendItem(Color color, String label) => Row(
    mainAxisSize: MainAxisSize.min,
    children: [
      Container(
        width: 12,
        height: 12,
        decoration: BoxDecoration(
          color: color,
          borderRadius: BorderRadius.circular(6),
        ),
      ),
      const SizedBox(width: 6),
      Text(
        label,
        style: const TextStyle(
          fontSize: 12,
          color: CymbraColors.onSurfaceVariant,
        ),
      ),
    ],
  );
}

/// Draws the judged notes on a horizontal timeline (x = onset time, y = pitch),
/// highlighting mistakes by [ReplayMark]. A playhead scrubs across at [progress]
/// (0→1). Correct notes render as faint dots; mistakes are colour-coded.
class MistakeReplayPainter extends CustomPainter {
  final List<NoteJudgment> judgments;
  final double progress;

  const MistakeReplayPainter({required this.judgments, required this.progress});

  @override
  void paint(Canvas canvas, Size size) {
    // Reference staff lines.
    final linePaint = Paint()
      ..color = CymbraColors.surfaceContainerHigh
      ..strokeWidth = 1;
    for (var i = 1; i <= 5; i++) {
      final y = size.height * i / 6;
      canvas.drawLine(Offset(0, y), Offset(size.width, y), linePaint);
    }

    if (judgments.isEmpty) return;

    var maxStart = 1;
    var minPitch = 127;
    var maxPitch = 0;
    for (final j in judgments) {
      if (j.startMs > maxStart) maxStart = j.startMs;
      if (!j.wrong) {
        if (j.pitch < minPitch) minPitch = j.pitch;
        if (j.pitch > maxPitch) maxPitch = j.pitch;
      }
    }
    if (minPitch > maxPitch) {
      minPitch = 60;
      maxPitch = 72;
    }
    final span = (maxStart + 500).toDouble();
    final pitchSpan = (maxPitch - minPitch).clamp(1, 127);
    const pad = 24.0;

    double xOf(int startMs) => (startMs / span) * size.width;
    double yOf(int pitch) =>
        size.height -
        pad -
        ((pitch - minPitch) / pitchSpan) * (size.height - 2 * pad);

    for (final j in judgments) {
      final x = xOf(j.startMs);
      final y = yOf(j.pitch);
      final center = Offset(x, y);
      switch (markFor(j)) {
        case ReplayMark.correct:
          canvas.drawCircle(
            center,
            5,
            Paint()..color = CymbraColors.onSurface.withValues(alpha: 0.5),
          );
        case ReplayMark.missed:
          canvas.drawCircle(center, 7, Paint()..color = CymbraColors.error);
        case ReplayMark.wrong:
          canvas.drawCircle(
            center,
            7,
            Paint()..color = CymbraColors.error.withValues(alpha: 0.5),
          );
        case ReplayMark.mistimed:
          canvas.drawCircle(
            center,
            8,
            Paint()
              ..style = PaintingStyle.stroke
              ..strokeWidth = 2.5
              ..color = CymbraColors.handLeft,
          );
        case ReplayMark.shortSustain:
          canvas.drawCircle(
            center,
            8,
            Paint()
              ..style = PaintingStyle.stroke
              ..strokeWidth = 2.5
              ..color = CymbraColors.primary,
          );
      }
    }

    // Scrubbing playhead.
    final px = progress.clamp(0.0, 1.0) * size.width;
    canvas.drawLine(
      Offset(px, 0),
      Offset(px, size.height),
      Paint()
        ..color = CymbraColors.secondary
        ..strokeWidth = 2,
    );
  }

  @override
  bool shouldRepaint(MistakeReplayPainter old) =>
      old.progress != progress || old.judgments != judgments;
}
