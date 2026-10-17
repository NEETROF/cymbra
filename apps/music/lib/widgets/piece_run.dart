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

import '../courses/course_manifest.dart';
import '../l10n/gen/app_localizations.dart';
import '../screens/open_score.dart';
import '../state/course_completion_notifier.dart';
import '../state/lesson_context.dart';
import '../state/player_data.dart';
import '../state/score_catalog.dart';
import '../state/session_summary.dart';
import '../state/today_ledger.dart';
import 'reward_celebration.dart';

/// The player settings a piece step imposes, as a one-run [LessonContext].
LessonContext lessonContextFor(
  PieceBlock block, {
  required String lessonTitle,
  required void Function(SessionResult result) onResult,
}) => LessonContext(
  lessonTitle: lessonTitle,
  hands: switch (block.hands) {
    LessonHands.right => Hand.right,
    LessonHands.left => Hand.left,
    LessonHands.both => Hand.both,
  },
  speed: block.speed,
  waitMode: block.waitMode,
  metronome: block.metronome,
  readingAid: switch (block.readingAid) {
    LessonReadingAid.off => NoteReadingAid.off,
    LessonReadingAid.name => NoteReadingAid.name,
    LessonReadingAid.nameAndRhythm => NoteReadingAid.nameAndRhythm,
  },
  passPct: block.passPct,
  onResult: onResult,
);

/// The piece's display title: the step's own, else the bundled score's, else
/// a generic word.
String pieceTitleOf(
  WidgetRef ref,
  PieceBlock block,
  AppLocalizations l10n,
  String lang,
) {
  final own = resolveInline(block.title, lang);
  if (own.isNotEmpty) return own;
  if (block.score.bundled case final id?) {
    final entry = ref
        .read(scoreCatalogProvider)
        .where((e) => e.id == id)
        .firstOrNull;
    if (entry != null) return entry.title;
  }
  return l10n.pieceCatalogPiece;
}

/// Runs a lesson's piece step in the full player and takes the learner back
/// (changes: add-music-piece-lessons, add-music-daily-session). Each finished
/// run is recorded against the course (first pass wins, best only rises) and
/// into today's ledger; a pass is celebrated once the learner is back, never
/// over the player. [onResult] sees each run too, for the caller's own state.
Future<PieceOpenOutcome> runPieceStep(
  BuildContext context,
  WidgetRef ref, {
  required String courseId,
  required String lessonTitle,
  required PieceBlock block,
  void Function(SessionResult result)? onResult,
}) async {
  double? lastPct;
  var passedRun = false;
  final lesson = lessonContextFor(
    block,
    lessonTitle: lessonTitle,
    onResult: (result) {
      final passed = result.overallSyncPct >= block.passPct;
      lastPct = result.overallSyncPct;
      passedRun = passedRun || passed;
      unawaited(
        ref
            .read(courseCompletionProvider.notifier)
            .recordPieceResult(
              courseId,
              passed: passed,
              syncPct: result.overallSyncPct,
            ),
      );
      unawaited(ref.read(dailyLedgerProvider.notifier).markPiece(courseId));
      onResult?.call(result);
    },
  );
  final outcome = await openScoreForLesson(
    context,
    ref,
    score: block.score,
    lesson: lesson,
  );
  if (!context.mounted) return outcome;
  final pct = lastPct;
  if (passedRun && pct != null) {
    final l10n = AppLocalizations.of(context);
    final lang = Localizations.localeOf(context).languageCode;
    unawaited(
      showRewardCelebration(
        context,
        title: l10n.celebrationPiecePassedTitle,
        message: l10n.celebrationPiecePassedMessage(
          pieceTitleOf(ref, block, l10n, lang),
          pct.round(),
        ),
        icon: Icons.music_note,
      ),
    );
  }
  return outcome;
}
