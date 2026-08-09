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
import '../theme/cymbra_theme.dart';

/// What the learner chose on the end-of-lesson celebration.
enum LessonCelebrationAction {
  /// Open the next lesson right away — the retention loop's one tap.
  next,

  /// Back to where they came from.
  close,
}

/// Shows the end-of-lesson celebration (change: add-notation-courses): a
/// reward-accent moment naming what was accomplished, the first-try stat when
/// the lesson had gated exercises, and — when a next lesson exists — a one-tap
/// "continue" into it. Follows the app's gamified-colour discipline: reward
/// moments wear the lilac accent, never the success green.
Future<LessonCelebrationAction> showLessonCelebration(
  BuildContext context, {
  required String lessonTitle,
  required int flawless,
  required int gated,
  String? nextLessonTitle,
}) async {
  final action = await showDialog<LessonCelebrationAction>(
    context: context,
    barrierDismissible: false,
    builder: (context) => _LessonCelebrationDialog(
      lessonTitle: lessonTitle,
      flawless: flawless,
      gated: gated,
      nextLessonTitle: nextLessonTitle,
    ),
  );
  return action ?? LessonCelebrationAction.close;
}

class _LessonCelebrationDialog extends StatelessWidget {
  const _LessonCelebrationDialog({
    required this.lessonTitle,
    required this.flawless,
    required this.gated,
    required this.nextLessonTitle,
  });

  final String lessonTitle;
  final int flawless;
  final int gated;
  final String? nextLessonTitle;

  @override
  Widget build(BuildContext context) {
    final l10n = AppLocalizations.of(context);
    const accent = CymbraColors.primary;
    return Dialog(
      backgroundColor: CymbraColors.surfaceContainerLow,
      shape: RoundedRectangleBorder(borderRadius: BorderRadius.circular(16)),
      child: ConstrainedBox(
        constraints: const BoxConstraints(maxWidth: 380),
        child: Padding(
          padding: const EdgeInsets.fromLTRB(24, 28, 24, 20),
          // Scrollable so a short landscape-phone viewport can never make the
          // celebration overflow — it scrolls instead.
          child: SingleChildScrollView(
            child: Column(
              mainAxisSize: MainAxisSize.min,
              children: [
                Container(
                  width: 72,
                  height: 72,
                  decoration: BoxDecoration(
                    color: accent.withValues(alpha: 0.15),
                    shape: BoxShape.circle,
                  ),
                  child: const Icon(Icons.school, size: 40, color: accent),
                ),
                const SizedBox(height: 16),
                Text(
                  l10n.lessonCelebrationTitle,
                  textAlign: TextAlign.center,
                  style: const TextStyle(
                    color: accent,
                    fontSize: 22,
                    fontWeight: FontWeight.w800,
                  ),
                ),
                const SizedBox(height: 6),
                Text(
                  lessonTitle,
                  textAlign: TextAlign.center,
                  style: const TextStyle(
                    color: CymbraColors.onSurface,
                    fontSize: 15,
                    fontWeight: FontWeight.w600,
                  ),
                ),
                if (gated > 0) ...[
                  const SizedBox(height: 10),
                  Text(
                    l10n.lessonCelebrationFlawless(flawless, gated),
                    key: const Key('lesson-celebration-stat'),
                    textAlign: TextAlign.center,
                    style: const TextStyle(
                      color: CymbraColors.onSurfaceVariant,
                      fontSize: 13,
                    ),
                  ),
                ],
                const SizedBox(height: 20),
                if (nextLessonTitle != null)
                  SizedBox(
                    width: double.infinity,
                    child: FilledButton(
                      key: const Key('lesson-celebration-next'),
                      style: FilledButton.styleFrom(
                        backgroundColor: CymbraColors.primaryContainer,
                      ),
                      onPressed: () => Navigator.of(
                        context,
                      ).pop(LessonCelebrationAction.next),
                      child: Text(
                        '${l10n.lessonContinueNext} — $nextLessonTitle',
                        maxLines: 1,
                        overflow: TextOverflow.ellipsis,
                      ),
                    ),
                  ),
                TextButton(
                  key: const Key('lesson-celebration-close'),
                  onPressed: () =>
                      Navigator.of(context).pop(LessonCelebrationAction.close),
                  child: Text(
                    l10n.lessonFinish,
                    style: const TextStyle(
                      color: CymbraColors.onSurfaceVariant,
                    ),
                  ),
                ),
              ],
            ),
          ),
        ),
      ),
    );
  }
}
