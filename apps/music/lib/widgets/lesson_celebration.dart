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
import 'celebration_surface.dart';

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
/// moments wear the lilac accent, never the success green. Animated through the
/// shared [CelebrationSurface] (change: add-music-motion-and-celebrations).
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
      child: CelebrationSurface(
        icon: Icons.school,
        accent: accent,
        title: l10n.lessonCelebrationTitle,
        subtitle: lessonTitle,
        extra: gated > 0
            ? Text(
                l10n.lessonCelebrationFlawless(flawless, gated),
                key: const Key('lesson-celebration-stat'),
                textAlign: TextAlign.center,
                style: const TextStyle(
                  color: CymbraColors.onSurfaceVariant,
                  fontSize: 13,
                ),
              )
            : null,
        actions: [
          if (nextLessonTitle != null)
            SizedBox(
              width: double.infinity,
              child: FilledButton(
                key: const Key('lesson-celebration-next'),
                style: FilledButton.styleFrom(
                  backgroundColor: CymbraColors.primaryContainer,
                ),
                onPressed: () =>
                    Navigator.of(context).pop(LessonCelebrationAction.next),
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
              style: const TextStyle(color: CymbraColors.onSurfaceVariant),
            ),
          ),
        ],
      ),
    );
  }
}
